use crate::{
	ContextLoader, Error, MetaError, Options, ProcessMeta, Processed, ProcessingResult,
	ProcessingStack, WarningHandler,
};
use futures::future::{BoxFuture, FutureExt};
use iref::IriRef;
use json_ld_core::{Context, ProcessingMode, Term};
use json_ld_syntax::{self as syntax, Entry, Nullable};
use locspan::{At, Meta};
use rdf_types::{IriVocabularyMut, VocabularyMut};

mod define;
mod iri;
mod merged;

pub use define::*;
pub use iri::*;
pub use merged::*;
use syntax::context::definition::KeyOrKeywordRef;

impl<T, B, M> ProcessMeta<T, B, M> for syntax::context::Value<M> {
	fn process_meta<'l: 'a, 'a, N, L: ContextLoader<T, M> + Send + Sync>(
		&'l self,
		meta: &'l M,
		vocabulary: &'a mut N,
		active_context: &'a Context<T, B, M>,
		stack: ProcessingStack<T>,
		loader: &'a mut L,
		base_url: Option<T>,
		options: Options,
		warnings: impl 'a + Send + WarningHandler<N, M>,
	) -> BoxFuture<'a, ProcessingResult<'l, T, B, M, L::ContextError>>
	where
		N: Send + Sync + VocabularyMut<Iri = T, BlankId = B>,
		T: Clone + PartialEq + Send + Sync,
		B: Clone + PartialEq + Send + Sync,
		M: 'a + Clone + Send + Sync,
	{
		async move {
			Ok(process_context(
				vocabulary,
				active_context,
				Meta(self, meta),
				stack,
				loader,
				base_url,
				options,
				warnings,
			)
			.await?
			.0)
		}
		.boxed()
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

type ContextProcessingResult<'a, T, B, M, L, W> =
	Result<(Processed<'a, T, B, M>, W), MetaError<M, <L as ContextLoader<T, M>>::ContextError>>;

// This function tries to follow the recommended context processing algorithm.
// See `https://www.w3.org/TR/json-ld11-api/#context-processing-algorithm`.
//
// The recommended default value for `remote_contexts` is the empty set,
// `false` for `override_protected`, and `true` for `propagate`.
fn process_context<'l: 'a, 'a, T, B, M, N, L, W>(
	vocabulary: &'a mut N,
	active_context: &'a Context<T, B, M>,
	Meta(local_context, meta): Meta<&'l syntax::context::Value<M>, &'l M>,
	mut remote_contexts: ProcessingStack<T>,
	loader: &'a mut L,
	base_url: Option<T>,
	mut options: Options,
	mut warnings: W,
) -> BoxFuture<'a, ContextProcessingResult<'l, T, B, M, L, W>>
where
	T: 'a + Clone + PartialEq + Send + Sync,
	B: 'a + Clone + PartialEq + Send + Sync,
	M: 'a + Clone + Send + Sync,
	N: Send + Sync + VocabularyMut<Iri = T, BlankId = B>,
	L: ContextLoader<T, M> + Send + Sync,
	W: 'a + Send + WarningHandler<N, M>,
{
	async move {
		// 1) Initialize result to the result of cloning active context.
		let mut result = active_context.clone();

		// 2) If `local_context` is an object containing the member @propagate,
		// its value MUST be boolean true or false, set `propagate` to that value.
		if let syntax::context::Value::One(Meta(syntax::Context::Definition(def), _)) =
			local_context
		{
			if let Some(propagate) = &def.propagate {
				if options.processing_mode == ProcessingMode::JsonLd1_0 {
					return Err(Error::InvalidContextEntry.at(propagate.key_metadata.clone()));
				}

				options.propagate = *propagate.value.value()
			}
		}

		// 3) If propagate is false, and result does not have a previous context,
		// set previous context in result to active context.
		if !options.propagate && result.previous_context().is_none() {
			result.set_previous_context(active_context.clone());
		}

		// 4) If local context is not an array, set it to an array containing only local context.
		// 5) For each item context in local context:
		for Meta(context, context_meta) in local_context {
			match context {
				// 5.1) If context is null:
				syntax::Context::Null => {
					// If `override_protected` is false and `active_context` contains any protected term
					// definitions, an invalid context nullification has been detected and processing
					// is aborted.
					if !options.override_protected && result.has_protected_items() {
						let e: MetaError<M, L::ContextError> =
							Error::InvalidContextNullification.at(context_meta.clone());
						return Err(e);
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
				syntax::Context::IriRef(iri_ref) => {
					// Initialize `context` to the result of resolving context against base URL.
					// If base URL is not a valid IRI, then context MUST be a valid IRI, otherwise
					// a loading document failed error has been detected and processing is aborted.
					let context_iri =
						resolve_iri(vocabulary, iri_ref.as_iri_ref(), base_url.as_ref())
							.ok_or_else(|| Error::LoadingDocumentFailed.at(context_meta.clone()))?;

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
						let loaded_context = loader
							.load_context_with(vocabulary, context_iri.clone())
							.await
							.map_err(|e| Error::ContextLoadingFailed(e).at(context_meta.clone()))?
							.into_document();

						// Set result to the result of recursively calling this algorithm, passing result
						// for active context, loaded context for local context, the documentUrl of context
						// document for base URL, and a copy of remote contexts.
						let new_options = Options {
							processing_mode: options.processing_mode,
							override_protected: false,
							propagate: true,
						};

						let (r, w) = process_context(
							vocabulary,
							&result,
							loaded_context.borrow(),
							remote_contexts.clone(),
							loader,
							Some(context_iri),
							new_options,
							warnings,
						)
						.await?;

						result = r.into_processed();
						warnings = w;
					}
				}

				// 5.4) Context definition.
				syntax::Context::Definition(context) => {
					// 5.5) If context has a @version entry:
					if let Some(version_value) = &context.version {
						// 5.5.2) If processing mode is set to json-ld-1.0, a processing mode conflict
						// error has been detected.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(Error::ProcessingModeConflict
								.at(version_value.value.metadata().clone()));
						}
					}

					// 5.6) If context has an @import entry:
					let context: Merged<'a, M> = if let Some(Entry {
						value: Meta(import_value, import_meta),
						..
					}) = &context.import
					{
						// 5.6.1) If processing mode is json-ld-1.0, an invalid context entry error
						// has been detected.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(Error::InvalidContextEntry.at(import_meta.clone()));
						}

						// 5.6.3) Initialize import to the result of resolving the value of
						// @import.
						let import =
							resolve_iri(vocabulary, import_value.as_iri_ref(), base_url.as_ref())
								.ok_or_else(|| Error::InvalidImportValue.at(import_meta.clone()))?;

						// 5.6.4) Dereference import.
						let import_context: syntax::context::Value<M> = loader
							.load_context_with(vocabulary, import)
							.await
							.map_err(|e| Error::ContextLoadingFailed(e).at(import_meta.clone()))?
							.into_document()
							.into_value();

						// If the dereferenced document has no top-level map with an @context
						// entry, or if the value of @context is not a context definition
						// (i.e., it is not an map), an invalid remote context has been
						// detected and processing is aborted; otherwise, set import context
						// to the value of that entry.
						match &import_context {
							syntax::context::Value::One(Meta(
								syntax::Context::Definition(import_context_def),
								_,
							)) => {
								// If `import_context` has a @import entry, an invalid context entry
								// error has been detected and processing is aborted.
								if let Some(Entry {
									value: Meta(_, loc),
									..
								}) = &import_context_def.import
								{
									return Err(Error::InvalidContextEntry.at(loc.clone()));
								}
							}
							_ => {
								return Err(Error::InvalidRemoteContext.at(import_meta.clone()));
							}
						}

						// Set `context` to the result of merging context into
						// `import_context`, replacing common entries with those from
						// `context`.
						Merged::new(context, Some(import_context))
					} else {
						Merged::new(context, None)
					};

					// 5.7) If context has a @base entry and remote contexts is empty, i.e.,
					// the currently being processed context is not a remote context:
					if remote_contexts.is_empty() {
						// Initialize value to the value associated with the @base entry.
						if let Some(Entry {
							value: Meta(value, base_meta),
							..
						}) = context.base()
						{
							match value {
								syntax::Nullable::Null => {
									// If value is null, remove the base IRI of result.
									result.set_base_iri(None);
								}
								syntax::Nullable::Some(iri_ref) => match iri_ref.as_iri() {
									Some(iri) => result.set_base_iri(Some(vocabulary.insert(iri))),
									None => {
										let resolved = resolve_iri(
											vocabulary,
											iri_ref.as_iri_ref(),
											result.base_iri(),
										)
										.ok_or_else(|| {
											Error::InvalidBaseIri.at(base_meta.clone())
										})?;
										result.set_base_iri(Some(resolved))
									}
								},
							}
						}
					}

					// 5.8) If context has a @vocab entry:
					// Initialize value to the value associated with the @vocab entry.
					if let Some(Entry {
						value: Meta(value, vocab_meta),
						..
					}) = context.vocab()
					{
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
									vocabulary,
									&result,
									Meta(Nullable::Some(value.into()), vocab_meta),
									true,
									true,
									&mut warnings,
								) {
									Meta(Term::Id(vocab), _) => {
										result.set_vocabulary(Some(Term::Id(vocab)))
									}
									_ => {
										return Err(
											Error::InvalidVocabMapping.at(vocab_meta.clone())
										)
									}
								}
							}
						}
					}

					// 5.9) If context has a @language entry:
					if let Some(Entry {
						value: Meta(value, _language_meta),
						..
					}) = context.language()
					{
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
					if let Some(Entry {
						value: Meta(value, direction_meta),
						..
					}) = context.direction()
					{
						// 5.10.1) If processing mode is json-ld-1.0, an invalid context entry error
						// has been detected and processing is aborted.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(Error::InvalidContextEntry.at(direction_meta.clone()));
						}

						match value {
							Nullable::Null => {
								// 5.10.3) If value is null, remove any base direction from result.
								result.set_default_base_direction(None);
							}
							Nullable::Some(dir) => {
								result.set_default_base_direction(Some(*dir));
							}
						}
					}

					// 5.12) Create a map `defined` to keep track of whether or not a term
					// has already been defined or is currently being defined during recursion.
					let mut defined = DefinedTerms::new();
					let protected = context
						.protected()
						.map(Entry::as_value)
						.map(Meta::value)
						.copied()
						.unwrap_or(false);

					// 5.13) For each key-value pair in context where key is not
					// @base, @direction, @import, @language, @propagate, @protected, @version,
					// or @vocab,
					// invoke the Create Term Definition algorithm passing result for
					// active context, context for local context, key, defined, base URL,
					// and the value of the @protected entry from context, if any, for protected.
					// (and the value of override protected)
					if let Some(ty) = context.type_() {
						warnings = define(
							vocabulary,
							&mut result,
							&context,
							Meta(
								KeyOrKeywordRef::Keyword(syntax::Keyword::Type),
								&ty.key_metadata,
							),
							&mut defined,
							remote_contexts.clone(),
							loader,
							base_url.clone(),
							protected,
							options,
							warnings,
						)
						.await
						.map_err(|e| e.at(ty.key_metadata.clone()))?
					}

					for (key, binding) in context.bindings() {
						warnings = define(
							vocabulary,
							&mut result,
							&context,
							Meta(key.into(), &binding.key_metadata),
							&mut defined,
							remote_contexts.clone(),
							loader,
							base_url.clone(),
							protected,
							options,
							warnings,
						)
						.await
						.map_err(|e| e.at(binding.key_metadata.clone()))?
					}
				}
			}
		}

		Ok((Processed::new(Meta(local_context, meta), result), warnings))
	}
	.boxed()
}
