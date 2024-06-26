use std::hash::Hash;

use crate::{
	Error, Options, Process, Processed, ProcessingResult, ProcessingStack, WarningHandler,
};
use iref::IriRef;
use json_ld_core::{Context, Environment, ExtractContext, Loader, ProcessingMode, Term};
use json_ld_syntax::{self as syntax, Nullable};
use rdf_types::{vocabulary::IriVocabularyMut, VocabularyMut};

mod define;
mod iri;
mod merged;

pub use define::*;
pub use iri::*;
pub use merged::*;
use syntax::context::definition::KeyOrKeywordRef;

impl Process for syntax::context::Context {
	async fn process_full<N, L, W>(
		&self,
		vocabulary: &mut N,
		active_context: &Context<N::Iri, N::BlankId>,
		loader: &L,
		base_url: Option<N::Iri>,
		options: Options,
		mut warnings: W,
	) -> Result<Processed<N::Iri, N::BlankId>, Error>
	where
		N: VocabularyMut,
		N::Iri: Clone + Eq + Hash,
		N::BlankId: Clone + PartialEq,
		L: Loader,
		W: WarningHandler<N>,
	{
		process_context(
			Environment {
				vocabulary,
				loader,
				warnings: &mut warnings,
			},
			active_context,
			self,
			ProcessingStack::default(),
			base_url,
			options,
		)
		.await
	}
}

/// Resolve `iri_ref` against the given base IRI.
fn resolve_iri<I>(
	vocabulary: &mut impl IriVocabularyMut<Iri = I>,
	iri_ref: &IriRef,
	base_iri: Option<&I>,
) -> Option<I> {
	match base_iri {
		Some(base_iri) => {
			let result = iri_ref.resolved(vocabulary.iri(base_iri).unwrap());
			Some(vocabulary.insert(result.as_iri()))
		}
		None => iri_ref.as_iri().map(|iri| vocabulary.insert(iri)),
	}
}

// This function tries to follow the recommended context processing algorithm.
// See `https://www.w3.org/TR/json-ld11-api/#context-processing-algorithm`.
//
// The recommended default value for `remote_contexts` is the empty set,
// `false` for `override_protected`, and `true` for `propagate`.
async fn process_context<'l: 'a, 'a, N, L, W>(
	mut env: Environment<'a, N, L, W>,
	active_context: &'a Context<N::Iri, N::BlankId>,
	local_context: &'l syntax::context::Context,
	mut remote_contexts: ProcessingStack<N::Iri>,
	base_url: Option<N::Iri>,
	mut options: Options,
) -> ProcessingResult<'l, N::Iri, N::BlankId>
where
	N: VocabularyMut,
	N::Iri: Clone + Eq + Hash,
	N::BlankId: Clone + PartialEq,
	L: Loader,
	W: WarningHandler<N>,
{
	// 1) Initialize result to the result of cloning active context.
	let mut result = active_context.clone();

	// 2) If `local_context` is an object containing the member @propagate,
	// its value MUST be boolean true or false, set `propagate` to that value.
	if let syntax::context::Context::One(syntax::ContextEntry::Definition(def)) = local_context {
		if let Some(propagate) = def.propagate {
			if options.processing_mode == ProcessingMode::JsonLd1_0 {
				return Err(Error::InvalidContextEntry);
			}

			options.propagate = propagate
		}
	}

	// 3) If propagate is false, and result does not have a previous context,
	// set previous context in result to active context.
	if !options.propagate && result.previous_context().is_none() {
		result.set_previous_context(active_context.clone());
	}

	// 4) If local context is not an array, set it to an array containing only local context.
	// 5) For each item context in local context:
	for context in local_context {
		match context {
			// 5.1) If context is null:
			syntax::ContextEntry::Null => {
				// If `override_protected` is false and `active_context` contains any protected term
				// definitions, an invalid context nullification has been detected and processing
				// is aborted.
				if !options.override_protected && result.has_protected_items() {
					return Err(Error::InvalidContextNullification);
				} else {
					// Otherwise, initialize result as a newly-initialized active context, setting
					// previous_context in result to the previous value of result if propagate is
					// false. Continue with the next context.
					let previous_result = result;

					// Initialize `result` as a newly-initialized active context, setting both
					// `base_iri` and `original_base_url` to the value of `original_base_url` in
					// active context, ...
					result = Context::new(active_context.original_base_url().cloned());

					// ... and, if `propagate` is `false`, `previous_context` in `result` to the
					// previous value of `result`.
					if !options.propagate {
						result.set_previous_context(previous_result);
					}
				}
			}

			// 5.2) If context is a string,
			syntax::ContextEntry::IriRef(iri_ref) => {
				// Initialize `context` to the result of resolving context against base URL.
				// If base URL is not a valid IRI, then context MUST be a valid IRI, otherwise
				// a loading document failed error has been detected and processing is aborted.
				let context_iri =
					resolve_iri(env.vocabulary, iri_ref.as_iri_ref(), base_url.as_ref())
						.ok_or(Error::LoadingDocumentFailed)?;

				// If the number of entries in the `remote_contexts` array exceeds a processor
				// defined limit, a context overflow error has been detected and processing is
				// aborted; otherwise, add context to remote contexts.
				//
				// If context was previously dereferenced, then the processor MUST NOT do a further
				// dereference, and context is set to the previously established internal
				// representation: set `context_document` to the previously dereferenced document,
				// and set loaded context to the value of the @context entry from the document in
				// context document.
				//
				// Otherwise, set `context document` to the RemoteDocument obtained by dereferencing
				// context using the LoadDocumentCallback, passing context for url, and
				// http://www.w3.org/ns/json-ld#context for profile and for requestProfile.
				//
				// If context cannot be dereferenced, or the document from context document cannot
				// be transformed into the internal representation , a loading remote context
				// failed error has been detected and processing is aborted.
				// If the document has no top-level map with an @context entry, an invalid remote
				// context has been detected and processing is aborted.
				// Set loaded context to the value of that entry.
				if remote_contexts.push(context_iri.clone()) {
					let loaded_context = env
						.loader
						.load_with(env.vocabulary, context_iri.clone())
						.await?
						.into_document()
						.into_ld_context()
						.map_err(Error::ContextExtractionFailed)?;

					// Set result to the result of recursively calling this algorithm, passing result
					// for active context, loaded context for local context, the documentUrl of context
					// document for base URL, and a copy of remote contexts.
					let new_options = Options {
						processing_mode: options.processing_mode,
						override_protected: false,
						propagate: true,
						vocab: options.vocab,
					};

					let r = Box::pin(process_context(
						Environment {
							vocabulary: env.vocabulary,
							loader: env.loader,
							warnings: env.warnings,
						},
						&result,
						&loaded_context,
						remote_contexts.clone(),
						Some(context_iri),
						new_options,
					))
					.await?;

					result = r.into_processed();
				}
			}

			// 5.4) Context definition.
			syntax::ContextEntry::Definition(context) => {
				// 5.5) If context has a @version entry:
				if context.version.is_some() {
					// 5.5.2) If processing mode is set to json-ld-1.0, a processing mode conflict
					// error has been detected.
					if options.processing_mode == ProcessingMode::JsonLd1_0 {
						return Err(Error::ProcessingModeConflict);
					}
				}

				// 5.6) If context has an @import entry:
				let import_context = match &context.import {
					Some(import_value) => {
						// 5.6.1) If processing mode is json-ld-1.0, an invalid context entry error
						// has been detected.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(Error::InvalidContextEntry);
						}

						// 5.6.3) Initialize import to the result of resolving the value of
						// @import.
						let import = resolve_iri(
							env.vocabulary,
							import_value.as_iri_ref(),
							base_url.as_ref(),
						)
						.ok_or(Error::InvalidImportValue)?;

						// 5.6.4) Dereference import.
						let import_context = env
							.loader
							.load_with(env.vocabulary, import.clone())
							.await?
							.into_document()
							.into_ld_context()
							.map_err(Error::ContextExtractionFailed)?;

						// If the dereferenced document has no top-level map with an @context
						// entry, or if the value of @context is not a context definition
						// (i.e., it is not an map), an invalid remote context has been
						// detected and processing is aborted; otherwise, set import context
						// to the value of that entry.
						match &import_context {
							syntax::context::Context::One(syntax::ContextEntry::Definition(
								import_context_def,
							)) => {
								// If `import_context` has a @import entry, an invalid context entry
								// error has been detected and processing is aborted.
								if import_context_def.import.is_some() {
									return Err(Error::InvalidContextEntry);
								}
							}
							_ => {
								return Err(Error::InvalidRemoteContext);
							}
						}

						// Set `context` to the result of merging context into
						// `import_context`, replacing common entries with those from
						// `context`.
						Some(import_context)
					}
					None => None,
				};

				let context = Merged::new(context, import_context);

				// 5.7) If context has a @base entry and remote contexts is empty, i.e.,
				// the currently being processed context is not a remote context:
				if remote_contexts.is_empty() {
					// Initialize value to the value associated with the @base entry.
					if let Some(value) = context.base() {
						match value {
							syntax::Nullable::Null => {
								// If value is null, remove the base IRI of result.
								result.set_base_iri(None);
							}
							syntax::Nullable::Some(iri_ref) => match iri_ref.as_iri() {
								Some(iri) => result.set_base_iri(Some(env.vocabulary.insert(iri))),
								None => {
									let resolved =
										resolve_iri(env.vocabulary, iri_ref, result.base_iri())
											.ok_or(Error::InvalidBaseIri)?;
									result.set_base_iri(Some(resolved))
								}
							},
						}
					}
				}

				// 5.8) If context has a @vocab entry:
				// Initialize value to the value associated with the @vocab entry.
				if let Some(value) = context.vocab() {
					match value {
						syntax::Nullable::Null => {
							// If value is null, remove any vocabulary mapping from result.
							result.set_vocabulary(None);
						}
						syntax::Nullable::Some(value) => {
							// Otherwise, if value is an IRI or blank node identifier, the
							// vocabulary mapping of result is set to the result of IRI
							// expanding value using true for document relative. If it is not
							// an IRI, or a blank node identifier, an invalid vocab mapping
							// error has been detected and processing is aborted.
							// NOTE: The use of blank node identifiers to value for @vocab is
							// obsolete, and may be removed in a future version of JSON-LD.
							match expand_iri_simple(
								&mut env,
								&result,
								Nullable::Some(value.into()),
								true,
								Some(options.vocab),
							)? {
								Some(Term::Id(vocab)) => {
									result.set_vocabulary(Some(Term::Id(vocab)))
								}
								_ => return Err(Error::InvalidVocabMapping),
							}
						}
					}
				}

				// 5.9) If context has a @language entry:
				if let Some(value) = context.language() {
					match value {
						Nullable::Null => {
							// 5.9.2) If value is null, remove any default language from result.
							result.set_default_language(None);
						}
						Nullable::Some(tag) => {
							result.set_default_language(Some(tag.to_owned()));
						}
					}
				}

				// 5.10) If context has a @direction entry:
				if let Some(value) = context.direction() {
					// 5.10.1) If processing mode is json-ld-1.0, an invalid context entry error
					// has been detected and processing is aborted.
					if options.processing_mode == ProcessingMode::JsonLd1_0 {
						return Err(Error::InvalidContextEntry);
					}

					match value {
						Nullable::Null => {
							// 5.10.3) If value is null, remove any base direction from result.
							result.set_default_base_direction(None);
						}
						Nullable::Some(dir) => {
							result.set_default_base_direction(Some(dir));
						}
					}
				}

				// 5.12) Create a map `defined` to keep track of whether or not a term
				// has already been defined or is currently being defined during recursion.
				let mut defined = DefinedTerms::new();
				let protected = context.protected().unwrap_or(false);

				// 5.13) For each key-value pair in context where key is not
				// @base, @direction, @import, @language, @propagate, @protected, @version,
				// or @vocab,
				// invoke the Create Term Definition algorithm passing result for
				// active context, context for local context, key, defined, base URL,
				// and the value of the @protected entry from context, if any, for protected.
				// (and the value of override protected)
				if context.type_().is_some() {
					define(
						Environment {
							vocabulary: env.vocabulary,
							loader: env.loader,
							warnings: env.warnings,
						},
						&mut result,
						&context,
						KeyOrKeywordRef::Keyword(syntax::Keyword::Type),
						&mut defined,
						remote_contexts.clone(),
						base_url.clone(),
						protected,
						options,
					)
					.await?
				}

				for (key, _binding) in context.bindings() {
					define(
						Environment {
							vocabulary: env.vocabulary,
							loader: env.loader,
							warnings: env.warnings,
						},
						&mut result,
						&context,
						key.into(),
						&mut defined,
						remote_contexts.clone(),
						base_url.clone(),
						protected,
						options,
					)
					.await?
				}
			}
		}
	}

	Ok(Processed::new(local_context, result))
}
