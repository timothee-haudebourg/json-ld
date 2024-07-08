use std::hash::Hash;

use super::{DefinedTerms, Environment, Merged};
use crate::{Error, Options, ProcessingStack, Warning, WarningHandler};
use contextual::WithContext;
use iref::{Iri, IriRef};
use json_ld_core::{warning, Context, Id, Loader, Term};
use json_ld_syntax::{self as syntax, context::definition::Key, ExpandableRef, Nullable};
use rdf_types::{
	vocabulary::{BlankIdVocabulary, IriVocabulary},
	BlankId, Vocabulary, VocabularyMut,
};
use syntax::{context::definition::KeyOrKeywordRef, is_keyword_like, CompactIri};

pub struct MalformedIri(pub String);

impl From<MalformedIri> for Warning {
	fn from(MalformedIri(s): MalformedIri) -> Self {
		Self::MalformedIri(s)
	}
}

/// Result of the [`expand_iri_with`] function.
pub type ExpandIriResult<T, B> = Result<Option<Term<T, B>>, Error>;

/// Default values for `document_relative` and `vocab` should be `false` and `true`.
#[allow(clippy::too_many_arguments)]
pub async fn expand_iri_with<'a, N, L, W>(
	mut env: Environment<'a, N, L, W>,
	active_context: &'a mut Context<N::Iri, N::BlankId>,
	value: Nullable<ExpandableRef<'a>>,
	document_relative: bool,
	vocab: Option<Action>,
	local_context: &'a Merged<'a>,
	defined: &'a mut DefinedTerms,
	remote_contexts: ProcessingStack<N::Iri>,
	options: Options,
) -> ExpandIriResult<N::Iri, N::BlankId>
where
	N: VocabularyMut,
	N::Iri: Clone + Eq + Hash,
	N::BlankId: Clone + PartialEq,
	L: Loader,
	W: WarningHandler<N>,
{
	match value {
		Nullable::Null => Ok(Some(Term::Null)),
		Nullable::Some(ExpandableRef::Keyword(k)) => Ok(Some(Term::Keyword(k))),
		Nullable::Some(ExpandableRef::String(value)) => {
			if is_keyword_like(value) {
				return Ok(Some(Term::Null));
			}

			// If `local_context` is not null, it contains an entry with a key that equals value, and the
			// value of the entry for value in defined is not true, invoke the Create Term Definition
			// algorithm, passing active context, local context, value as term, and defined. This will
			// ensure that a term definition is created for value in active context during Context
			// Processing.
			Box::pin(super::define(
				Environment {
					vocabulary: env.vocabulary,
					loader: env.loader,
					warnings: env.warnings,
				},
				active_context,
				local_context,
				value.into(),
				defined,
				remote_contexts.clone(),
				None,
				false,
				options.with_no_override(),
			))
			.await?;

			if let Some(term_definition) = active_context.get(value) {
				// If active context has a term definition for value, and the associated IRI mapping
				// is a keyword, return that keyword.
				if let Some(value) = term_definition.value() {
					if value.is_keyword() {
						return Ok(Some(value.clone()));
					}
				}

				// If vocab is true and the active context has a term definition for value, return the
				// associated IRI mapping.
				if vocab.is_some() {
					return match term_definition.value() {
						Some(value) => Ok(Some(value.clone())),
						None => Ok(Some(Term::Null)),
					};
				}
			}

			if value.find(':').map(|i| i > 0).unwrap_or(false) {
				if let Ok(blank_id) = BlankId::new(value) {
					return Ok(Some(Term::Id(Id::blank(
						env.vocabulary.insert_blank_id(blank_id),
					))));
				}

				if value == "_:" {
					return Ok(Some(Term::Id(Id::Invalid("_:".to_string()))));
				}

				if let Ok(compact_iri) = CompactIri::new(value) {
					// If local context is not null, it contains a `prefix` entry, and the value of the
					// prefix entry in defined is not true, invoke the Create Term Definition
					// algorithm, passing active context, local context, prefix as term, and defined.
					// This will ensure that a term definition is created for prefix in active context
					// during Context Processing.
					Box::pin(super::define(
						Environment {
							vocabulary: env.vocabulary,
							loader: env.loader,
							warnings: env.warnings,
						},
						active_context,
						local_context,
						KeyOrKeywordRef::Key(compact_iri.prefix().into()),
						defined,
						remote_contexts,
						None,
						false,
						options.with_no_override(),
					))
					.await?;

					// If active context contains a term definition for prefix having a non-null IRI
					// mapping and the prefix flag of the term definition is true, return the result
					// of concatenating the IRI mapping associated with prefix and suffix.
					let prefix_key = Key::from(compact_iri.prefix().to_string());
					if let Some(term_definition) = active_context.get_normal(&prefix_key) {
						if term_definition.prefix {
							if let Some(mapping) = &term_definition.value {
								let mut result =
									mapping.with(&*env.vocabulary).as_str().to_string();
								result.push_str(compact_iri.suffix());

								return Ok(Some(Term::Id(Id::from_string_in(
									env.vocabulary,
									result,
								))));
							}
						}
					}
				}

				if let Ok(iri) = Iri::new(value) {
					return Ok(Some(Term::Id(Id::iri(env.vocabulary.insert(iri)))));
				}
			}

			// If vocab is true, and active context has a vocabulary mapping, return the result of
			// concatenating the vocabulary mapping with value.
			if let Some(action) = vocab {
				match active_context.vocabulary() {
					Some(Term::Id(mapping)) => {
						return match action {
							Action::Keep => {
								let mut result =
									mapping.with(&*env.vocabulary).as_str().to_string();
								result.push_str(value);

								Ok(Some(Term::Id(Id::from_string_in(env.vocabulary, result))))
							}
							Action::Drop => Ok(None),
							Action::Reject => Err(Error::ForbiddenVocab),
						}
					}
					Some(_) => return Ok(Some(invalid_iri(&mut env, value.to_string()))),
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
						super::resolve_iri(env.vocabulary, iri_ref, active_context.base_iri())
					{
						return Ok(Some(Term::from(iri)));
					}
				}
			}

			// Return value as is.
			Ok(Some(invalid_iri(&mut env, value.to_string())))
		}
	}
}

fn invalid_iri<N, L, W: json_ld_core::warning::Handler<N, Warning>>(
	env: &mut Environment<N, L, W>,
	value: String,
) -> Term<N::Iri, N::BlankId>
where
	N: Vocabulary,
{
	env.warnings
		.handle(env.vocabulary, MalformedIri(value.clone()).into());
	Term::Id(Id::Invalid(value))
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Action {
	#[default]
	Keep,
	Drop,
	Reject,
}

impl Action {
	pub fn is_reject(&self) -> bool {
		matches!(self, Self::Reject)
	}
}

#[derive(Debug)]
pub struct RejectVocab;

pub type IriExpansionResult<N> =
	Result<Option<Term<<N as IriVocabulary>::Iri, <N as BlankIdVocabulary>::BlankId>>, RejectVocab>;

/// Default values for `document_relative` and `vocab` should be `false` and `true`.
pub fn expand_iri_simple<W, N, L, H>(
	env: &mut Environment<N, L, H>,
	active_context: &Context<N::Iri, N::BlankId>,
	value: Nullable<ExpandableRef>,
	document_relative: bool,
	vocab: Option<Action>,
) -> IriExpansionResult<N>
where
	N: VocabularyMut,
	N::Iri: Clone,
	N::BlankId: Clone,
	W: From<MalformedIri>,
	H: warning::Handler<N, W>,
{
	match value {
		Nullable::Null => Ok(Some(Term::Null)),
		Nullable::Some(ExpandableRef::Keyword(k)) => Ok(Some(Term::Keyword(k))),
		Nullable::Some(ExpandableRef::String(value)) => {
			if is_keyword_like(value) {
				return Ok(Some(Term::Null));
			}

			if let Some(term_definition) = active_context.get(value) {
				// If active context has a term definition for value, and the associated IRI mapping
				// is a keyword, return that keyword.
				if let Some(value) = term_definition.value() {
					if value.is_keyword() {
						return Ok(Some(value.clone()));
					}
				}

				// If vocab is true and the active context has a term definition for value, return the
				// associated IRI mapping.
				if vocab.is_some() {
					return match term_definition.value() {
						Some(value) => Ok(Some(value.clone())),
						None => Ok(Some(Term::Null)),
					};
				}
			}

			if value.find(':').map(|i| i > 0).unwrap_or(false) {
				if let Ok(blank_id) = BlankId::new(value) {
					return Ok(Some(Term::Id(Id::blank(
						env.vocabulary.insert_blank_id(blank_id),
					))));
				}

				if value == "_:" {
					return Ok(Some(Term::Id(Id::Invalid("_:".to_string()))));
				}

				if let Ok(compact_iri) = CompactIri::new(value) {
					// If active context contains a term definition for prefix having a non-null IRI
					// mapping and the prefix flag of the term definition is true, return the result
					// of concatenating the IRI mapping associated with prefix and suffix.
					let prefix_key = Key::from(compact_iri.prefix().to_string());
					if let Some(term_definition) = active_context.get_normal(&prefix_key) {
						if term_definition.prefix {
							if let Some(mapping) = &term_definition.value {
								let mut result =
									mapping.with(&*env.vocabulary).as_str().to_string();
								result.push_str(compact_iri.suffix());

								return Ok(Some(Term::Id(Id::from_string_in(
									env.vocabulary,
									result,
								))));
							}
						}
					}
				}

				if let Ok(iri) = Iri::new(value) {
					return Ok(Some(Term::Id(Id::iri(env.vocabulary.insert(iri)))));
				}
			}

			// If vocab is true, and active context has a vocabulary mapping, return the result of
			// concatenating the vocabulary mapping with value.
			if let Some(action) = vocab {
				match active_context.vocabulary() {
					Some(Term::Id(mapping)) => {
						return match action {
							Action::Keep => {
								let mut result =
									mapping.with(&*env.vocabulary).as_str().to_string();
								result.push_str(value);

								Ok(Some(Term::Id(Id::from_string_in(env.vocabulary, result))))
							}
							Action::Drop => Ok(None),
							Action::Reject => Err(RejectVocab),
						}
					}
					Some(_) => return Ok(Some(invalid_iri_simple(env, value.to_string()))),
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
						super::resolve_iri(env.vocabulary, iri_ref, active_context.base_iri())
					{
						return Ok(Some(Term::from(iri)));
					}
				}
			}

			// Return value as is.
			Ok(Some(invalid_iri_simple(env, value.to_string())))
		}
	}
}

fn invalid_iri_simple<W, N, L, H>(
	env: &mut Environment<N, L, H>,
	value: String,
) -> Term<N::Iri, N::BlankId>
where
	N: Vocabulary,
	W: From<MalformedIri>,
	H: warning::Handler<N, W>,
{
	env.warnings
		.handle(env.vocabulary, MalformedIri(value.clone()).into());
	Term::Id(Id::Invalid(value))
}
