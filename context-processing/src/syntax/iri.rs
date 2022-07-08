use json_ld_core::{
	Id,
	Context,
	Term,
	Reference
};
use json_ld_syntax::{
	self as syntax,
	Nullable,
	ExpandableRef,
	context::{
		Key,
		KeyRef,
		KeyOrKeywordRef
	}
};
use locspan::Meta;
use iref::IriRefBuf;
use std::future::Future;
use crate::{
	ProcessingOptions,
	ProcessingStack,
	ContextLoader,
	Warning,
	MetaWarning,
	Error
};
use super::{
	Merged,
	DefinedTerms
};

pub struct MalformedIri(pub String);

impl From<MalformedIri> for Warning {
	fn from(MalformedIri(s): MalformedIri) -> Self {
		Self::MalformedIri(s)
	}
}

/// Default values for `document_relative` and `vocab` should be `false` and `true`.
pub fn expand_iri_with<
	'a,
	T: Id + Send + Sync,
	C: syntax::AnyContextEntry + Send + Sync,
	L: ContextLoader + Send + Sync
>(
	active_context: &'a mut Context<T, C>,
	Meta(value, loc): Meta<Nullable<ExpandableRef<'a>>, C::Metadata>,
	document_relative: bool,
	vocab: bool,
	local_context: &'a Merged<C>,
	defined: &'a mut DefinedTerms<C>,
	remote_contexts: ProcessingStack,
	loader: &'a mut L,
	options: ProcessingOptions,
	mut warnings: impl 'a + Send + FnMut(MetaWarning<C>),
) -> impl 'a + Send + Future<Output = Result<Term<T>, Error>>
where
	L::Output: Into<C>,
{
	async move {
		let iri_ref = match value {
			Nullable::Null => return Ok(Term::Null),
			Nullable::Some(ExpandableRef::Keyword(k)) => return Ok(Term::Keyword(k)),
			Nullable::Some(ExpandableRef::IriRef(iri_ref)) => iri_ref.to_owned(),
			Nullable::Some(ExpandableRef::Key(key)) => {
				// If `local_context` is not null, it contains an entry with a key that equals value, and the
				// value of the entry for value in defined is not true, invoke the Create Term Definition
				// algorithm, passing active context, local context, value as term, and defined. This will
				// ensure that a term definition is created for value in active context during Context
				// Processing.
				super::define(
					active_context,
					local_context,
					Meta(key.into(), loc.clone()),
					defined,
					remote_contexts.clone(),
					loader,
					None,
					false,
					options.with_no_override(),
					&mut warnings,
				)
				.await?;

				let key = key.to_owned();
				if let Some(term_definition) = active_context.get(&key) {
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
						return match &term_definition.value {
							Some(value) => Ok(value.clone()),
							None => Ok(Term::Null)
						}
					}
				}

				match key {
					Key::Blank(blank_id) => {
						return Ok(Term::Ref(Reference::Blank(blank_id)));
					}
					Key::CompactIri(compact_iri) => {
						// If local context is not null, it contains a `prefix` entry, and the value of the
						// prefix entry in defined is not true, invoke the Create Term Definition
						// algorithm, passing active context, local context, prefix as term, and defined.
						// This will ensure that a term definition is created for prefix in active context
						// during Context Processing.
						super::define(
							active_context,
							local_context,
							Meta(KeyOrKeywordRef::Key(KeyRef::Term(compact_iri.prefix())), loc.clone()),
							defined,
							remote_contexts,
							loader,
							None,
							false,
							options.with_no_override(),
							&mut warnings,
						)
						.await?;

						// If active context contains a term definition for prefix having a non-null IRI
						// mapping and the prefix flag of the term definition is true, return the result
						// of concatenating the IRI mapping associated with prefix and suffix.
						let prefix_key = Key::Term(compact_iri.prefix().to_string());
						if let Some(term_definition) = active_context.get(&prefix_key) {
							if term_definition.prefix {
								if let Some(mapping) = &term_definition.value {
									let mut result = mapping.as_str().to_string();
									result.push_str(compact_iri.suffix());

									return Ok(Term::Ref(Reference::from_string(result)))
								}
							}
						}

						compact_iri.into_iri_ref()
					}
					Key::Iri(iri) => {
						return Ok(Term::Ref(Reference::Id(T::from_iri(iri.as_iri()))));
					}
					Key::Term(term) => {
						match IriRefBuf::from_string(term) {
							Ok(iri_ref) => iri_ref,
							Err((_, term)) => return Ok(Term::Ref(Reference::Invalid(term)))
						}
					}
				}
			}
		};

		// If value has the form of an IRI, return value.
		if let Ok(iri) = iri_ref.as_iri() {
			return Ok(Term::from(T::from_iri(iri)));
		}

		// If vocab is true, and active context has a vocabulary mapping, return the result of
		// concatenating the vocabulary mapping with value.
		if vocab {
			match active_context.vocabulary() {
				Some(Term::Ref(mapping)) => {
					let mut result = mapping.as_str().to_string();
					result.push_str(iri_ref.as_str());

					return Ok(Term::Ref(Reference::from_string(result)))
				}
				Some(_) => {
					return Ok(invalid_iri(Meta(iri_ref.to_string(), loc), warnings))
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
			if let Some(value) = super::resolve_iri(iri_ref.as_iri_ref(), active_context.base_iri()) {
				return Ok(Term::from(T::from_iri(value.as_iri())));
			} else {
				return Ok(invalid_iri(Meta(iri_ref.to_string(), loc), warnings));
			}
		}

		// Return value as is.
		Ok(invalid_iri(Meta(iri_ref.to_string(), loc), warnings))
	}
}

/// Default values for `document_relative` and `vocab` should be `false` and `true`.
pub fn expand_iri_simple<
	'a,
	T: Id,
	C: syntax::AnyContextEntry,
	W: From<MalformedIri>
>(
	active_context: &'a Context<T, C>,
	Meta(value, loc): Meta<Nullable<ExpandableRef<'a>>, C::Metadata>,
	document_relative: bool,
	vocab: bool,
	warnings: impl FnMut(Meta<W, C::Metadata>),
) -> Term<T> {
	let iri_ref = match value {
		Nullable::Null => return Term::Null,
		Nullable::Some(ExpandableRef::Keyword(k)) => return Term::Keyword(k),
		Nullable::Some(ExpandableRef::IriRef(iri_ref)) => iri_ref.to_owned(),
		Nullable::Some(ExpandableRef::Key(key)) => {
			let key = key.to_owned();
			if let Some(term_definition) = active_context.get(&key) {
				// If active context has a term definition for value, and the associated IRI mapping
				// is a keyword, return that keyword.
				if let Some(value) = &term_definition.value {
					if value.is_keyword() {
						return value.clone()
					}
				}

				// If vocab is true and the active context has a term definition for value, return the
				// associated IRI mapping.
				if vocab {
					return match &term_definition.value {
						Some(value) => value.clone(),
						None => Term::Null
					}
				}
			}

			match key {
				Key::Blank(blank_id) => {
					return Term::Ref(Reference::Blank(blank_id))
				}
				Key::CompactIri(compact_iri) => {
					// If active context contains a term definition for prefix having a non-null IRI
					// mapping and the prefix flag of the term definition is true, return the result
					// of concatenating the IRI mapping associated with prefix and suffix.
					let prefix_key = Key::Term(compact_iri.prefix().to_string());
					if let Some(term_definition) = active_context.get(&prefix_key) {
						if term_definition.prefix {
							if let Some(mapping) = &term_definition.value {
								let mut result = mapping.as_str().to_string();
								result.push_str(compact_iri.suffix());

								return Term::Ref(Reference::from_string(result))
							}
						}
					}

					compact_iri.into_iri_ref()
				}
				Key::Iri(iri) => {
					return Term::Ref(Reference::Id(T::from_iri(iri.as_iri())))
				}
				Key::Term(term) => {
					match IriRefBuf::from_string(term) {
						Ok(iri_ref) => iri_ref,
						Err((_, term)) => return Term::Ref(Reference::Invalid(term))
					}
				}
			}
		}
	};

	// If value has the form of an IRI, return value.
	if let Ok(iri) = iri_ref.as_iri() {
		return Term::from(T::from_iri(iri))
	}

	// If vocab is true, and active context has a vocabulary mapping, return the result of
	// concatenating the vocabulary mapping with value.
	if vocab {
		match active_context.vocabulary() {
			Some(Term::Ref(mapping)) => {
				let mut result = mapping.as_str().to_string();
				result.push_str(iri_ref.as_str());

				return Term::Ref(Reference::from_string(result))
			}
			Some(_) => {
				return invalid_iri(Meta(iri_ref.to_string(), loc), warnings)
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
		if let Some(value) = super::resolve_iri(iri_ref.as_iri_ref(), active_context.base_iri()) {
			return Term::from(T::from_iri(value.as_iri()))
		} else {
			return invalid_iri(Meta(iri_ref.to_string(), loc), warnings)
		}
	}

	// Return value as is.
	invalid_iri(Meta(iri_ref.to_string(), loc), warnings)
}

fn invalid_iri<T, M, W: From<MalformedIri>>(
	Meta(value, loc): Meta<String, M>,
	mut warnings: impl FnMut(Meta<W, M>)
) -> Term<T> {
	warnings(Meta(
		MalformedIri(value.clone()).into(),
		loc
	));

	Term::Ref(Reference::Invalid(value))
}