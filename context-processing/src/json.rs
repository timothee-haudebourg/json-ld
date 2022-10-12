use json_ld_core::{
	Id,
	Context,
	Loc,
	syntax::Keyword,
	ProcessingMode,
	utils::as_array
};
use iref::{Iri, IriBuf, IriRef};
use futures::future::BoxFuture;
use generic_json::{JsonSendSync, JsonClone, ValueRef};
use crate::{
	Process,
	ProcessingStack,
	ProcessingOptions,
	ProcessingResult,
	ProcessedContext,
	Loader,
	Warning,
	Error,
	LocError
};

mod utils;
mod iri;
mod define;

use utils::*;
use iri::*;
use define::*;

/// Json local context.
pub trait JsonContext = JsonSendSync + JsonClone;

impl<J: JsonContext, T: Id> ProcessMeta<T> for J {
	fn process_full<'a, C: ProcessMeta<T>, L: Loader + Send + Sync>(
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
		async move {
			let mut warnings = Vec::new();
			let processed = process_context(
				active_context,
				self,
				stack,
				loader,
				base_url,
				options,
				&mut warnings,
			)
			.await?;
			Ok(ProcessedContext::with_warnings(processed, warnings))
		}
		.boxed()
	}
}

// This function tries to follow the recommended context proessing algorithm.
// See `https://www.w3.org/TR/json-ld11-api/#context-processing-algorithm`.
//
// The recommended default value for `remote_contexts` is the empty set,
// `false` for `override_protected`, and `true` for `propagate`.
fn process_context<
	'a,
	J: JsonContext,
	T: Id + Send + Sync,
	C: ProcessMeta<T>,
	L: Loader + Send + Sync,
>(
	active_context: &'a C,
	local_context: &'a J,
	mut remote_contexts: ProcessingStack,
	loader: &'a mut L,
	base_url: Option<Iri>,
	mut options: ProcessingOptions,
	warnings: &'a mut Vec<Loc<Warning, C::Source, C::MetaData>>,
) -> BoxFuture<'a, Result<C, LocError<C::Source, C::MetaData>>>
where
	L::Output: Into<J>,
{
	let source = loader.id_opt(base_url);
	let base_url_buf = base_url.map(IriBuf::from);

	async move {
		let base_url = base_url_buf.as_ref().map(|base_url| base_url.as_iri());

		// 1) Initialize result to the result of cloning active context.
		let mut result = active_context.clone();

		// 2) If `local_context` is an object containing the member @propagate,
		// its value MUST be boolean true or false, set `propagate` to that value.
		if let ValueRef::Object(obj) = local_context.as_value_ref() {
			if let Some(propagate_value) = obj.get(Keyword::Propagate.into()) {
				if options.processing_mode == ProcessingMode::JsonLd1_0 {
					return Err(Error::InvalidContextEntry
						.located(source, propagate_value.metadata().clone()));
				}

				if let ValueRef::Boolean(b) = propagate_value.as_value_ref() {
					options.propagate = b;
				} else {
					return Err(Error::InvalidPropagateValue
						.located(source, propagate_value.metadata().clone()));
				}
			}
		}

		// 3) If propagate is false, and result does not have a previous context,
		// set previous context in result to active context.
		if !options.propagate && result.previous_context().is_none() {
			result.set_previous_context(active_context.clone());
		}

		// 4) If local context is not an array, set it to an array containing only local context.
		let (local_context, _) = as_array(local_context);

		// 5) For each item context in local context:
		for context in local_context {
			match context.as_value_ref() {
				// 5.1) If context is null:
				ValueRef::Null => {
					// If `override_protected` is false and `active_context` contains any protected term
					// definitions, an invalid context nullification has been detected and processing
					// is aborted.
					if !options.override_protected && result.has_protected_items() {
						return Err(Error::InvalidContextNullification
							.located(source, context.metadata().clone()));
					} else {
						// Otherwise, initialize result as a newly-initialized active context, setting
						// previous_context in result to the previous value of result if propagate is
						// false. Continue with the next context.
						let previous_result = result;

						// Initialize `result` as a newly-initialized active context, setting both
						// `base_iri` and `original_base_url` to the value of `original_base_url` in
						// active context, ...
						result = C::new(active_context.original_base_url());

						// ... and, if `propagate` is `false`, `previous_context` in `result` to the
						// previous value of `result`.
						if !options.propagate {
							result.set_previous_context(previous_result);
						}
					}
				}

				// 5.2) If context is a string,
				ValueRef::String(context_str) => {
					let context_str: &str = context_str.as_ref();
					// Initialize `context` to the result of resolving context against base URL.
					// If base URL is not a valid IRI, then context MUST be a valid IRI, otherwise
					// a loading document failed error has been detected and processing is aborted.
					let context_iri = if let Ok(iri_ref) = IriRef::new(context_str) {
						resolve_iri(iri_ref, base_url).ok_or_else(|| {
							Error::LoadingDocumentFailed
								.located(source, context.metadata().clone())
						})?
					} else {
						return Err(Error::LoadingDocumentFailed
							.located(source, context.metadata().clone()));
					};

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
						let context_document = loader
							.load_context(context_iri.as_iri())
							.await
							.map_err(|e| e.located(source, context.metadata().clone()))?
							.cast::<J>();
						let loaded_context = context_document.context();

						// Set result to the result of recursively calling this algorithm, passing result
						// for active context, loaded context for local context, the documentUrl of context
						// document for base URL, and a copy of remote contexts.
						let new_options = ProcessingOptions {
							processing_mode: options.processing_mode,
							override_protected: false,
							propagate: true,
						};

						result = loaded_context
							.process_full(
								&result,
								remote_contexts.clone(),
								loader,
								Some(context_document.url()),
								new_options,
							)
							.await?
							.into_inner();
						// result = process_context(&result, loaded_context, remote_contexts, loader, Some(context_document.url()), new_options).await?
					}
				}

				// 5.4) Context definition.
				ValueRef::Object(context) => {
					// 5.5) If context has an @version entry:
					if let Some(version_value) = context.get(Keyword::Version.into()) {
						// 5.5.1) If the associated value is not `1.1`, an invalid @version value has
						// been detected.
						if version_value.as_f32() != Some(1.1)
							&& version_value.as_f64() != Some(1.1)
						{
							return Err(ErrorCode::InvalidVersionValue
								.located(source, version_value.metadata().clone()));
						}

						// 5.5.2) If processing mode is set to json-ld-1.0, a processing mode conflict
						// error has been detected.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(ErrorCode::ProcessingModeConflict
								.located(source, version_value.metadata().clone()));
						}
					}

					// 5.6) If context has an @import entry:
					let context: LocalContextObject<'_, J::Object> = if let Some(import_value) =
						context.get(Keyword::Import.into())
					{
						// 5.6.1) If processing mode is json-ld-1.0, an invalid context entry error
						// has been detected.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(ErrorCode::InvalidContextEntry
								.located(source, import_value.metadata().clone()));
						}

						if let Some(import_value_str) = import_value.as_str() {
							// 5.6.3) Initialize import to the result of resolving the value of
							// @import.
							let import = if let Ok(iri_ref) = IriRef::new(import_value_str) {
								resolve_iri(iri_ref, base_url).ok_or_else(|| {
									ErrorCode::InvalidImportValue
										.located(source, import_value.metadata().clone())
								})?
							} else {
								return Err(ErrorCode::InvalidImportValue
									.located(source, import_value.metadata().clone()));
							};

							// 5.6.4) Dereference import.
							let import_context_document = loader
								.load_context(import.as_iri())
								.await
								.map_err(|e| e.located(source, import_value.metadata().clone()))?
								.cast::<J>();
							let import_source = import_context_document.source();
							let import_context = import_context_document.into_context();
							let import_context_metadata = import_context.metadata().clone();

							// If the dereferenced document has no top-level map with an @context
							// entry, or if the value of @context is not a context definition
							// (i.e., it is not an map), an invalid remote context has been
							// detected and processing is aborted; otherwise, set import context
							// to the value of that entry.
							if let generic_json::Value::Object(import_context_obj) =
								import_context.into()
							{
								// If `import_context` has a @import entry, an invalid context entry
								// error has been detected and processing is aborted.
								if let Some((import_key, _)) =
									import_context_obj.get_key_value(Keyword::Import.into())
								{
									return Err(ErrorCode::InvalidContextEntry.located(
										Some(import_source),
										import_key.metadata().clone(),
									));
								}

								// Set `context` to the result of merging context into
								// `import_context`, replacing common entries with those from
								// `context`.
								let mut merged_context =
									LocalContextObject::new(Mown::Owned(import_context_obj));
								merged_context.merge_with(Mown::Borrowed(context));

								merged_context
							} else {
								return Err(ErrorCode::InvalidRemoteContext
									.located(Some(import_source), import_context_metadata));
							}
						} else {
							// 5.6.2) If the value of @import is not a string, an invalid
							// @import value error has been detected.
							return Err(ErrorCode::InvalidImportValue
								.located(source, import_value.metadata().clone()));
						}
					} else {
						LocalContextObject::new(Mown::Borrowed(context))
					};

					// 5.7) If context has a @base entry and remote contexts is empty, i.e.,
					// the currently being processed context is not a remote context:
					if remote_contexts.is_empty() {
						// Initialize value to the value associated with the @base entry.
						if let Some(value) = context.get(Keyword::Base.into()) {
							match value.as_value_ref() {
								ValueRef::Null => {
									// If value is null, remove the base IRI of result.
									result.set_base_iri(None);
								}
								ValueRef::String(value_str) => {
									let value_str: &str = value_str.as_ref();
									if let Ok(value_iri_ref) = IriRef::new(value_str) {
										match value_iri_ref.into_iri() {
											Ok(value_iri) => result.set_base_iri(Some(value_iri)),
											Err(value_not_iri) => {
												let resolved =
													resolve_iri(value_not_iri, result.base_iri())
														.ok_or_else(|| {
														ErrorCode::InvalidBaseIri.located(
															source,
															value.metadata().clone(),
														)
													})?;
												result.set_base_iri(Some(resolved.as_iri()))
											}
										}
									} else {
										return Err(ErrorCode::InvalidBaseIri
											.located(source, value.metadata().clone()));
									}
								}
								_ => {
									return Err(ErrorCode::InvalidBaseIri
										.located(source, value.metadata().clone()))
								}
							}
						}
					}

					// 5.8) If context has a @vocab entry:
					// Initialize value to the value associated with the @vocab entry.
					if let Some(value) = context.get(Keyword::Vocab.into()) {
						match value.as_value_ref() {
							ValueRef::Null => {
								// If value is null, remove any vocabulary mapping from result.
								result.set_vocabulary(None);
							}
							ValueRef::String(string_value) => {
								let string_value = string_value.as_ref();
								// Otherwise, if value is an IRI or blank node identifier, the
								// vocabulary mapping of result is set to the result of IRI
								// expanding value using true for document relative. If it is not
								// an IRI, or a blank node identifier, an invalid vocab mapping
								// error has been detected and processing is aborted.
								// NOTE: The use of blank node identifiers to value for @vocab is
								// obsolete, and may be removed in a future version of JSON-LD.
								match expansion::expand_iri(
									source,
									&result,
									string_value,
									value.metadata(),
									true,
									true,
									warnings,
								) {
									Term::Ref(vocab) => {
										result.set_vocabulary(Some(Term::Ref(vocab)))
									}
									_ => {
										return Err(ErrorCode::InvalidVocabMapping
											.located(source, value.metadata().clone()))
									}
								}
							}
							_ => {
								return Err(ErrorCode::InvalidVocabMapping
									.located(source, value.metadata().clone()))
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
				// 5.3) An invalid local context error has been detected.
				_ => {
					return Err(
						ErrorCode::InvalidLocalContext.located(source, context.metadata().clone())
					)
				}
			}
		}

		Ok(result)
	}
	.boxed()
}