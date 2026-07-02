use rdf_syntax::BlankId;
use rdf_syntax::{Iri, IriBuf, IriRef};

use crate::Lenient;
use crate::algorithms::JsonLdLocatedError;
use crate::{
	Nullable, Term,
	algorithms::{
		AsyncProcessingEnvironment, JsonLdBacktraceBuilder,
		context_processing::{ContextProcessor, TargetProcessedContext, merged::Merged},
		warning::Warning,
	},
	context::RawProcessedContext,
	syntax::{
		CompactIri, ExpandableRef,
		context::{ContextTerm, KeyOrKeywordRef},
		is_keyword_like,
	},
};

/// Resolve `iri_ref` against the given base IRI.
pub fn resolve_iri(iri_ref: &IriRef, base_iri: Option<&Iri>) -> Option<IriBuf> {
	match base_iri {
		Some(base_iri) => Some(iri_ref.resolved(base_iri)),
		None => iri_ref.as_iri().map(ToOwned::to_owned),
	}
}

/// Result of the [`expand_iri_with`] function.
pub type ExpandIriResult = Result<Term, JsonLdLocatedError>;

impl<'a> ContextProcessor<'a> {
	pub async fn expand_iri_recursive(
		&self,
		env: &impl AsyncProcessingEnvironment,
		result: &mut TargetProcessedContext<'_>,
		local_context: &Merged<'_>,
		value: Nullable<ExpandableRef<'_>>,
		location: JsonLdBacktraceBuilder<'_>,
	) -> ExpandIriResult {
		match value {
			Nullable::Null => Ok(Term::Null),
			Nullable::Some(ExpandableRef::Keyword(k)) => Ok(Term::Keyword(k)),
			Nullable::Some(ExpandableRef::String(value)) => {
				if is_keyword_like(value) {
					return Ok(Term::Null);
				}

				// If `local_context` is not null, it contains an entry with a key that equals value, and the
				// value of the entry for value in defined is not true, invoke the Create Term Definition
				// algorithm, passing active context, local context, value as term, and defined. This will
				// ensure that a term definition is created for value in active context during Context
				// Processing.
				Box::pin(self.for_recursive_definition().define(
					env,
					result,
					local_context,
					value.into(),
					false,
					location,
				))
				.await?;

				if let Some(term_definition) = result.value.get(value) {
					// If active context has a term definition for value, and the associated IRI mapping
					// is a keyword, return that keyword.
					if let Some(value) = term_definition.value()
						&& value.is_keyword()
					{
						return Ok(value.clone());
					}

					// Return the associated IRI mapping.
					return match term_definition.value() {
						Some(value) => Ok(value.clone()),
						None => Ok(Term::Null),
					};
				}

				if value.find(':').map(|i| i > 0).unwrap_or(false) {
					if let Ok(blank_id) = BlankId::new(value) {
						return Ok(Term::Id(Lenient::blank(blank_id.to_owned())));
					}

					if value == "_:" {
						return Ok(Term::Id(Lenient::Invalid("_:".to_string())));
					}

					if let Ok(compact_iri) = CompactIri::new(value) {
						// If local context is not null, it contains a `prefix` entry, and the value of the
						// prefix entry in defined is not true, invoke the Create Term Definition
						// algorithm, passing active context, local context, prefix as term, and defined.
						// This will ensure that a term definition is created for prefix in active context
						// during Context Processing.
						Box::pin(self.for_recursive_definition().define(
							env,
							result,
							local_context,
							KeyOrKeywordRef::Key(compact_iri.prefix().into()),
							false,
							location,
						))
						.await?;

						// If active context contains a term definition for prefix having a non-null IRI
						// mapping and the prefix flag of the term definition is true, return the result
						// of concatenating the IRI mapping associated with prefix and suffix.
						let prefix_key = ContextTerm::from(compact_iri.prefix().to_string());
						if let Some(term_definition) = result.value.get_normal(&prefix_key)
							&& term_definition.prefix
							&& let Some(mapping) = &term_definition.value
						{
							let mut result = mapping.as_str().to_owned();
							result.push_str(compact_iri.suffix());

							return Ok(Term::Id(Lenient::from_string(result).0));
						}
					}

					if let Ok(iri) = Iri::new(value) {
						return Ok(Term::Id(Lenient::iri(iri.to_owned())));
					}
				}

				// If active context has a vocabulary mapping, return the result of
				// concatenating the vocabulary mapping with value.
				match result.value.vocabulary() {
					Some(Term::Id(mapping)) => {
						let mut result = mapping.as_str().to_owned();
						result.push_str(value);

						return Ok(Term::Id(Lenient::from_string(result).0));
					}
					Some(_) => return Ok(invalid_iri(value.to_owned(), |w| env.warn(w))),
					None => (),
				}

				// Return value as is.
				Ok(invalid_iri(value.to_owned(), |w| env.warn(w)))
			}
		}
	}
}

impl RawProcessedContext {
	pub fn expand_iri(
		&self,
		value: Nullable<ExpandableRef<'_>>,
		document_relative: bool,
		vocab: bool,
	) -> Term {
		self.expand_iri_with(value, document_relative, vocab, |_| {})
	}

	pub fn expand_iri_with(
		&self,
		value: Nullable<ExpandableRef<'_>>,
		document_relative: bool,
		vocab: bool,
		on_warning: impl FnOnce(Warning),
	) -> Term {
		match value {
			Nullable::Null => Term::Null,
			Nullable::Some(ExpandableRef::Keyword(k)) => Term::Keyword(k),
			Nullable::Some(ExpandableRef::String(value)) => {
				if is_keyword_like(value) {
					return Term::Null;
				}

				if let Some(term_definition) = self.get(value) {
					// If active context has a term definition for value, and the associated IRI mapping
					// is a keyword, return that keyword.
					if let Some(value) = term_definition.value()
						&& value.is_keyword()
					{
						return value.clone();
					}

					// If vocab is true and the active context has a term definition for value, return the
					// associated IRI mapping.
					if vocab {
						return match term_definition.value() {
							Some(value) => value.clone(),
							None => Term::Null,
						};
					}
				}

				if value.find(':').map(|i| i > 0).unwrap_or(false) {
					if let Ok(blank_id) = BlankId::new(value) {
						return Term::Id(Lenient::blank(blank_id.to_owned()));
					}

					if value == "_:" {
						return Term::Id(Lenient::Invalid("_:".to_string()));
					}

					if let Ok(compact_iri) = CompactIri::new(value) {
						// If active context contains a term definition for prefix having a non-null IRI
						// mapping and the prefix flag of the term definition is true, return the result
						// of concatenating the IRI mapping associated with prefix and suffix.
						let prefix_key = ContextTerm::from(compact_iri.prefix().to_string());
						if let Some(term_definition) = self.get_normal(&prefix_key)
							&& term_definition.prefix
							&& let Some(mapping) = &term_definition.value
						{
							let mut result = mapping.as_str().to_owned();
							result.push_str(compact_iri.suffix());
							return Term::Id(Lenient::from_string(result).0);
						}
					}

					if let Ok(iri) = Iri::new(value) {
						return Term::Id(Lenient::iri(iri.to_owned()));
					}
				}

				// If vocab is true, and active context has a vocabulary mapping, return the result of
				// concatenating the vocabulary mapping with value.
				if vocab {
					match self.vocabulary() {
						Some(Term::Id(mapping)) => {
							let mut result = mapping.as_str().to_owned();
							result.push_str(value);

							return Term::Id(Lenient::from_string(result).0);
						}
						Some(_) => return invalid_iri(value.to_string(), on_warning),
						None => (),
					}
				}

				// Otherwise, if document relative is true set value to the result of resolving value
				// against the base IRI from active context. Only the basic algorithm in section 5.2 of
				// [RFC3986] is used; neither Syntax-Based Normalization nor Scheme-Based Normalization
				// are performed. Characters additionally allowed in IRI references are treated in the
				// same way that unreserved characters are treated in URI references, per section 6.5 of
				// [RFC3987].
				if document_relative
					&& let Ok(iri_ref) = IriRef::new(value)
					&& let Some(iri) = resolve_iri(iri_ref, self.base_iri())
				{
					return Term::from(iri);
				}

				// Return value as is.
				invalid_iri(value.to_string(), on_warning)
			}
		}
	}
}

fn invalid_iri(value: String, on_warning: impl FnOnce(Warning)) -> Term {
	(on_warning)(Warning::MalformedIri(value.clone()));
	Term::Id(Lenient::Invalid(value))
}
