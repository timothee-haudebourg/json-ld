use std::future::Future;
use std::collections::HashMap;
use json_ld_core::{
	Loc,
	syntax::{Term, Keyword, is_keyword_like},
	Reference,
	Context,
	Id
};
use iref::{Iri, IriRef, IriBuf};
use rdf_types::BlankIdBuf;
use crate::{
	Warning,
	Loader,
	Process,
	ProcessingOptions,
	ProcessingStack,
	Error
};
use super::{
	JsonContext,
	LocalContextObject,
	define
};

/// Resolve `iri_ref` against the given base IRI.
fn resolve_iri(iri_ref: IriRef, base_iri: Option<Iri>) -> Option<IriBuf> {
	match base_iri {
		Some(base_iri) => Some(iri_ref.resolved(base_iri)),
		None => match iri_ref.into_iri() {
			Ok(iri) => Some(iri.into()),
			Err(_) => None,
		},
	}
}

/// Build an invalid reference and emit a warning.
fn invalid_iri<T, S, M: Clone>(
	value: String,
	source: Option<S>,
	metadata: &M,
	warnings: &mut Vec<Loc<Warning, S, M>>,
) -> Term<T> {
	warnings.push(Loc::new(
		Warning::MalformedIri(value.clone()),
		source,
		metadata.clone(),
	));
	Reference::Invalid(value).into()
}

/// Default values for `document_relative` and `vocab` should be `false` and `true`.
fn expand_iri<
	'a,
	J: JsonContext,
	T: Id + Send + Sync,
	C: Process<T>,
	L: Loader + Send + Sync,
>(
	active_context: &'a mut Context<T, C>,
	value: &str,
	source: Option<C::Source>,
	metadata: &'a J::MetaData,
	document_relative: bool,
	vocab: bool,
	local_context: &'a LocalContextObject<'a, J::Object>,
	defined: &'a mut HashMap<String, bool>,
	remote_contexts: ProcessingStack,
	loader: &'a mut L,
	options: ProcessingOptions,
	warnings: &'a mut Vec<Loc<Warning, C::Source, C::MetaData>>,
) -> impl 'a + Send + Future<Output = Result<Term<T>, Error>>
where
	L::Output: Into<J>,
{
	let value = value.to_string();
	async move {
		if let Ok(keyword) = Keyword::try_from(value.as_ref()) {
			Ok(Term::Keyword(keyword))
		} else {
			// If value has the form of a keyword, a processor SHOULD generate a warning and return
			// null.
			if is_keyword_like(value.as_ref()) {
				warnings.push(Loc::new(
					Warning::KeywordLikeValue(value),
					source,
					metadata.clone(),
				));
				return Ok(Term::Null);
			}

			// If `local_context` is not null, it contains an entry with a key that equals value, and the
			// value of the entry for value in defined is not true, invoke the Create Term Definition
			// algorithm, passing active context, local context, value as term, and defined. This will
			// ensure that a term definition is created for value in active context during Context
			// Processing.
			define(
				active_context,
				local_context,
				value.as_ref(),
				metadata,
				defined,
				remote_contexts.clone(),
				loader,
				None,
				false,
				options.with_no_override(),
				warnings,
			)
			.await?;

			if let Some(term_definition) = active_context.get(value.as_ref()) {
				// If active context has a term definition for value, and the associated IRI mapping
				// is a keyword, return that keyword.
				if let Some(value) = &term_definition.value {
					if value.is_keyword() {
						return Ok(value.clone());
					}
				}

				// If vocab is true and the active context has a term definition for value, return the
				// associated IRI mapping.
				if vocab {
					if let Some(value) = &term_definition.value {
						return Ok(value.clone());
					} else {
						return Ok(invalid_iri(value.to_string(), source, metadata, warnings));
					}
				}
			}

			// If value contains a colon (:) anywhere after the first character, it is either an IRI,
			// a compact IRI, or a blank node identifier:
			if let Some(index) = value.find(':') {
				if index > 0 {
					// Split value into a prefix and suffix at the first occurrence of a colon (:).
					let (prefix, suffix) = value.split_at(index);
					let suffix = &suffix[1..suffix.len()];

					// If prefix is underscore (_) or suffix begins with double-forward-slash (//),
					// return value as it is already an IRI or a blank node identifier.
					if prefix == "_" {
						return Ok(Term::from(BlankIdBuf::from_suffix(suffix)));
					}

					if suffix.starts_with("//") {
						if let Ok(iri) = Iri::new(value.as_ref() as &str) {
							return Ok(Term::from(T::from_iri(iri)));
						} else {
							return Ok(invalid_iri(value.to_string(), source, metadata, warnings));
						}
					}

					// If local context is not null, it contains a `prefix` entry, and the value of the
					// prefix entry in defined is not true, invoke the Create Term Definition
					// algorithm, passing active context, local context, prefix as term, and defined.
					// This will ensure that a term definition is created for prefix in active context
					// during Context Processing.
					define(
						active_context,
						local_context,
						prefix,
						metadata,
						defined,
						remote_contexts,
						loader,
						None,
						false,
						options.with_no_override(),
						warnings,
					)
					.await?;

					// If active context contains a term definition for prefix having a non-null IRI
					// mapping and the prefix flag of the term definition is true, return the result
					// of concatenating the IRI mapping associated with prefix and suffix.
					if let Some(term_definition) = active_context.get(prefix) {
						if term_definition.prefix {
							if let Some(mapping) = &term_definition.value {
								let mut result = mapping.as_str().to_string();
								result.push_str(suffix);

								if let Ok(result) = Iri::new(&result) {
									return Ok(Term::from(T::from_iri(result)));
								} else if let Ok(blank) = BlankIdBuf::from_suffix(result.as_ref()) {
									return Ok(Term::from(blank));
								} else {
									return Ok(Reference::Invalid(result).into());
								}
							}
						}
					}

					// If value has the form of an IRI, return value.
					if let Ok(result) = Iri::new(value.as_ref() as &str) {
						return Ok(Term::from(T::from_iri(result)));
					}
				}
			}

			// If vocab is true, and active context has a vocabulary mapping, return the result of
			// concatenating the vocabulary mapping with value.
			if vocab {
				match active_context.vocabulary() {
					Some(Term::Ref(mapping)) => {
						let mut result = mapping.as_str().to_string();
						result.push_str(value.as_ref());

						if let Ok(result) = Iri::new(&result) {
							return Ok(Term::from(T::from_iri(result)));
						} else if let Ok(blank) = BlankIdBuf::from_suffix(result.as_ref()) {
							return Ok(Term::from(blank));
						} else {
							return Ok(Reference::Invalid(result).into());
						}
					}
					Some(_) => {
						return Ok(invalid_iri(value.to_string(), source, metadata, warnings))
					}
					None => (),
				}
			}

			// Otherwise, if document relative is true set value to the result of resolving value
			// against the base IRI from active context. Only the basic algorithm in section 5.2 of
			// [RFC3986] is used; neither Syntax-Based Normalization nor Scheme-Based Normalization
			// are performed. Characters additionally allowed in IRI references are treated in the
			// same way that unreserved characters are treated in URI references, per section 6.5 of
			// [RFC3987].
			if document_relative {
				if let Ok(iri_ref) = IriRef::new(value.as_ref() as &str) {
					if let Some(value) = resolve_iri(iri_ref, active_context.base_iri()) {
						return Ok(Term::from(T::from_iri(value.as_iri())));
					} else {
						return Ok(invalid_iri(value.to_string(), source, metadata, warnings));
					}
				} else {
					return Ok(invalid_iri(value.to_string(), source, metadata, warnings));
				}
			}

			// Return value as is.
			Ok(invalid_iri(value.to_string(), source, metadata, warnings))
		}
	}
}
