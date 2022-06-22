use json_ld_core::{
	Id,
	Context,
	ProcessingMode
};
use json_ld_syntax as syntax;
use iref::{Iri, IriBuf, IriRef};
use futures::future::BoxFuture;
use locspan::Loc;
use crate::{
	Process,
	ProcessingStack,
	ProcessingOptions,
	ProcessingResult,
	Loader,
	Warning,
	LocWarning,
	Error,
	LocError
};

mod iri;
mod merged;

use iri::*;
use merged::*;

impl<C: syntax::AnyContextEntry + Send + Sync, T: Id> Process<T> for C {
	type Source = C::Source;
	type Span = C::Span;
	
	fn process_full<'a, L: Loader + Send + Sync>(
		&'a self,
		active_context: &'a Context<T, C>,
		stack: ProcessingStack,
		loader: &'a mut L,
		base_url: Option<Iri<'a>>,
		options: ProcessingOptions,
	) -> BoxFuture<'a, ProcessingResult<T, C>>
	where
		L::Output: Into<C>,
		T: Send + Sync
	{
		todo!()
	}
}

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

// This function tries to follow the recommended context processing algorithm.
// See `https://www.w3.org/TR/json-ld11-api/#context-processing-algorithm`.
//
// The recommended default value for `remote_contexts` is the empty set,
// `false` for `override_protected`, and `true` for `propagate`.
fn process_context<'a, T, C, L>(
	active_context: &'a Context<T, C>,
	local_context: &'a C,
	mut remote_contexts: ProcessingStack,
	loader: &'a mut L,
	base_url: Option<Iri>,
	mut options: ProcessingOptions,
	warnings: &'a mut Vec<LocWarning<T, C>>,
) -> BoxFuture<'a, Result<C, LocError<T, C>>>
where
	T: Id + Send + Sync,
	<C as Process<T>>::Source: Clone,
	<C as Process<T>>::Span: Clone,
	C: Clone + syntax::AnyContextEntry + Process<T, Source=<C as syntax::AnyContextEntry>::Source, Span=<C as syntax::AnyContextEntry>::Span>,
	L: Loader<Output=C> + Send + Sync
{
	use syntax::AnyContextDefinition;
	let base_url_buf = base_url.map(IriBuf::from);

	async move {
		let base_url = base_url_buf.as_ref().map(|base_url| base_url.as_iri());

		// 1) Initialize result to the result of cloning active context.
		let mut result = active_context.clone();

		// 2) If `local_context` is an object containing the member @propagate,
		// its value MUST be boolean true or false, set `propagate` to that value.
		let local_context_ref = local_context.as_entry_ref();
		if let syntax::ContextEntryRef::One(Loc(syntax::ContextRef::Definition(def), _)) = local_context_ref {
			if let Some(propagate) = def.propagate() {
				options.propagate = *propagate.value()
			}
		}

		// 3) If propagate is false, and result does not have a previous context,
		// set previous context in result to active context.
		if !options.propagate && result.previous_context().is_none() {
			result.set_previous_context(active_context.clone());
		}

		// 4) If local context is not an array, set it to an array containing only local context.
		// 5) For each item context in local context:
		for Loc(context, context_loc) in local_context_ref {
			match context {
				// 5.1) If context is null:
				syntax::ContextRef::Null => {
					// If `override_protected` is false and `active_context` contains any protected term
					// definitions, an invalid context nullification has been detected and processing
					// is aborted.
					if !options.override_protected && result.has_protected_items() {
						let e: LocError<T, C> = Error::InvalidContextNullification
							.located(context_loc);
						return Err(e);
					} else {
						// Otherwise, initialize result as a newly-initialized active context, setting
						// previous_context in result to the previous value of result if propagate is
						// false. Continue with the next context.
						let previous_result = result;

						// Initialize `result` as a newly-initialized active context, setting both
						// `base_iri` and `original_base_url` to the value of `original_base_url` in
						// active context, ...
						result = Context::new(active_context.original_base_url());

						// ... and, if `propagate` is `false`, `previous_context` in `result` to the
						// previous value of `result`.
						if !options.propagate {
							result.set_previous_context(previous_result);
						}
					}
				}

				// 5.2) If context is a string,
				syntax::ContextRef::IriRef(iri_ref) => {
					// Initialize `context` to the result of resolving context against base URL.
					// If base URL is not a valid IRI, then context MUST be a valid IRI, otherwise
					// a loading document failed error has been detected and processing is aborted.
					let context_iri = resolve_iri(iri_ref, base_url).ok_or_else(|| {
						Error::LoadingDocumentFailed
							.located(context_loc)
					})?;

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
					if remote_contexts.push(context_iri.as_iri()) {
						let loaded_context = loader
							.load_context(context_iri.as_iri())
							.await
							.map_err(|e| e.located(context_loc))?;

						// Set result to the result of recursively calling this algorithm, passing result
						// for active context, loaded context for local context, the documentUrl of context
						// document for base URL, and a copy of remote contexts.
						let new_options = ProcessingOptions {
							processing_mode: options.processing_mode,
							override_protected: false,
							propagate: true,
						};

						let (processed, processed_warnings) = loaded_context
							.process_full(
								&result,
								remote_contexts.clone(),
								loader,
								Some(context_iri.as_iri()),
								new_options,
							)
							.await?
							.into_parts();

						warnings.extend(processed_warnings);
						result = processed
					}
				}

				// 5.4) Context definition.
				syntax::ContextRef::Definition(context) => {
					// 5.5) If context has a @version entry:
					if let Some(version_value) = context.version() {
						// 5.5.2) If processing mode is set to json-ld-1.0, a processing mode conflict
						// error has been detected.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(Error::ProcessingModeConflict
								.located(version_value.location().clone().cast()));
						}
					}

					// 5.6) If context has an @import entry:
					let context: Merged<'a, C::Definition> = if let Some(Loc(import_value, import_loc)) = context.import() {
						// 5.6.1) If processing mode is json-ld-1.0, an invalid context entry error
						// has been detected.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(Error::InvalidContextEntry
								.located(import_loc));
						}

						// 5.6.3) Initialize import to the result of resolving the value of
						// @import.
						let import = resolve_iri(import_value, base_url).ok_or_else(|| {
							Error::InvalidImportValue
								.located(import_loc)
						})?;

						// 5.6.4) Dereference import.
						let import_context = loader
							.load_context(import.as_iri())
							.await
							.map_err(|e| e.located(import_loc))?;

						// If the dereferenced document has no top-level map with an @context
						// entry, or if the value of @context is not a context definition
						// (i.e., it is not an map), an invalid remote context has been
						// detected and processing is aborted; otherwise, set import context
						// to the value of that entry.
						match import_context.as_entry_ref() {
							syntax::ContextEntryRef::One(Loc(syntax::ContextRef::Definition(import_context), _)) => {
								// If `import_context` has a @import entry, an invalid context entry
								// error has been detected and processing is aborted.
								if let Some(Loc(_, loc)) = import_context.import() {
									return Err(Error::InvalidContextEntry.located(loc));
								}

								// Set `context` to the result of merging context into
								// `import_context`, replacing common entries with those from
								// `context`.
								Merged::new(context, Some(import_context))
							}
							_ => {
								return Err(Error::InvalidRemoteContext
									.located(import_loc));
							}
						}
					} else {
						Merged::new(context, None)
					};

					// 5.7) If context has a @base entry and remote contexts is empty, i.e.,
					// the currently being processed context is not a remote context:
					if remote_contexts.is_empty() {
						// Initialize value to the value associated with the @base entry.
						if let Some(Loc(value, base_loc)) = context.base() {
							match value {
								syntax::Nullable::Null => {
									// If value is null, remove the base IRI of result.
									result.set_base_iri(None);
								}
								syntax::Nullable::Some(iri_ref) => {
									match iri_ref.into_iri() {
										Ok(iri) => result.set_base_iri(Some(iri)),
										Err(not_iri) => {
											let resolved =
												resolve_iri(not_iri, result.base_iri())
													.ok_or_else(|| {
													Error::InvalidBaseIri.located(base_loc)
												})?;
											result.set_base_iri(Some(resolved.as_iri()))
										}
									}
								}
							}
						}
					}

					// 5.8) If context has a @vocab entry:
					// Initialize value to the value associated with the @vocab entry.
					if let Some(Loc(value, vocab_loc)) = context.vocab() {
						match value {
							syntax::Nullable::Null => {
								// If value is null, remove any vocabulary mapping from result.
								result.set_vocabulary(None);
							}
							syntax::Nullable::Some(value) => {
								let string_value = value.as_ref();
								// Otherwise, if value is an IRI or blank node identifier, the
								// vocabulary mapping of result is set to the result of IRI
								// expanding value using true for document relative. If it is not
								// an IRI, or a blank node identifier, an invalid vocab mapping
								// error has been detected and processing is aborted.
								// NOTE: The use of blank node identifiers to value for @vocab is
								// obsolete, and may be removed in a future version of JSON-LD.
								match expand_iri(
									&result,
									string_value,
									true,
									true,
									warnings,
								) {
									Term::Ref(vocab) => {
										result.set_vocabulary(Some(Term::Ref(vocab)))
									}
									_ => {
										return Err(Error::InvalidVocabMapping
											.located(vocab_loc))
									}
								}
							}
							_ => {
								return Err(Error::InvalidVocabMapping
									.located(vocab_loc))
							}
						}
					}

					// 5.9) If context has a @language entry:
					if let Some(value) = context.get(Keyword::Language.into()) {
						if value.is_null() {
							// 5.9.2) If value is null, remove any default language from result.
							result.set_default_language(None);
						} else if let Some(str_value) = value.as_str() {
							// 5.9.3) Otherwise, if value is string, the default language of result is
							// set to value.
							match LanguageTagBuf::parse_copy(str_value) {
								Ok(lang) => result.set_default_language(Some(lang.into())),
								Err(err) => {
									// If value is not well-formed according to section 2.2.9 of [BCP47],
									// processors SHOULD issue a warning.
									warnings.push(Loc::new(
										Warning::MalformedLanguageTag(str_value.to_string(), err),
										source,
										value.metadata().clone(),
									));
									result.set_default_language(Some(str_value.to_string().into()));
								}
							}
						} else {
							return Err(ErrorCode::InvalidDefaultLanguage
								.located(source, value.metadata().clone()));
						}
					}

					// 5.10) If context has a @direction entry:
					if let Some((direction_key, value)) =
						context.get_key_value(Keyword::Direction.into())
					{
						// 5.10.1) If processing mode is json-ld-1.0, an invalid context entry error
						// has been detected and processing is aborted.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(ErrorCode::InvalidContextEntry
								.located(source, direction_key.metadata().clone()));
						}

						if value.is_null() {
							// 5.10.3) If value is null, remove any base direction from result.
							result.set_default_base_direction(None);
						} else if let Some(str) = value.as_str() {
							let dir = match str {
								"ltr" => Direction::Ltr,
								"rtl" => Direction::Rtl,
								_ => {
									return Err(ErrorCode::InvalidBaseDirection
										.located(source, value.metadata().clone()))
								}
							};
							result.set_default_base_direction(Some(dir));
						} else {
							return Err(ErrorCode::InvalidBaseDirection
								.located(source, value.metadata().clone()));
						}
					}

					// 5.12) Create a map `defined` to keep track of whether or not a term
					// has already been defined or is currently being defined during recursion.
					let mut defined = HashMap::new();
					let protected = context
						.get(Keyword::Protected.into())
						.and_then(|p| p.as_bool())
						.unwrap_or(false);

					// 5.13) For each key-value pair in context where key is not
					// @base, @direction, @import, @language, @propagate, @protected, @version,
					// or @vocab,
					// invoke the Create Term Definition algorithm passing result for
					// active context, context for local context, key, defined, base URL,
					// and the value of the @protected entry from context, if any, for protected.
					// (and the value of override protected)
					for (key, _) in context.iter() {
						let key_metadata = key.metadata();
						let key: &str = &**key;
						match key {
							"@base" | "@direction" | "@import" | "@language" | "@propagate"
							| "@protected" | "@version" | "@vocab" => (),
							_ => define(
								&mut result,
								&context,
								key,
								key_metadata,
								&mut defined,
								remote_contexts.clone(),
								loader,
								base_url,
								protected,
								options,
								warnings,
							)
							.await
							.map_err(|e| e.located(source, key_metadata.clone()))?,
						}
					}
				}
			}
		}

		Ok(result)
	}
	.boxed()
}