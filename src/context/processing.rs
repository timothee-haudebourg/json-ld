use std::ops::Deref;
use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::rc::Rc;
use futures::future::{LocalBoxFuture, FutureExt};
use json::{JsonValue, object::Object as JsonObject};
use iref::{Iri, IriBuf, IriRef};
use crate::util::as_array;
use crate::{ProcessingMode, Error, ErrorCode, Keyword, BlankId, Id, Term, Property, is_keyword, is_keyword_like, Direction, ContainerType, expansion};
use super::{ContextProcessingOptions, LocalContext, ActiveContext, MutableActiveContext, ContextLoader, TermDefinition};

impl<T: Id, C: MutableActiveContext<T>> LocalContext<T, C> for JsonValue where C::LocalContext: From<JsonValue> {
	/// Load a local context.
	fn process_with<'a, L: ContextLoader<C::LocalContext>>(&'a self, active_context: &'a C, stack: ProcessingStack, loader: &'a mut L, base_url: Option<Iri>, options: ContextProcessingOptions) -> Pin<Box<dyn 'a + Future<Output = Result<C, Error>>>> {
		process_context(active_context, self, stack, loader, base_url, options)
	}

	fn as_json_ld(&self) -> &JsonValue {
		self
	}
}

pub fn has_protected_items<T: Id, C: ActiveContext<T>>(active_context: &C) -> bool {
	for (_, definition) in active_context.definitions() {
		if definition.protected {
			return true
		}
	}

	false
}

fn resolve_iri(iri_ref: IriRef, base_iri: Option<Iri>) -> Option<IriBuf> {
	match base_iri {
		Some(base_iri) => Some(iri_ref.resolved(base_iri)),
		None => match iri_ref.into_iri() {
			Ok(iri) => Some(iri.into()),
			Err(_) => None
		}
	}
}

struct StackNode {
	previous: Option<Rc<StackNode>>,
	url: IriBuf
}

impl StackNode {
	fn new(previous: Option<Rc<StackNode>>, url: IriBuf) -> StackNode {
		StackNode {
			previous,
			url
		}
	}

	fn contains(&self, url: Iri) -> bool {
		if self.url == url {
			true
		} else {
			match &self.previous {
				Some(prev) => prev.contains(url),
				None => false
			}
		}
	}
}

#[derive(Clone)]
pub struct ProcessingStack {
	head: Option<Rc<StackNode>>
}

impl ProcessingStack {
	pub fn new() -> ProcessingStack {
		ProcessingStack {
			head: None
		}
	}

	pub fn is_empty(&self) -> bool {
		self.head.is_none()
	}

	pub fn cycle(&self, url: Iri) -> bool {
		match &self.head {
			Some(head) => head.contains(url),
			None => false
		}
	}

	pub fn push(&mut self, url: Iri) -> bool {
		if self.cycle(url) {
			false
		} else {
			let mut head = None;
			std::mem::swap(&mut head, &mut self.head);
			self.head = Some(Rc::new(StackNode::new(head, url.into())));
			true
		}
	}
}

// This function tries to follow the recommended context proessing algorithm.
// See `https://www.w3.org/TR/json-ld11-api/#context-processing-algorithm`.
//
// The recommended default value for `remote_contexts` is the empty set,
// `false` for `override_protected`, and `true` for `propagate`.
fn process_context<'a, T: Id, C: MutableActiveContext<T>, L: ContextLoader<C::LocalContext>>(active_context: &'a C, local_context: &'a JsonValue, mut remote_contexts: ProcessingStack, loader: &'a mut L, base_url: Option<Iri>, mut options: ContextProcessingOptions) -> LocalBoxFuture<'a, Result<C, Error>> where C::LocalContext: From<JsonValue> {
	let base_url = match base_url {
		Some(base_url) => Some(IriBuf::from(base_url)),
		None => None
	};

	async move {
		let base_url = match base_url.as_ref() {
			Some(base_url) => Some(base_url.as_iri()),
			None => None
		};

		// 1) Initialize result to the result of cloning active context.
		let mut result = active_context.clone();

		// 2) If `local_context` is an object containing the member @propagate,
		// its value MUST be boolean true or false, set `propagate` to that value.
		if let JsonValue::Object(obj) = local_context {
			if let Some(propagate_value) = obj.get(Keyword::Propagate.into()) {
				if options.processing_mode == ProcessingMode::JsonLd1_0 {
					return Err(ErrorCode::InvalidContextEntry.into())
				}

				if let JsonValue::Boolean(b) = propagate_value {
					options.propagate = *b;
				} else {
					return Err(ErrorCode::InvalidPropagateValue.into())
				}
			}
		}

		// 3) If propagate is false, and result does not have a previous context,
		// set previous context in result to active context.
		if !options.propagate && result.previous_context().is_none() {
			result.set_previous_context(active_context.clone());
		}

		// 4) If local context is not an array, set it to an array containing only local context.
		let local_context = as_array(local_context);

		// 5) For each item context in local context:
		for context in local_context {
			match context {
				// 5.1) If context is null:
				JsonValue::Null => {
					// If `override_protected` is false and `active_context` contains any protected term
					// definitions, an invalid context nullification has been detected and processing
					// is aborted.
					if !options.override_protected && has_protected_items(active_context) {
						return Err(ErrorCode::InvalidContextNullification.into())
					} else {
						// Otherwise, initialize result as a newly-initialized active context, setting
						// previous_context in result to the previous value of result if propagate is
						// false. Continue with the next context.
						let previous_result = result;

						// Initialize `result` as a newly-initialized active context, setting both
						// `base_iri` and `original_base_url` to the value of `original_base_url` in
						// active context, ...
						result = C::new(active_context.original_base_url(), active_context.original_base_url());

						// ... and, if `propagate` is `false`, `previous_context` in `result` to the
						// previous value of `result`.
						if !options.propagate {
							result.set_previous_context(previous_result);
						}
					}
				},

				// 5.2) If context is a string,
				JsonValue::String(_) | JsonValue::Short(_) => {
					// Initialize `context` to the result of resolving context against base URL.
					// If base URL is not a valid IRI, then context MUST be a valid IRI, otherwise
					// a loading document failed error has been detected and processing is aborted.
					let context = if let Ok(iri_ref) = IriRef::new(context.as_str().unwrap()) {
						resolve_iri(iri_ref, base_url).ok_or(Error::from(ErrorCode::LoadingRemoteContextFailed))?
					} else {
						return Err(ErrorCode::LoadingDocumentFailed.into())
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
					if remote_contexts.push(context.as_iri()) {
						let context_document = loader.load(context.as_iri()).await?;
						let loaded_context = context_document.context();


						// Set result to the result of recursively calling this algorithm, passing result
						// for active context, loaded context for local context, the documentUrl of context
						// document for base URL, and a copy of remote contexts.
						let new_options = ContextProcessingOptions {
							processing_mode: options.processing_mode,
							override_protected: false,
							propagate: true
						};

						result = loaded_context.process_with(&result, remote_contexts.clone(), loader, Some(context_document.url()), new_options).await?;
						// result = process_context(&result, loaded_context, remote_contexts, loader, Some(context_document.url()), new_options).await?
					}
				},

				// 5.4) Context definition.
				JsonValue::Object(context) => {
					// 5.5) If context has an @version entry:
					if let Some(version_value) = context.get(Keyword::Version.into()) {
						// 5.5.1) If the associated value is not 1.1, an invalid @version value has
						// been detected.
						if version_value.as_str() != Some("1.1") && version_value.as_f32() != Some(1.1) {
							return Err(ErrorCode::InvalidVersionValue.into())
						}

						// 5.5.2) If processing mode is set to json-ld-1.0, a processing mode conflict
						// error has been detected.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(ErrorCode::ProcessingModeConflict.into())
						}
					}

					// 5.6) If context has an @import entry:
					let context = if let Some(import_value) = context.get(Keyword::Import.into()) {
						// 5.6.1) If processing mode is json-ld-1.0, an invalid context entry error
						// has been detected.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(ErrorCode::InvalidContextEntry.into())
						}

						if let Some(import_value) = import_value.as_str() {
							// 5.6.3) Initialize import to the result of resolving the value of
							// @import.
							let import = if let Ok(iri_ref) = IriRef::new(import_value) {
								resolve_iri(iri_ref, base_url).ok_or(Error::from(ErrorCode::InvalidImportValue))?
							} else {
								return Err(ErrorCode::InvalidImportValue.into())
							};

							// 5.6.4) Dereference import.
							let context_document = loader.load(import.as_iri()).await?;
							let import_context = context_document.context();

							// If the dereferenced document has no top-level map with an @context
							// entry, or if the value of @context is not a context definition
							// (i.e., it is not an map), an invalid remote context has been
							// detected and processing is aborted; otherwise, set import context
							// to the value of that entry.
							if let JsonValue::Object(import_context) = import_context.as_json_ld() {
								// If `import_context` has a @import entry, an invalid context entry
								// error has been detected and processing is aborted.
								if let Some(_) = import_context.get(Keyword::Import.into()) {
									return Err(ErrorCode::InvalidContextEntry.into());
								}

								// Set `context` to the result of merging context into
								// `import context`, replacing common entries with those from
								// `context`.
								let mut context = context.clone();
								for (key, value) in import_context.iter() {
									if context.get(key).is_none() {
										context.insert(key, value.clone());
									}
								}

								JsonObjectRef::Owned(context)
							} else {
								return Err(ErrorCode::InvalidRemoteContext.into())
							}
						} else {
							// 5.6.2) If the value of @import is not a string, an invalid
							// @import value error has been detected.
							return Err(ErrorCode::InvalidImportValue.into())
						}
					} else {
						JsonObjectRef::Borrowed(context)
					};

					// 5.7) If context has a @base entry and remote contexts is empty, i.e.,
					// the currently being processed context is not a remote context:
					if remote_contexts.is_empty() {
						// Initialize value to the value associated with the @base entry.
						if let Some(value) = context.get(Keyword::Base.into()) {
							match value {
								JsonValue::Null => {
									// If value is null, remove the base IRI of result.
									result.set_base_iri(None);
								},
								JsonValue::String(_) | JsonValue::Short(_) => {
									if let Ok(value) = IriRef::new(value.as_str().unwrap()) {
										match value.into_iri() {
											Ok(value) => {
												result.set_base_iri(Some(value))
											},
											Err(value) => {
												let resolved = resolve_iri(value, result.base_iri()).ok_or(Error::from(ErrorCode::InvalidBaseIri))?;
												result.set_base_iri(Some(resolved.as_iri()))
											}
										}
									} else {
										return Err(ErrorCode::InvalidBaseIri.into())
									}
								},
								_ => {
									return Err(ErrorCode::InvalidBaseIri.into())
								}
							}
						}
					}

					// 5.8) If context has a @vocab entry:
					// Initialize value to the value associated with the @vocab entry.
					if let Some(value) = context.get(Keyword::Vocab.into()) {
						match value {
							JsonValue::Null => {
								// If value is null, remove any vocabulary mapping from result.
								result.set_vocabulary(None);
							},
							JsonValue::String(_) | JsonValue::Short(_) => {
								let value = value.as_str().unwrap();
								// Otherwise, if value is an IRI or blank node identifier, the
								// vocabulary mapping of result is set to the result of IRI
								// expanding value using true for document relative. If it is not
								// an IRI, or a blank node identifier, an invalid vocab mapping
								// error has been detected and processing is aborted.
								// NOTE: The use of blank node identifiers to value for @vocab is
								// obsolete, and may be removed in a future version of JSON-LD.
								match expansion::expand_iri(&result, value, true, true) {
									Term::Prop(vocab) => result.set_vocabulary(Some(Term::Prop(vocab))),
									_ => return Err(ErrorCode::InvalidVocabMapping.into())
								}
							},
							_ => {
								return Err(ErrorCode::InvalidVocabMapping.into())
							}
						}
					}

					// 5.9) If context has a @language entry:
					if let Some(value) = context.get(Keyword::Language.into()) {
						if value.is_null() {
							// 5.9.2) If value is null, remove any default language from result.
							result.set_default_language(None);
						} else if let Some(str) = value.as_str() {
							// 5.9.3) Otherwise, if value is string, the default language of result is
							// set to value.
							result.set_default_language(Some(str.to_string()));
						} else {
							return Err(ErrorCode::InvalidDefaultLanguage.into())
						}
					}

					// 5.10) If context has a @direction entry:
					if let Some(value) = context.get(Keyword::Direction.into()) {
						// 5.10.1) If processing mode is json-ld-1.0, an invalid context entry error
						// has been detected and processing is aborted.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(ErrorCode::InvalidContextEntry.into())
						}

						if value.is_null() {
							// 5.10.3) If value is null, remove any base direction from result.
							result.set_default_base_direction(None);
						} else if let Some(str) = value.as_str() {
							let dir = match str {
								"ltr" => Direction::Ltr,
								"rtl" => Direction::Rtl,
								_ => return Err(ErrorCode::InvalidBaseDirection.into())
							};
							result.set_default_base_direction(Some(dir));
						} else {
							return Err(ErrorCode::InvalidBaseDirection.into())
						}
					}

					// 5.12) Create a map `defined` to keep track of whether or not a term
					// has already been defined or is currently being defined during recursion.
					let mut defined = HashMap::new();

					let protected = if let Some(JsonValue::Boolean(protected)) = context.get(Keyword::Protected.into()) {
						*protected
					} else {
						false
					};

					// 5.13) For each key-value pair in context where key is not a keyword,
					// invoke the Create Term Definition algorithm passing result for
					// active context, context for local context, key, defined, base URL,
					// and the value of the @protected entry from context, if any, for protected.
					// (and the value of override protected)
					for (key, _) in context.iter() {
						if !is_keyword(key) {
							define(&mut result, context.as_ref(), key, &mut defined, remote_contexts.clone(), loader, base_url, protected, options).await?;
						}
					}
				},
				// 5.3) An invalid local context error has been detected.
				_ => return Err(ErrorCode::InvalidLocalContext.into())
			}
		}

		Ok(result)
	}.boxed_local()
}

enum JsonObjectRef<'a> {
	Owned(JsonObject),
	Borrowed(&'a JsonObject)
}

impl<'a> JsonObjectRef<'a> {
	fn as_ref(&self) -> &JsonObject {
		match self {
			JsonObjectRef::Owned(obj) => &obj,
			JsonObjectRef::Borrowed(obj) => obj
		}
	}
}

impl<'a> Deref for JsonObjectRef<'a> {
	type Target = JsonObject;

	fn deref(&self) -> &JsonObject {
		self.as_ref()
	}
}

fn is_gen_delim(c: char) -> bool {
	match c {
		':' | '/' | '?' | '#' | '[' | ']' | '@' => true,
		_ => false
	}
}

fn is_gen_delim_or_blank<T: Id>(t: &Term<T>) -> bool {
	match t {
		Term::Keyword(_) => false,
		Term::Prop(Property::Blank(_)) => true,
		Term::Prop(Property::Id(id)) => {
			if let Some(c) = id.iri().as_str().chars().last() {
				is_gen_delim(c)
			} else {
				false
			}
		},
		Term::Unknown(_) => false,
		Term::Null => false
	}
}

fn contains_nz(id: &str, c: char) -> bool {
	if let Some(i) = id.find(c) {
		i > 0
	} else {
		false
	}
}

fn iri_eq_opt<T: Id>(a: &Option<Term<T>>, b: &Option<Term<T>>) -> bool {
	match (a, b) {
		(Some(a), Some(b)) => a.iri_eq(b),
		(None, None) => true,
		_ => false
	}
}

// fn define<'a>(&mut self, env: &mut DefinitionEnvironment<'a>, term: &str, value: &JsonValue) -> Result<(), Self::Error> {

/// Follows the `https://www.w3.org/TR/json-ld11-api/#create-term-definition` algorithm.
/// Default value for `base_url` is `None`. Default values for `protected` and `override_protected` are `false`.
pub fn define<'a, T: Id, C: MutableActiveContext<T>, L: ContextLoader<C::LocalContext>>(active_context: &'a mut C, local_context: &'a JsonObject, term: &'a str, defined: &'a mut HashMap<String, bool>, remote_contexts: ProcessingStack, loader: &'a mut L, base_url: Option<Iri<'a>>, protected: bool, options: ContextProcessingOptions) -> LocalBoxFuture<'a, Result<(), Error>> where C::LocalContext: From<JsonValue> {
	// let term = term.to_string();
	// let base_url = if let Some(base_url) = base_url {
	// 	Some(IriBuf::from(base_url))
	// } else {
	// 	None
	// };

	async move {
		match defined.get(term) {
			// If defined contains the entry term and the associated value is true (indicating
			// that the term definition has already been created), return.
			Some(true) => Ok(()),
			// Otherwise, if the value is false, a cyclic IRI mapping error has been detected and processing is aborted.
			Some(false) => Err(ErrorCode::CyclicIriMapping.into()),
			None => {
				// Initialize `value` to a copy of the value associated with the entry `term` in
				// `local_context`.
				if let Some(value) = local_context.get(term) {
					// Set the value associated with defined's term entry to false.
					// This indicates that the term definition is now being created but is not yet
					// complete.
					defined.insert(term.to_string(), false);

					// If term is @type, ...
					if term == "@type" {
						// ... and processing mode is json-ld-1.0, a keyword
						// redefinition error has been detected and processing is aborted.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(ErrorCode::KeywordRedefinition.into())
						}

						// At this point, `value` MUST be a map with only either or both of the
						// following entries:
						// An entry for @container with value @set.
						// An entry for @protected.
						// Any other value means that a keyword redefinition error has been detected
						// and processing is aborted.
						if let JsonValue::Object(value) = value {
							for (key, value) in value.iter() {
								match key {
									"@container" if value == "@set" => (),
									"@protected" => (),
									_ => return Err(ErrorCode::KeywordRedefinition.into())
								}
							}
						} else {
							return Err(ErrorCode::KeywordRedefinition.into())
						}
					}

					// Otherwise, since keywords cannot be overridden, term MUST NOT be a keyword and
					// a keyword redefinition error has been detected and processing is aborted.
					// If term has the form of a keyword (i.e., it matches the ABNF rule "@"1*ALPHA
					// from [RFC5234]), return; processors SHOULD generate a warning.
					if is_keyword_like(term) {
						// TODO warning
						return Ok(())
					}

					// Initialize `previous_definition` to any existing term definition for `term` in
					// `active_context`, removing that term definition from active context.
					let previous_definition = active_context.set(term, None);

					let mut simple_term = true;
					let value = match value {
						JsonValue::Null => {
							// If `value` is null, convert it to a map consisting of a single entry
							// whose key is @id and whose value is null.
							let mut map = JsonObject::with_capacity(1);
							map.insert("@id", json::Null);
							JsonObjectRef::Owned(map)
						},
						JsonValue::String(_) | JsonValue::Short(_) => {
							// Otherwise, if value is a string, convert it to a map consisting of a
							// single entry whose key is @id and whose value is value. Set simple
							// term to true (it already is).
							let mut map = JsonObject::with_capacity(1);
							map.insert("@id", value.clone());
							JsonObjectRef::Owned(map)
						},
						JsonValue::Object(value) => {
							simple_term = false;
							JsonObjectRef::Borrowed(value)
						},
						_ => {
							return Err(ErrorCode::InvalidTermDefinition.into())
						}
					};

					// Create a new term definition, `definition`, initializing `prefix` flag to
					// `false`, `protected` to `protected`, and `reverse_property` to `false`.
					let mut definition = TermDefinition::<T, C>::default();
					definition.protected = protected;

					// If the @protected entry in value is true set the protected flag in
					// definition to true.
					if let Some(protected_value) = value.get("@protected") {
						if let JsonValue::Boolean(b) = protected_value {
							definition.protected = *b;
						} else {
							// If the value of @protected is not a boolean, an invalid @protected
							// value error has been detected.
							return Err(ErrorCode::InvalidProtectedValue.into())
						}

						// If processing mode is json-ld-1.0, an invalid term definition has
						// been detected and processing is aborted.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(ErrorCode::InvalidTermDefinition.into())
						}
					}

					// If value contains the entry @type:
					if let Some(type_value) = value.get("@type") {
						// Initialize `typ` to the value associated with the `@type` entry, which
						// MUST be a string. Otherwise, an invalid type mapping error has been
						// detected and processing is aborted.
						if let Some(typ) = type_value.as_str() {
							// Set `typ` to the result of IRI expanding type, using local context,
							// and defined.
							if let Ok(typ) = expand_iri(active_context, typ, false, true, local_context, defined, remote_contexts.clone(), loader, options).await? {
								// If the expanded type is @json or @none, and processing mode is
								// json-ld-1.0, an invalid type mapping error has been detected and
								// processing is aborted.
								if options.processing_mode == ProcessingMode::JsonLd1_0 && (typ == Term::Keyword(Keyword::JSON) || typ == Term::Keyword(Keyword::None)) {
									return Err(ErrorCode::InvalidTypeMapping.into())
								}

								if let Ok(typ) = typ.try_into() {
									// Set the type mapping for definition to type.
									definition.typ = Some(typ);
								} else {
									// Otherwise, if the expanded type is neither @id, nor @json, nor
									// @none, nor @vocab, nor an IRI, an invalid type mapping error has
									// been detected and processing is aborted.
									return Err(ErrorCode::InvalidTypeMapping.into())
								}
							} else {
								return Err(ErrorCode::InvalidTypeMapping.into())
							}
						} else {
							return Err(ErrorCode::InvalidTypeMapping.into())
						}
					}

					// If `value` contains the entry @reverse:
					if let Some(reverse_value) = value.get("@reverse") {
						// If `value` contains `@id` or `@nest`, entries, an invalid reverse
						// property error has been detected and processing is aborted.
						if value.get("@id").is_some() || value.get("@nest").is_some() {
							return Err(ErrorCode::InvalidReverseProperty.into())
						}

						if let Some(reverse_value) = reverse_value.as_str() {
							// If the value associated with the @reverse entry is a string having
							// the form of a keyword, return; processors SHOULD generate a warning.
							if is_keyword_like(reverse_value) {
								// TODO warning
								return Ok(())
							}

							// Otherwise, set the IRI mapping of definition to the result of IRI
							// expanding the value associated with the @reverse entry, using
							// local context, and defined.
							// If the result does not have the form of an IRI or a blank node
							// identifier, an invalid IRI mapping error has been detected and
							// processing is aborted.
							match expand_iri(active_context, reverse_value, false, true, local_context, defined, remote_contexts, loader, options).await? {
								Ok(Term::Prop(mapping)) => {
									definition.value = Some(Term::Prop(mapping))
								},
								_ => {
									return Err(ErrorCode::InvalidIriMapping.into())
								}
							}

							// If `value` contains an `@container` entry, set the `container`
							// mapping of `definition` to an array containing its value;
							// if its value is neither `@set`, nor `@index`, nor null, an
							// invalid reverse property error has been detected (reverse properties
							// only support set- and index-containers) and processing is aborted.
							if let Some(container_value) = value.get("@container") {
								match container_value {
									JsonValue::Null => (),
									JsonValue::String(_) | JsonValue::Short(_) => {
										if let Ok(container_value) = ContainerType::try_from(container_value.as_str().unwrap()) {
											match container_value {
												ContainerType::Set | ContainerType::Index => {
													definition.container.add(container_value);
												},
												_ => return Err(ErrorCode::InvalidReverseProperty.into())
											}
										} else {
											return Err(ErrorCode::InvalidReverseProperty.into())
										}
									},
									_ => return Err(ErrorCode::InvalidReverseProperty.into())
								};
							}

							// Set the `reverse_property` flag of `definition` to `true`.
							definition.reverse_property = true;

							// Set the term definition of `term` in `active_context` to
							// `definition` and the value associated with `defined`'s entry `term`
							// to `true` and return.
							active_context.set(term, Some(definition.into()));
							defined.insert(term.to_string(), true);
							return Ok(())
						} else {
							// If the value associated with the `@reverse` entry is not a string,
							// an invalid IRI mapping error has been detected and processing is
							// aborted.
							return Err(ErrorCode::InvalidIriMapping.into())
						}
					}

					// If `value` contains the entry `@id` and its value does not equal `term`:
					let id_value = value.get("@id");
					if id_value.is_some() && id_value.unwrap().as_str() != Some(term) {
						let id_value = id_value.unwrap();
						// If the `@id` entry of value is `null`, the term is not used for IRI
						// expansion, but is retained to be able to detect future redefinitions
						// of this term.
						if !id_value.is_null() {
							// Otherwise:
							if let Some(id_value) = id_value.as_str() {
								// If the value associated with the `@id` entry is not a
								// keyword, but has the form of a keyword, return;
								// processors SHOULD generate a warning.
								if is_keyword_like(id_value) && !is_keyword(id_value) {
									// TODO warning
									return Ok(())
								}

								// Otherwise, set the IRI mapping of `definition` to the result
								// of IRI expanding the value associated with the `@id` entry,
								// using `local_context`, and `defined`.
								definition.value = if let Ok(value) = expand_iri(active_context, id_value, false, true, local_context, defined, remote_contexts.clone(), loader, options).await? {
									// if it equals `@context`, an invalid keyword alias error has
									// been detected and processing is aborted.
									if value == Term::Keyword(Keyword::Context) {
										return Err(ErrorCode::InvalidKeywordAlias.into())
									}

									Some(value)
								} else {
									// If the resulting IRI mapping is neither a keyword,
									// nor an IRI, nor a blank node identifier, an
									// invalid IRI mapping error has been detected and processing
									// is aborted;
									return Err(ErrorCode::InvalidIriMapping.into())
								};

								// If `term` contains a colon (:) anywhere but as the first or
								// last character of `term`, or if it contains a slash (/)
								// anywhere:
								if contains_nz(term, ':') || term.contains('/') {
									// Set the value associated with `defined`'s `term` entry
									// to `true`.
									defined.insert(term.to_string(), true);

									// If the result of IRI expanding `term` using
									// `local_context`, and `defined`, is not the same as the
									// IRI mapping of definition, an invalid IRI mapping error
									// has been detected and processing is aborted.
									if let Ok(expanded_term) = expand_iri(active_context, term, false, true, local_context, defined, remote_contexts.clone(), loader, options).await? {
										if !iri_eq_opt(&Some(expanded_term), &definition.value) {
											return Err(ErrorCode::InvalidIriMapping.into())
										}
									} else {
										return Err(ErrorCode::InvalidIriMapping.into())
									}
								}

								// If `term` contains neither a colon (:) nor a slash (/),
								// simple term is true, and if the IRI mapping of definition
								// is either an IRI ending with a gen-delim character,
								// or a blank node identifier, set the `prefix` flag in
								// `definition` to true.
								if !term.contains(':') && !term.contains('/') && simple_term && is_gen_delim_or_blank(definition.value.as_ref().unwrap()) {
									definition.prefix = true;
								}
							} else {
								// If the value associated with the `@id` entry is not a
								// string, an invalid IRI mapping error has been detected and
								// processing is aborted.
								return Err(ErrorCode::InvalidIriMapping.into())
							}
						}
					} else if contains_nz(term, ':') {
						// Otherwise if the `term` contains a colon (:) anywhere after the first
						// character:
						let i = term.find(':').unwrap();
						let (prefix, suffix) = term.split_at(i);
						let suffix = &suffix[1..suffix.len()];

						// If `term` is a compact IRI with a prefix that is an entry in local
						// context a dependency has been found.
						// Use this algorithm recursively passing `active_context`,
						// `local_context`, the prefix as term, and `defined`.
						define(active_context, local_context, prefix, defined, remote_contexts.clone(), loader, None, false, options.with_no_override()).await?;

						// If `term`'s prefix has a term definition in `active_context`, set the
						// IRI mapping of `definition` to the result of concatenating the value
						// associated with the prefix's IRI mapping and the term's suffix.
						if let Some(prefix_definition) = active_context.get(prefix) {
							let mut result = String::new();

							if let Some(prefix_key) = &prefix_definition.value {
								if let Some(prefix_iri) = prefix_key.iri() {
									result = prefix_iri.as_str().to_string()
								}
							}

							result.push_str(suffix);

							if let Ok(iri) = Iri::new(result.as_str()) {
								definition.value = Some(Term::<T>::from(T::from_iri(iri)))
							} else {
								return Err(ErrorCode::InvalidIriMapping.into())
							}
						} else {
							// Otherwise, `term` is an IRI or blank node identifier.
							// Set the IRI mapping of `definition` to `term`.
							if prefix == "_" { // blank node
								definition.value = Some(BlankId::new(suffix).into())
							} else {
								if let Ok(iri) = Iri::new(term) {
									definition.value = Some(Term::<T>::from(T::from_iri(iri)))
								} else {
									return Err(ErrorCode::InvalidIriMapping.into())
								}
							}
						}
					} else if term.contains('/') {
						// Term is a relative IRI reference.
						// Set the IRI mapping of definition to the result of IRI expanding
						// term.
						match expansion::expand_iri(active_context, term, false, true) {
							Term::Prop(Property::Id(id)) => {
								definition.value = Some(id.into())
							},
							// If the resulting IRI mapping is not an IRI, an invalid IRI mapping
							// error has been detected and processing is aborted.
							_ => return Err(ErrorCode::InvalidIriMapping.into())
						}
					} else if term == "@type" {
						// Otherwise, if `term` is ``@type`, set the IRI mapping of definition to
						// `@type`.
						definition.value = Some(Term::Keyword(Keyword::Type))
					} else if let Some(vocabulary) = active_context.vocabulary() {
						// Otherwise, if `active_context` has a vocabulary mapping, the IRI mapping
						// of `definition` is set to the result of concatenating the value
						// associated with the vocabulary mapping and `term`.
						// If it does not have a vocabulary mapping, an invalid IRI mapping error
						// been detected and processing is aborted.
						if let Some(vocabulary_iri) = vocabulary.iri() {
							let mut result = vocabulary_iri.as_str().to_string();
							result.push_str(term);
							if let Ok(iri) = Iri::new(result.as_str()) {
								definition.value = Some(Term::<T>::from(T::from_iri(iri)))
							} else {
								return Err(ErrorCode::InvalidIriMapping.into())
							}
						} else {
							return Err(ErrorCode::InvalidIriMapping.into())
						}
					} else {
						// If it does not have a vocabulary mapping, an invalid IRI mapping error
						// been detected and processing is aborted.
						return Err(ErrorCode::InvalidIriMapping.into())
					}

					// If value contains the entry @container:
					if let Some(container_value) = value.get("@container") {
						// If the container value is @graph, @id, or @type, or is otherwise not a
						// string, generate an invalid container mapping error and abort processing
						// if processing mode is json-ld-1.0.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							match container_value.as_str() {
								Some("@graph") | Some("@id") | Some("@type") | None => {
									return Err(ErrorCode::InvalidContainerMapping.into())
								},
								_ => ()
							}
						}

						// Initialize `container` to the value associated with the `@container`
						// entry, which MUST be either `@graph`, `@id`, `@index`, `@language`,
						// `@list`, `@set`, `@type`, or an array containing exactly any one of
						// those keywords, an array containing `@graph` and either `@id` or
						// `@index` optionally including `@set`, or an array containing a
						// combination of `@set` and any of `@index`, `@graph`, `@id`, `@type`,
						// `@language` in any order.
						// Otherwise, an invalid container mapping has been detected and processing
						// is aborted.
						for entry in as_array(container_value) {
							if let Some(entry) = entry.as_str() {
								match ContainerType::try_from(entry) {
									Ok(c) => {
										if !definition.container.add(c) {
											return Err(ErrorCode::InvalidContainerMapping.into())
										}
									},
									Err(_) => return Err(ErrorCode::InvalidContainerMapping.into())
								}
							} else {
								return Err(ErrorCode::InvalidContainerMapping.into())
							}
						}

						// Set the container mapping of definition to container coercing to an
						// array, if necessary.
						// already done.

						// If the `container` mapping of definition includes `@type`:
						if definition.container.contains(ContainerType::Type) {
							if let Some(typ) = &definition.typ {
								// If type mapping in definition is neither `@id` nor `@vocab`,
								// an invalid type mapping error has been detected and processing
								// is aborted.
								match typ {
									Term::Keyword(Keyword::Id) | Term::Keyword(Keyword::Vocab) => (),
									_ => return Err(ErrorCode::InvalidTypeMapping.into())
								}
							} else {
								// If type mapping in definition is undefined, set it to @id.
								definition.typ = Some(Term::Keyword(Keyword::Id))
							}
						}
					}

					// If value contains the entry @index:
					if let Some(index_value) = value.get("@index") {
						// If processing mode is json-ld-1.0 or container mapping does not include
						// `@index`, an invalid term definition has been detected and processing
						// is aborted.
						if !definition.container.contains(ContainerType::Index) || options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(ErrorCode::InvalidTermDefinition.into())
						}

						// Initialize `index` to the value associated with the `@index` entry,
						// which MUST be a string expanding to an IRI.
						// Otherwise, an invalid term definition has been detected and processing
						// is aborted.
						if let Some(index) = index_value.as_str() {
							match expansion::expand_iri(active_context, index, false, true) {
								Term::Prop(Property::Id(_)) => (),
								_ => {
									return Err(ErrorCode::InvalidTermDefinition.into())
								}
							}

							definition.index = Some(index.to_string())
						} else {
							return Err(ErrorCode::InvalidTermDefinition.into())
						}
					}

					// If `value` contains the entry `@context`:
					if let Some(context) = value.get("@context") {
						// If processing mode is json-ld-1.0, an invalid term definition has been
						// detected and processing is aborted.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(ErrorCode::InvalidTermDefinition.into())
						}

						// Initialize `context` to the value associated with the @context entry,
						// which is treated as a local context.
						// done.

						// Invoke the Context Processing algorithm using the `active_context`,
						// `context` as local context, `base_url`, and `true` for override
						// protected.
						if let Err(_) = process_context(active_context, context, remote_contexts.clone(), loader, base_url, options.with_override()).await {
							// If any error is detected, an invalid scoped context error has been
							// detected and processing is aborted.
							return Err(ErrorCode::InvalidScopedContext.into())
						}

						// Set the local context of definition to context, and base URL to base URL.
						definition.context = Some(context.clone().into());
						definition.base_url = base_url.as_ref().map(|url| url.into());
					}

					// If `value` contains the entry `@language` and does not contain the entry
					// `@type`:
					if value.get("@type").is_none() {
						if let Some(language_value) = value.get("@language") {
							// Initialize `language` to the value associated with the `@language`
							// entry, which MUST be either null or a string.
							// If `language` is not well-formed according to section 2.2.9 of
							// [BCP47], processors SHOULD issue a warning.
							// Otherwise, an invalid language mapping error has been detected and
							// processing is aborted.
							// Set the `language` mapping of definition to `language`.
							definition.language = Some(match language_value {
								JsonValue::Null => None,
								JsonValue::String(_) | JsonValue::Short(_) => {
									// TODO lang tags
									Some(language_value.as_str().unwrap().to_string())
								},
								_ => {
									return Err(ErrorCode::InvalidLanguageMapping.into())
								}
							});
						}

						// If `value` contains the entry `@direction` and does not contain the
						// entry `@type`:
						if let Some(direction_value) = value.get("@direction") {
							// Initialize `direction` to the value associated with the `@direction`
							// entry, which MUST be either null, "ltr", or "rtl".
							definition.direction = Some(match direction_value.as_str() {
								Some("ltr") => Some(Direction::Ltr),
								Some("rtl") => Some(Direction::Rtl),
								_ => {
									if direction_value.is_null() {
										None
									} else {
										// Otherwise, an invalid base direction error has been
										// detected and processing is aborted.
										return Err(ErrorCode::InvalidBaseDirection.into())
									}
								}
							});
						}
					}

					// If value contains the entry @nest:
					if let Some(nest_value) = value.get("@nest") {
						// If processing mode is json-ld-1.0, an invalid term definition has been
						// detected and processing is aborted.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(ErrorCode::InvalidTermDefinition.into())
						}

						// Initialize `nest` value in `definition` to the value associated with the
						// `@nest` entry, which MUST be a string and MUST NOT be a keyword other
						// than @nest.
						if let Some(nest_value) = nest_value.as_str() {
							if is_keyword(nest_value) && nest_value != "@nest" {
								return Err(ErrorCode::InvalidNestValue.into())
							}

							definition.nest = Some(nest_value.to_string());
						} else {
							// Otherwise, an invalid @nest value error has been detected and
							// processing is aborted.
							return Err(ErrorCode::InvalidNestValue.into())
						}
					}

					// If value contains the entry @prefix:
					if let Some(prefix_value) = value.get("@prefix") {
						// If processing mode is json-ld-1.0, or if `term` contains a colon (:) or
						// slash (/), an invalid term definition has been detected and processing
						// is aborted.
						if term.contains(':') || term.contains('/') || options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(ErrorCode::InvalidTermDefinition.into())
						}

						// Set the `prefix` flag to the value associated with the @prefix entry,
						// which MUST be a boolean.
						// Otherwise, an invalid @prefix value error has been detected and
						// processing is aborted.
						if let Some(prefix) = prefix_value.as_bool() {
							definition.prefix = prefix
						} else {
							return Err(ErrorCode::InvalidPrefixValue.into())
						}

						// If the `prefix` flag of `definition` is set to `true`, and its IRI
						// mapping is a keyword, an invalid term definition has been detected and
						// processing is aborted.
						if definition.prefix && definition.value.as_ref().unwrap().is_keyword() {
							return Err(ErrorCode::InvalidTermDefinition.into())
						}
					}

					// If value contains any entry other than @id, @reverse, @container, @context,
					// @direction, @index, @language, @nest, @prefix, @protected, or @type, an
					// invalid term definition error has been detected and processing is aborted.
					for (key, _) in value.iter() {
						match key {
							"@id" | "@reverse" | "@container" | "@context" | "@direction" | "@index" | "@language" | "@nest" | "@prefix" | "@protected" | "@type" => (),
							_ => {
								return Err(ErrorCode::InvalidTermDefinition.into())
							}
						}
					}

					// If override protected is false and previous_definition exists and is protected;
					if !options.override_protected {
						if let Some(previous_definition) = previous_definition {
							if previous_definition.protected {
								// If `definition` is not the same as `previous_definition`
								// (other than the value of protected), a protected term
								// redefinition error has been detected, and processing is aborted.
								if definition != previous_definition {
									return Err(ErrorCode::ProtectedTermRedefinition.into())
								}

								// Set `definition` to `previous definition` to retain the value of
								// protected.
								definition.protected = true;
							}
						}
					}

					// Set the term definition of `term` in `active_context` to `definition` and
					// set the value associated with `defined`'s entry term to true.
					active_context.set(term, Some(definition.into()));
					defined.insert(term.to_string(), true);
				}

				// if the term is not in `local_context`.
				Ok(())
			}
		}
	}.boxed_local()
}

/// Default values for `document_relative` and `vocab` should be `false` and `true`.
pub fn expand_iri<'a, T: Id, C: MutableActiveContext<T>, L: ContextLoader<C::LocalContext>>(active_context: &'a mut C, value: &str, document_relative: bool, vocab: bool, local_context: &'a JsonObject, defined: &'a mut HashMap<String, bool>, remote_contexts: ProcessingStack, loader: &'a mut L, options: ContextProcessingOptions) -> impl 'a + Future<Output = Result<Result<Term<T>, Option<String>>, Error>> where C::LocalContext: From<JsonValue> {
	let value = value.to_string();
	async move {
		if let Ok(keyword) = Keyword::try_from(value.as_ref()) {
			Ok(Ok(Term::Keyword(keyword)))
		} else {
			// If value has the form of a keyword, a processor SHOULD generate a warning and return
			// null.
			// TODO

			// If `local_context` is not null, it contains an entry with a key that equals value, and the
			// value of the entry for value in defined is not true, invoke the Create Term Definition
			// algorithm, passing active context, local context, value as term, and defined. This will
			// ensure that a term definition is created for value in active context during Context
			// Processing.
			define(active_context, local_context, value.as_ref(), defined, remote_contexts.clone(), loader, None, false, options.with_no_override()).await?;

			if let Some(term_definition) = active_context.get(value.as_ref()) {
				// If active context has a term definition for value, and the associated IRI mapping
				// is a keyword, return that keyword.
				if let Some(value) = &term_definition.value {
					if value.is_keyword() {
						return Ok(Ok(value.clone()))
					}
				}

				// If vocab is true and the active context has a term definition for value, return the
				// associated IRI mapping.
				if vocab {
					if let Some(value) = &term_definition.value {
						return Ok(Ok(value.clone()))
					} else {
						return Ok(Err(Some(value.to_string())))
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
						return Ok(Ok(BlankId::new(suffix).into()))
					}

					if suffix.starts_with("//") {
						if let Ok(iri) = Iri::new(value.as_ref() as &str) {
							return Ok(Ok(T::from_iri(iri).into()))
						} else {
							return Ok(Err(Some(value.to_string())))
						}
					}

					// If local context is not null, it contains a `prefix` entry, and the value of the
					// prefix entry in defined is not true, invoke the Create Term Definition
					// algorithm, passing active context, local context, prefix as term, and defined.
					// This will ensure that a term definition is created for prefix in active context
					// during Context Processing.
					define(active_context, local_context, prefix, defined, remote_contexts, loader, None, false, options.with_no_override()).await?;

					// If active context contains a term definition for prefix having a non-null IRI
					// mapping and the prefix flag of the term definition is true, return the result
					// of concatenating the IRI mapping associated with prefix and suffix.
					if let Some(term_definition) = active_context.get(prefix) {
						if term_definition.prefix {
							if let Some(mapping) = &term_definition.value {
								let mut result = mapping.to_string();
								result.push_str(suffix);

								if let Ok(result) = Iri::new(&result) {
									return Ok(Ok(T::from_iri(result).into()))
								} else {
									if let Ok(blank) = BlankId::try_from(result.as_ref()) {
										return Ok(Ok(blank.into()))
									} else {
										return Ok(Err(Some(result)))
									}
								}
							}
						}
					}

					// If value has the form of an IRI, return value.
					if let Ok(result) = Iri::new(value.as_ref() as &str) {
						return Ok(Ok(T::from_iri(result).into()))
					}
				}
			}

			// If vocab is true, and active context has a vocabulary mapping, return the result of
			// concatenating the vocabulary mapping with value.
			if vocab {
				if let Some(vocabulary) = active_context.vocabulary() {
					if let Term::Prop(mapping) = vocabulary {
						let mut result = mapping.as_str().to_string();
						result.push_str(value.as_ref());

						if let Ok(result) = Iri::new(&result) {
							return Ok(Ok(T::from_iri(result).into()))
						} else {
							if let Ok(blank) = BlankId::try_from(result.as_ref()) {
								return Ok(Ok(blank.into()))
							} else {
								return Ok(Err(Some(result)))
							}
						}
					} else {
						return Ok(Err(Some(value.to_string())))
					}
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
						return Ok(Ok(T::from_iri(value.as_iri()).into()))
					} else {
						return Ok(Err(Some(value.to_string())))
					}
				} else {
					return Ok(Err(Some(value.to_string())))
				}
			}

			// Return value as is.
			Ok(Err(Some(value.to_string())))
		}
	}
}
