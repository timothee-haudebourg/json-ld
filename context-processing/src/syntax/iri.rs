use super::{DefinedTerms, Merged};
use crate::{Error, Options, ProcessMeta, ProcessingStack, Warning, WarningHandler};
use contextual::WithContext;
use iref::{Iri, IriRef};
use json_ld_core::{Context, ContextLoader, Reference, Term};
use json_ld_syntax::{
	self as syntax,
	context::definition::{Key, KeyOrKeywordRef},
	ExpandableRef, Nullable,
};
use locspan::Meta;
use rdf_types::{BlankId, VocabularyMut};
use std::future::Future;
use syntax::{is_keyword_like, CompactIri};

pub struct MalformedIri(pub String);

impl From<MalformedIri> for Warning {
	fn from(MalformedIri(s): MalformedIri) -> Self {
		Self::MalformedIri(s)
	}
}

/// Default values for `document_relative` and `vocab` should be `false` and `true`.
pub fn expand_iri_with<
	'a,
	T: Clone + Send + Sync + PartialEq,
	B: Clone + Send + Sync + PartialEq,
	M: 'a + Clone + Send + Sync,
	C,
	N: Send + Sync + VocabularyMut<T, B>,
	L: ContextLoader<T, M> + Send + Sync,
	W: 'a + Send + WarningHandler<N, M>,
>(
	vocabulary: &'a mut N,
	active_context: &'a mut Context<T, B, C, M>,
	Meta(value, loc): Meta<Nullable<ExpandableRef<'a>>, M>,
	document_relative: bool,
	vocab: bool,
	local_context: &'a Merged<M, C>,
	defined: &'a mut DefinedTerms<M>,
	remote_contexts: ProcessingStack<T>,
	loader: &'a mut L,
	options: Options,
	mut warnings: W,
) -> impl 'a + Send + Future<Output = Result<(Term<T, B>, W), Error<L::ContextError>>>
where
	C: ProcessMeta<T, B, M>,
	L::Context: Into<C>,
{
	async move {
		match value {
			Nullable::Null => Ok((Term::Null, warnings)),
			Nullable::Some(ExpandableRef::Keyword(k)) => Ok((Term::Keyword(k), warnings)),
			Nullable::Some(ExpandableRef::String(value)) => {
				if is_keyword_like(value) {
					return Ok((Term::Null, warnings));
				}

				// If `local_context` is not null, it contains an entry with a key that equals value, and the
				// value of the entry for value in defined is not true, invoke the Create Term Definition
				// algorithm, passing active context, local context, value as term, and defined. This will
				// ensure that a term definition is created for value in active context during Context
				// Processing.
				warnings = super::define(
					vocabulary,
					active_context,
					local_context,
					Meta(value.into(), loc.clone()),
					defined,
					remote_contexts.clone(),
					loader,
					None,
					false,
					options.with_no_override(),
					warnings,
				)
				.await?;

				if let Some(term_definition) = active_context.get(value) {
					// If active context has a term definition for value, and the associated IRI mapping
					// is a keyword, return that keyword.
					if let Some(value) = &term_definition.value {
						if value.is_keyword() {
							return Ok((value.clone(), warnings));
						}
					}

					// If vocab is true and the active context has a term definition for value, return the
					// associated IRI mapping.
					if vocab {
						return match &term_definition.value {
							Some(value) => Ok((value.clone(), warnings)),
							None => Ok((Term::Null, warnings)),
						};
					}
				}

				if value.find(':').map(|i| i > 0).unwrap_or(false) {
					if let Ok(blank_id) = BlankId::new(value) {
						return Ok((
							Term::Ref(Reference::blank(vocabulary.insert_blank_id(blank_id))),
							warnings,
						));
					}

					if value == "_:" {
						return Ok((Term::Ref(Reference::Invalid("_:".to_string())), warnings));
					}

					if let Ok(compact_iri) = CompactIri::new(value) {
						// If local context is not null, it contains a `prefix` entry, and the value of the
						// prefix entry in defined is not true, invoke the Create Term Definition
						// algorithm, passing active context, local context, prefix as term, and defined.
						// This will ensure that a term definition is created for prefix in active context
						// during Context Processing.
						warnings = super::define(
							vocabulary,
							active_context,
							local_context,
							Meta(
								KeyOrKeywordRef::Key(compact_iri.prefix().into()),
								loc.clone(),
							),
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
						let prefix_key = Key::from(compact_iri.prefix().to_string());
						if let Some(term_definition) = active_context.get(&prefix_key) {
							if term_definition.prefix {
								if let Some(mapping) = &term_definition.value {
									let mut result =
										mapping.with(&*vocabulary).as_str().to_string();
									result.push_str(compact_iri.suffix());

									return Ok((
										Term::Ref(Reference::from_string_in(vocabulary, result)),
										warnings,
									));
								}
							}
						}
					}

					if let Ok(iri) = Iri::new(value) {
						return Ok((Term::Ref(Reference::id(vocabulary.insert(iri))), warnings));
					}
				}

				// If vocab is true, and active context has a vocabulary mapping, return the result of
				// concatenating the vocabulary mapping with value.
				if vocab {
					match active_context.vocabulary() {
						Some(Term::Ref(mapping)) => {
							let mut result = mapping.with(&*vocabulary).as_str().to_string();
							result.push_str(value);

							return Ok((
								Term::Ref(Reference::from_string_in(vocabulary, result)),
								warnings,
							));
						}
						Some(_) => {
							return Ok(invalid_iri::<_, _, _, _, Warning, _>(
								vocabulary,
								Meta(value.to_string(), loc),
								warnings,
							))
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
					if let Ok(iri_ref) = IriRef::new(value) {
						if let Some(iri) =
							super::resolve_iri(vocabulary, iri_ref, active_context.base_iri())
						{
							return Ok((Term::from(iri), warnings));
						}
					}
				}

				// Return value as is.
				Ok(invalid_iri::<_, _, _, _, Warning, _>(
					vocabulary,
					Meta(value.to_string(), loc),
					warnings,
				))
			}
		}
	}
}

fn invalid_iri<
	T,
	B,
	N,
	M,
	W: From<MalformedIri>,
	H: json_ld_core::warning::Handler<N, Meta<Warning, M>>,
>(
	vocabulary: &N,
	Meta(value, loc): Meta<String, M>,
	mut warnings: H,
) -> (Term<T, B>, H) {
	warnings.handle(vocabulary, Meta(MalformedIri(value.clone()).into(), loc));

	(Term::Ref(Reference::Invalid(value)), warnings)
}

/// Default values for `document_relative` and `vocab` should be `false` and `true`.
pub fn expand_iri_simple<
	'a,
	T: Clone,
	B: Clone,
	M: Clone,
	N: VocabularyMut<T, B>,
	C,
	W: From<MalformedIri>,
	H: json_ld_core::warning::Handler<N, Meta<W, M>>,
>(
	vocabulary: &'a mut N,
	active_context: &'a Context<T, B, C, M>,
	Meta(value, meta): Meta<Nullable<ExpandableRef<'a>>, M>,
	document_relative: bool,
	vocab: bool,
	warnings: &mut H,
) -> Meta<Term<T, B>, M> {
	match value {
		Nullable::Null => Meta(Term::Null, meta),
		Nullable::Some(ExpandableRef::Keyword(k)) => Meta(Term::Keyword(k), meta),
		Nullable::Some(ExpandableRef::String(value)) => {
			if is_keyword_like(value) {
				return Meta(Term::Null, meta);
			}

			if let Some(term_definition) = active_context.get(value) {
				// If active context has a term definition for value, and the associated IRI mapping
				// is a keyword, return that keyword.
				if let Some(value) = &term_definition.value {
					if value.is_keyword() {
						return Meta(value.clone(), meta);
					}
				}

				// If vocab is true and the active context has a term definition for value, return the
				// associated IRI mapping.
				if vocab {
					return match &term_definition.value {
						Some(value) => Meta(value.clone(), meta),
						None => Meta(Term::Null, meta),
					};
				}
			}

			if value.find(':').map(|i| i > 0).unwrap_or(false) {
				if let Ok(blank_id) = BlankId::new(value) {
					return Meta(
						Term::Ref(Reference::blank(vocabulary.insert_blank_id(blank_id))),
						meta,
					);
				}

				if value == "_:" {
					return Meta(Term::Ref(Reference::Invalid("_:".to_string())), meta);
				}

				if let Ok(compact_iri) = CompactIri::new(value) {
					// If active context contains a term definition for prefix having a non-null IRI
					// mapping and the prefix flag of the term definition is true, return the result
					// of concatenating the IRI mapping associated with prefix and suffix.
					let prefix_key = Key::from(compact_iri.prefix().to_string());
					if let Some(term_definition) = active_context.get(&prefix_key) {
						if term_definition.prefix {
							if let Some(mapping) = &term_definition.value {
								let mut result = mapping.with(&*vocabulary).as_str().to_string();
								result.push_str(compact_iri.suffix());

								return Meta(
									Term::Ref(Reference::from_string_in(vocabulary, result)),
									meta,
								);
							}
						}
					}
				}

				if let Ok(iri) = Iri::new(value) {
					return Meta(Term::Ref(Reference::id(vocabulary.insert(iri))), meta);
				}
			}

			// If vocab is true, and active context has a vocabulary mapping, return the result of
			// concatenating the vocabulary mapping with value.
			if vocab {
				match active_context.vocabulary() {
					Some(Term::Ref(mapping)) => {
						let mut result = mapping.with(&*vocabulary).as_str().to_string();
						result.push_str(value);

						return Meta(
							Term::Ref(Reference::from_string_in(vocabulary, result)),
							meta,
						);
					}
					Some(_) => {
						return invalid_iri_simple(
							vocabulary,
							Meta(value.to_string(), meta),
							warnings,
						)
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
				if let Ok(iri_ref) = IriRef::new(value) {
					if let Some(iri) =
						super::resolve_iri(vocabulary, iri_ref, active_context.base_iri())
					{
						return Meta(Term::from(iri), meta);
					}
				}
			}

			// Return value as is.
			invalid_iri_simple(vocabulary, Meta(value.to_string(), meta), warnings)
		}
	}
}

fn invalid_iri_simple<
	T,
	B,
	N,
	M: Clone,
	W: From<MalformedIri>,
	H: json_ld_core::warning::Handler<N, Meta<W, M>>,
>(
	vocabulary: &N,
	Meta(value, meta): Meta<String, M>,
	warnings: &mut H,
) -> Meta<Term<T, B>, M> {
	warnings.handle(
		vocabulary,
		Meta(MalformedIri(value.clone()).into(), meta.clone()),
	);
	Meta(Term::Ref(Reference::Invalid(value)), meta)
}
