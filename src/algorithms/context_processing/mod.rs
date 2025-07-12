//! JSON-LD context processing types and algorithms.
use iref::Iri;

mod define;
mod iri;
mod merged;
mod options;
mod stack;

use define::*;
use iri::*;
use merged::Merged;
pub use options::ContextProcessingOptions;
use stack::ProcessingStack;

use crate::{
	algorithms::{error::Error, ProcessingEnvironment},
	syntax::{context::KeyOrKeywordRef, Context, ContextEntry, Keyword},
	ContextDocument, Loader, Nullable, ProcessedContext, ProcessingMode, Term,
};

struct TargetProcessedContext<'a> {
	pub value: &'a mut ProcessedContext,
	pub defined: DefinedTerms,
}

struct ContextProcessor<'a> {
	pub remote_contexts: ProcessingStack,
	pub active_context: &'a ProcessedContext,
	pub base_url: Option<&'a Iri>,
	pub options: ContextProcessingOptions,
}

impl<'a> ContextProcessor<'a> {
	// fn for_definition<'b>(
	// 	&'b self,
	// 	// defined: &'b mut DefinedTerms,
	// 	// local_context: &'b Merged<'b>,
	// ) -> ContextProcessor<'b> {
	// 	// TermDefiner {
	// 	// 	defined,
	// 	// 	local_context,
	// 	// 	base_url: self.base_url,
	// 	// 	remote_contexts: self.remote_contexts.clone(),
	// 	// 	options: self.options,
	// 	// }
	// 	ContextProcessor {
	// 		remote_contexts: self.remote_contexts.clone(),
	// 		active_context: self.active_context,
	// 		base_url: self.base_url,
	// 		options: self.options.with_no_override(),
	// 	}
	// }

	fn for_recursive_definition<'b>(&'b self) -> ContextProcessor<'b> {
		ContextProcessor {
			remote_contexts: self.remote_contexts.clone(),
			active_context: self.active_context,
			base_url: None,
			options: self.options.with_no_override(),
		}
	}

	fn with_override<'b>(&'b self) -> ContextProcessor<'b> {
		ContextProcessor {
			remote_contexts: self.remote_contexts.clone(),
			active_context: self.active_context,
			base_url: self.base_url,
			options: self.options.with_no_override(),
		}
	}

	fn for_sub_context<'b>(
		&'b self,
		active_context: &'b ProcessedContext,
		base_url: Option<&'b Iri>,
		options: ContextProcessingOptions,
	) -> ContextProcessor<'b> {
		ContextProcessor {
			remote_contexts: self.remote_contexts.clone(),
			active_context,
			base_url,
			options,
		}
	}
}

impl ContextDocument {
	/// Process this context with the default options.
	///
	/// See: <https://www.w3.org/TR/json-ld11-api/#context-processing-algorithm>
	pub async fn process(
		&self,
		env: impl ProcessingEnvironment,
	) -> Result<ProcessedContext, Error> {
		self.document.context.process(env, self.url()).await
	}
}

impl Context {
	/// Process this context with the default options.
	///
	/// See: <https://www.w3.org/TR/json-ld11-api/#context-processing-algorithm>
	pub async fn process(
		&self,
		env: impl ProcessingEnvironment,
		base_url: Option<&Iri>,
	) -> Result<ProcessedContext, Error> {
		let active_context = ProcessedContext::new(None);
		self.process_with(
			env,
			base_url,
			&active_context,
			ContextProcessingOptions::default(),
		)
		.await
	}

	/// Process this context with the given options.
	///
	/// See: <https://www.w3.org/TR/json-ld11-api/#context-processing-algorithm>
	pub async fn process_with(
		&self,
		mut env: impl ProcessingEnvironment,
		base_url: Option<&Iri>,
		active_context: &ProcessedContext,
		options: ContextProcessingOptions,
	) -> Result<ProcessedContext, Error> {
		ContextProcessor {
			options,
			remote_contexts: ProcessingStack::new(),
			base_url,
			active_context,
		}
		.process(&mut env, self)
		.await
	}
}

impl<'a> ContextProcessor<'a> {
	async fn process(
		mut self,
		env: &mut impl ProcessingEnvironment,
		local_context: &Context,
	) -> Result<ProcessedContext, Error> {
		// 1) Initialize result to the result of cloning active context.
		let mut result = self.active_context.clone();

		// 2) If `local_context` is an object containing the member @propagate,
		// its value MUST be boolean true or false, set `propagate` to that value.
		if let Context::One(ContextEntry::Definition(def)) = local_context {
			if let Some(propagate) = def.propagate {
				if self.options.processing_mode == ProcessingMode::JsonLd1_0 {
					return Err(Error::InvalidContextEntry);
				}

				self.options.propagate = propagate
			}
		}

		// 3) If propagate is false, and result does not have a previous context,
		// set previous context in result to active context.
		if !self.options.propagate && result.previous_context().is_none() {
			result.set_previous_context(self.active_context.clone());
		}

		// 4) If local context is not an array, set it to an array containing only local context.
		// 5) For each item context in local context:
		for context in local_context {
			match context {
				// 5.1) If context is null:
				ContextEntry::Null => {
					// If `override_protected` is false and `active_context` contains any protected term
					// definitions, an invalid context nullification has been detected and processing
					// is aborted.
					if !self.options.override_protected && result.has_protected_items() {
						return Err(Error::InvalidContextNullification);
					} else {
						// Otherwise, initialize result as a newly-initialized active context, setting
						// previous_context in result to the previous value of result if propagate is
						// false. Continue with the next context.
						let previous_result = result;

						// Initialize `result` as a newly-initialized active context, setting both
						// `base_iri` and `original_base_url` to the value of `original_base_url` in
						// active context, ...
						result = ProcessedContext::new(
							self.active_context
								.original_base_url()
								.map(ToOwned::to_owned),
						);

						// ... and, if `propagate` is `false`, `previous_context` in `result` to the
						// previous value of `result`.
						if !self.options.propagate {
							result.set_previous_context(previous_result);
						}
					}
				}

				// 5.2) If context is a string,
				ContextEntry::IriRef(iri_ref) => {
					// Initialize `context` to the result of resolving context against base URL.
					// If base URL is not a valid IRI, then context MUST be a valid IRI, otherwise
					// a loading document failed error has been detected and processing is aborted.
					let context_iri =
						resolve_iri(iri_ref, self.base_url).ok_or(Error::LoadingDocumentFailed)?;

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
					if self.remote_contexts.push(context_iri.clone()) {
						let loaded_context = env
							.loader_mut()
							.load(&context_iri)
							.await?
							.try_into_context_document()
							.map_err(Error::RemoteContextSyntax)?
							.into_document()
							.context;

						// Set result to the result of recursively calling this algorithm, passing result
						// for active context, loaded context for local context, the documentUrl of context
						// document for base URL, and a copy of remote contexts.
						let new_options = ContextProcessingOptions {
							processing_mode: self.options.processing_mode,
							override_protected: false,
							propagate: true,
						};

						result = Box::pin(
							self.for_sub_context(&result, Some(&context_iri), new_options)
								.process(env, &loaded_context),
						)
						.await?;
					}
				}

				// 5.4) Context definition.
				ContextEntry::Definition(context) => {
					// 5.5) If context has a @version entry:
					if context.version.is_some() {
						// 5.5.2) If processing mode is set to json-ld-1.0, a processing mode conflict
						// error has been detected.
						if self.options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(Error::ProcessingModeConflict);
						}
					}

					// 5.6) If context has an @import entry:
					let import_context = match &context.import {
						Some(import_value) => {
							// 5.6.1) If processing mode is json-ld-1.0, an invalid context entry error
							// has been detected.
							if self.options.processing_mode == ProcessingMode::JsonLd1_0 {
								return Err(Error::InvalidContextEntry);
							}

							// 5.6.3) Initialize import to the result of resolving the value of
							// @import.
							let import = resolve_iri(import_value, self.base_url)
								.ok_or(Error::InvalidImportValue)?;

							// 5.6.4) Dereference import.
							let import_context = env
								.loader_mut()
								.load(&import)
								.await?
								.try_into_context_document()
								.map_err(Error::RemoteContextSyntax)?
								.into_document()
								.context;

							// If the dereferenced document has no top-level map with an @context
							// entry, or if the value of @context is not a context definition
							// (i.e., it is not an map), an invalid remote context has been
							// detected and processing is aborted; otherwise, set import context
							// to the value of that entry.
							match &import_context {
								Context::One(ContextEntry::Definition(import_context_def)) => {
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
					if self.remote_contexts.is_empty() {
						// Initialize value to the value associated with the @base entry.
						if let Some(value) = context.base() {
							match value {
								Nullable::Null => {
									// If value is null, remove the base IRI of result.
									result.set_base_iri(None);
								}
								Nullable::Some(iri_ref) => match iri_ref.as_iri() {
									Some(iri) => result.set_base_iri(Some(iri.to_owned())),
									None => {
										let resolved = resolve_iri(iri_ref, result.base_iri())
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
							Nullable::Null => {
								// If value is null, remove any vocabulary mapping from result.
								result.set_vocabulary(None);
							}
							Nullable::Some(value) => {
								// Otherwise, if value is an IRI or blank node identifier, the
								// vocabulary mapping of result is set to the result of IRI
								// expanding value using true for document relative. If it is not
								// an IRI, or a blank node identifier, an invalid vocab mapping
								// error has been detected and processing is aborted.
								// NOTE: The use of blank node identifiers to value for @vocab is
								// obsolete, and may be removed in a future version of JSON-LD.
								match result.expand_iri_with(
									Nullable::Some(value.into()),
									true,
									true,
									|w| env.warn(w),
								) {
									Term::Id(vocab) => result.set_vocabulary(Some(Term::Id(vocab))),
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
						if self.options.processing_mode == ProcessingMode::JsonLd1_0 {
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
					let mut result = TargetProcessedContext {
						value: &mut result,
						defined: DefinedTerms::new(),
					};

					let protected = context.protected().unwrap_or(false);

					// 5.13) For each key-value pair in context where key is not
					// @base, @direction, @import, @language, @propagate, @protected, @version,
					// or @vocab,
					// invoke the Create Term Definition algorithm passing result for
					// active context, context for local context, key, defined, base URL,
					// and the value of the @protected entry from context, if any, for protected.
					// (and the value of override protected)
					if context.type_().is_some() {
						self.define(
							env,
							&mut result,
							&context,
							KeyOrKeywordRef::Keyword(Keyword::Type),
							protected,
						)
						.await?
					}

					for (key, _binding) in context.bindings() {
						self.define(env, &mut result, &context, key.into(), protected)
							.await?
					}
				}
			}
		}

		Ok(result)
	}
}

// /// Result of context processing functions.
// pub type ProcessingResult<'a, T, B> = Result<Processed<'a, T, B>, Error>;

// pub trait Process {
// 	/// Process the local context with specific options.
// 	#[allow(async_fn_in_trait)]
// 	async fn process_full<N, L, W>(
// 		&self,
// 		vocabulary: &mut N,
// 		active_context: &Context<N::Iri, N::BlankId>,
// 		loader: &L,
// 		base_url: Option<N::Iri>,
// 		options: Options,
// 		warnings: W,
// 	) -> Result<Processed<N::Iri, N::BlankId>, Error>
// 	where
// 		N: VocabularyMut,
// 		N::Iri: Clone + Eq + Hash,
// 		N::BlankId: Clone + PartialEq,
// 		L: Loader,
// 		W: WarningHandler<N>;

// 	/// Process the local context with specific options.
// 	#[allow(clippy::type_complexity)]
// 	#[allow(async_fn_in_trait)]
// 	async fn process_with<N, L>(
// 		&self,
// 		vocabulary: &mut N,
// 		active_context: &Context<N::Iri, N::BlankId>,
// 		loader: &L,
// 		base_url: Option<N::Iri>,
// 		options: Options,
// 	) -> Result<Processed<N::Iri, N::BlankId>, Error>
// 	where
// 		N: VocabularyMut,
// 		N::Iri: Clone + Eq + Hash,
// 		N::BlankId: Clone + PartialEq,
// 		L: Loader,
// 	{
// 		self.process_full(
// 			vocabulary,
// 			active_context,
// 			loader,
// 			base_url,
// 			options,
// 			warning::Print,
// 		)
// 		.await
// 	}

// 	/// Process the local context with the given initial active context with the default options:
// 	/// `is_remote` is `false`, `override_protected` is `false` and `propagate` is `true`.
// 	#[allow(async_fn_in_trait)]
// 	async fn process<N, L>(
// 		&self,
// 		vocabulary: &mut N,
// 		loader: &L,
// 		base_url: Option<N::Iri>,
// 	) -> Result<Processed<N::Iri, N::BlankId>, Error>
// 	where
// 		N: VocabularyMut,
// 		N::Iri: Clone + Eq + Hash,
// 		N::BlankId: Clone + PartialEq,
// 		L: Loader,
// 	{
// 		let active_context = Context::default();
// 		self.process_full(
// 			vocabulary,
// 			&active_context,
// 			loader,
// 			base_url,
// 			Options::default(),
// 			warning::Print,
// 		)
// 		.await
// 	}
// }
