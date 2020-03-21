use async_trait::async_trait;
use std::ops::Deref;
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;
use std::convert::TryFrom;
use futures::future::{LocalBoxFuture, FutureExt};
use json::{JsonValue, object::Object as JsonObject};
use iref::{Iri, IriBuf, IriRef};
use crate::{Error, Keyword, is_keyword, is_keyword_like, Direction, Container, ContainerType, expansion};
use super::{LocalContext, ActiveContext, MutableActiveContext, Id, Key, TermDefinition};

pub enum ContextProcessingError {
	InvalidIri,

	InvalidIriRef,

	InvalidLocalContext,

	/// The `@propagate` value is not set to a boolean.
	InvalidPropagateValue,

	InvalidVersionValue,

	InvalidBaseIri,

	InvalidVocabMapping,

	InvalidDefaultLanguage,

	InvalidBaseDirection,

	InvalidContextNullification,

	ContextOverflow,

	LoadingDocumentFailed,

	LoadingRemoteContextFailed,

	InvalidRemoteContext,

	InvalidImportValue,

	InvalidContextEntry,

	InvalidTermDefinition,

	InvalidProtectedValue,

	InvalidTypeMapping,

	InvalidReverseProperty,

	InvalidIriMapping,

	InvalidKeywordAlias,

	CyclicIriMapping,

	KeywordRedefinition,

	InvalidContainerMapping,

	InvalidLanguageMapping,

	InvalidNestValue,

	InvalidPrefixValue,

	ProtectedTermRedefinition
}

impl From<reqwest::Error> for ContextProcessingError {
	fn from(e: reqwest::Error) -> ContextProcessingError {
		ContextProcessingError::LoadingDocumentFailed
	}
}

//
// // Type of a definition.
// pub enum Type {
// 	// @id
// 	Id
// }
//
#[async_trait]
impl<T: Id, C: MutableActiveContext<T>> LocalContext<T, C> for JsonValue {
	/// Load a local context.
	fn process<'a>(&'a self, active_context: &'a C, base_url: Iri) -> Pin<Box<dyn 'a + Future<Output = Result<C, ContextProcessingError>>>> {
		process_context(active_context, self, base_url, Arc::new(RemoteContexts::new()), false, true)
	}
}
//
// fn parse_container(json: &JsonValue) -> Result<Vec<Container>> {
// 	let mut container = Vec::new();
//
// 	match json.as_str() {
// 		Some(id) => {
// 			container.push(container_by_id(id)?)
// 		},
// 		None => {
// 			match json {
// 				JsonValue::Array(vec) => {
// 					for e in vec {
// 						if let Some(id) = json.as_str() {
// 							container.push(container_by_id(id)?)
// 						} else {
// 							return Err(Error::InvalidContainerMapping)
// 						}
// 					}
// 				},
// 				_ => return Err(Error::InvalidContainerMapping)
// 			}
// 		}
// 	}
//
// 	let mut is_valid = true;
//
// 	if container.contains(Container::List) {
// 		if container.len() > 1 {
// 			return Err(Error::InvalidContainerMapping)
// 		}
// 	} else if container.contains(Container::Graph) {
// 		for c in &container {
// 			if c != Container::Graph && c != Container::Id && c != Container::Index && c != Container::Set {
// 				return Err(Error::InvalidContainerMapping)
// 			}
// 		}
// 	} else if container.contains(Container::Graph) {
// 		is_valid &= container.len() <= 2
// 	} else {
// 		is_valid &= container.len() <= 1
// 	}
//
// 	return container;
// }
//
// /// JSON-LD context.
// pub trait Context: Sized + Clone {
// 	type Term: grdf::Entity;
// 	type Error;
//
// 	/// Retreive the key bound to the given id.
// 	fn key(&self, id: &str) -> Result<Key<T>, Self::Error>;
//
// 	/// Set (or unset) the definition of a term, and return the previous definition if any.
// 	fn set(&mut self, term: &str, definition: Option<TermDefinition>) -> Result<Option<TermDefinition>, Self::Error>;
// }
//
// fn is_keyword(str: &str) -> bool {
// 	match str {
// 		"@base" | "@container" | "@context" | "@direction" | "@graph" | "@id" |
// 		"@import" | "@imported" | "@index" | "@json" | "@language" | "@list" |
// 		"@nest" | "@none" | "@prefix" | "@propagate" | "@protected" | "@reverse" |
// 		"@set" | "@value" | "@version" | "@vocab" => true,
// 		_ => false
// 	}
// }
//
// pub struct Loader<'a, C: 'a + Context> {
// 	/// Active context. The one beeing modified.
// 	ctx: &'a mut C,
//
// 	/// Terms that are beeing loaded, or are already loaded.
// 	defined: HashMap<String, bool>,
//
// 	/// Local contexts beeing loaded.
// 	stack: Vec<&'a JsonObject>
// }
//
// trait IriContext {
// 	fn resolve(iri: Iri) -> Result<Iri, Error>;
// }
//
// impl<T: Context> IriContext for T {
// 	fn resolve(iri: Iri) -> Result<Iri, Error> {
// 		// ...
// 	}
// }
//
// pub struct LocalContext {
// 	/// The parent context.
// 	parent: Option<Rc<dyn Context>>,
//
// 	/// Does the context propagates to sub nodes.
// 	propagate: bool,
//
// 	/// Is it possible to redefine protected terms.
// 	override_protected: bool
// }

pub fn as_array(json: &JsonValue) -> &[JsonValue] {
	match json {
		JsonValue::Array(ary) => ary,
		_ => unsafe { std::mem::transmute::<&JsonValue, &[JsonValue; 1]>(json) as &[JsonValue] }
	}
}

pub fn has_protected_items<T: Id, C: ActiveContext<T>>(active_context: &C) -> bool {
	for (term, definition) in active_context.definitions() {
		if definition.protected {
			return true
		}
	}

	false
}

struct RemoteContext {
	//
}

impl RemoteContext {
	pub fn context(&self) -> &JsonValue {
		panic!("TODO context")
	}

	pub fn url(&self) -> Iri {
		panic!("TODO url")
	}
}

struct RemoteContexts {
	// ...
}

impl RemoteContexts {
	pub fn new() -> RemoteContexts {
		RemoteContexts {
			// ...
		}
	}

	pub fn is_empty(&self) -> bool {
		panic!("TODO")
	}

	pub async fn load(&self, url: Iri<'_>) -> Result<RemoteContext, ContextProcessingError> {
		use reqwest::header::*;

		let client = reqwest::Client::new();
		let request = client.get(url.as_str()).header(ACCEPT, "application/ld+json, application/json");
		let response = request.send().await?;

		// // ...
		//
		// response.headers().get_all(CONTENT_TYPE).find(|&value| value == "").is_some();
		// let bytes = response.bytes().await?;
		//
		// // ...
		panic!("TODO")
	}
}

// fn resolve<T, C: ActiveContext<T>>(context: &C, iri_ref: IriRef) -> Result<IriBuf, ContextProcessingError> {
// 	if let Some(base_iri) = context.base_iri() {
// 		Ok(iri_ref.resolved(base_iri))
// 	} else {
// 		Err(ContextProcessingError::InvalidBaseIri)
// 	}
// }

// This function tries to follow the recommended context proessing algorithm.
// See `https://www.w3.org/TR/json-ld11-api/#context-processing-algorithm`.
//
// The recommended default value for `remote_contexts` is the empty set,
// `false` for `override_protected`, and `true` for `propagate`.
fn process_context<'a, T: Id, C: MutableActiveContext<T>>(active_context: &'a C, local_context: &'a JsonValue, base_url: Iri, remote_contexts: Arc<RemoteContexts>, mut override_protected: bool, mut propagate: bool) -> LocalBoxFuture<'a, Result<C, ContextProcessingError>> {
	let base_url = IriBuf::from(base_url);
	async move {
		// 1) Initialize result to the result of cloning active context.
		let mut result = active_context.copy();

		// 2) If `local_context` is an object containing the member @propagate,
		// its value MUST be boolean true or false, set `propagate` to that value.
		if let JsonValue::Object(obj) = local_context {
			if let Some(propagate_value) = obj.get(Keyword::Propagate.into()) {
				if let JsonValue::Boolean(b) = propagate_value {
					propagate = *b;
				} else {
					return Err(ContextProcessingError::InvalidPropagateValue)
				}
			}
		}

		// 3) If propagate is false, and result does not have a previous context,
		// set previous context in result to active context.
		if !propagate && result.previous_context().is_none() {
			result.set_previous_context(active_context.copy());
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
					if !override_protected && has_protected_items(active_context) {
						return Err(ContextProcessingError::InvalidContextNullification)
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
						if !propagate {
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
						iri_ref.resolved(base_url.as_iri())
					} else {
						return Err(ContextProcessingError::LoadingDocumentFailed)
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
					let context_document = remote_contexts.load(context.as_iri()).await?;
					let loaded_context = context_document.context();

					// Set result to the result of recursively calling this algorithm, passing result
					// for active context, loaded context for local context, the documentUrl of context
					// document for base URL, and a copy of remote contexts.
					result = process_context(&result, loaded_context, context_document.url(), remote_contexts.clone(), false, true).await?;
				},

				// 5.4) Context definition.
				JsonValue::Object(context) => {
					// 5.5) If context has an @version entry:
					if let Some(version_value) = context.get(Keyword::Version.into()) {
						// 5.5.1) If the associated value is not 1.1, an invalid @version value has
						// been detected.
						if version_value.as_str() != Some("1.1") && version_value.as_f32() != Some(1.1) {
							return Err(ContextProcessingError::InvalidVersionValue)
						}
					}

					// 5.5.2) If processing mode is set to json-ld-1.0, a processing mode conflict
					// error has been detected.
					// TODO

					// 5.6) If context has an @import entry:
					if let Some(import_value) = context.get(Keyword::Import.into()) {
						// 5.6.1) If processing mode is json-ld-1.0, an invalid context entry error
						// has been detected.
						// TODO

						if let Some(import_value) = import_value.as_str() {
							// 5.6.3) Initialize import to the result of resolving the value of
							// @import.
							let import = if let Ok(iri_ref) = IriRef::new(import_value) {
								iri_ref.resolved(base_url.as_iri())
							} else {
								return Err(ContextProcessingError::InvalidImportValue)
							};

							// 5.6.4) Dereference import.
							let context_document = remote_contexts.load(import.as_iri()).await?;
							let import_context = context_document.context();

							// If the dereferenced document has no top-level map with an @context
							// entry, or if the value of @context is not a context definition
							// (i.e., it is not an map), an invalid remote context has been
							// detected and processing is aborted; otherwise, set import context
							// to the value of that entry.
							if let JsonValue::Object(loaded_context) = import_context {
								// If `import_context` has a @import entry, an invalid context entry
								// error has been detected and processing is aborted.
								if let Some(_) = loaded_context.get(Keyword::Import.into()) {
									return Err(ContextProcessingError::InvalidContextEntry);
								}

								// Set `context` to the result of merging context into
								// `import context`, replacing common entries with those from
								// `context`.
								panic!("TODO");
							} else {
								return Err(ContextProcessingError::InvalidRemoteContext)
							}
						} else {
							// 5.6.2) If the value of @import is not a string, an invalid
							// @import value error has been detected.
							return Err(ContextProcessingError::InvalidImportValue)
						}
					}

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
												if let Some(base_iri) = result.base_iri() {
													let resolved = value.resolved(base_iri);
													result.set_base_iri(Some(resolved.as_iri()))
												} else {
													return Err(ContextProcessingError::InvalidBaseIri)
												}
											}
										}
									} else {
										return Err(ContextProcessingError::InvalidBaseIri)
									}
								},
								_ => {
									return Err(ContextProcessingError::InvalidBaseIri)
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
								if let Some(vocab) = expansion::expand_iri(&result, value, true, false) {
									result.set_vocabulary(Some(vocab));
								} else {
									return Err(ContextProcessingError::InvalidVocabMapping)
								}
							},
							_ => {
								return Err(ContextProcessingError::InvalidVocabMapping)
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
							return Err(ContextProcessingError::InvalidDefaultLanguage)
						}
					}

					// 5.10) If context has a @direction entry:
					if let Some(value) = context.get(Keyword::Direction.into()) {
						// 5.10.1) If processing mode is json-ld-1.0, an invalid context entry error
						// has been detected and processing is aborted.
						// TODO

						if value.is_null() {
							// 5.10.3) If value is null, remove any base direction from result.
							result.set_default_base_direction(None);
						} else if let Some(str) = value.as_str() {
							let dir = match str {
								"ltr" => Direction::Ltr,
								"rtl" => Direction::Rtl,
								_ => return Err(ContextProcessingError::InvalidBaseDirection)
							};
							result.set_default_base_direction(Some(dir));
						} else {
							return Err(ContextProcessingError::InvalidBaseDirection)
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
					for (key, value) in context.iter() {
						if !is_keyword(key) {
							define(&mut result, context, key, &mut defined, Some(base_url.as_iri()), protected, false).await?;
						}
					}
				},
				// 5.3) An invalid local context error has been detected.
				_ => return Err(ContextProcessingError::InvalidLocalContext)
			}
		}
		panic!("TODO")
	}.boxed_local()
}

enum JsonObjectRef<'a> {
	Owned(JsonObject),
	Borrowed(&'a JsonObject)
}

impl<'a> Deref for JsonObjectRef<'a> {
	type Target = JsonObject;

	fn deref(&self) -> &JsonObject {
		match self {
			JsonObjectRef::Owned(obj) => &obj,
			JsonObjectRef::Borrowed(obj) => obj
		}
	}
}

// A partial term definition.
pub struct PartialTermDefinition<T: Id, C: ActiveContext<T>> {
	// IRI mapping, maybe undefined.
	pub value: Option<Key<T>>,

	// Prefix flag.
	pub prefix: bool,

	// Protected flag.
	pub protected: bool,

	// Reverse property flag.
	pub reverse_property: bool,

	// Optional type mapping.
	pub typ: Option<Key<T>>,

	// Optional language mapping.
	pub language: Option<String>,

	// Optional direction mapping.
	pub direction: Option<Direction>,

	// Optional context.
	pub context: Option<Box<dyn LocalContext<T, C>>>,

	// Optional nest value.
	pub nest: Option<String>,

	// Optional index mapping.
	pub index: Option<String>,

	// Container mapping.
	pub container: Container
}

impl<T: Id, C: ActiveContext<T>> PartialEq<TermDefinition<T, C>> for PartialTermDefinition<T, C> {
	fn eq(&self, other: &TermDefinition<T, C>) -> bool {
		// NOTE we ignore the `protected` flag.

		// FIXME `context` comparison!

		self.prefix == other.prefix &&
		self.reverse_property == other.reverse_property &&
		self.language == other.language &&
		self.direction == other.direction &&
		self.nest == other.nest &&
		self.index == other.index &&
		self.container == other.container &&
		*self.value.as_ref().unwrap() == other.value &&
		self.typ == other.typ
	}
}

impl<T: Id, C: ActiveContext<T>> Default for PartialTermDefinition<T, C> {
	fn default() -> PartialTermDefinition<T, C> {
		PartialTermDefinition {
			value: None,
			prefix: false,
			protected: false,
			reverse_property: false,
			typ: None,
			language: None,
			direction: None,
			context: None,
			nest: None,
			index: None,
			container: Container::new()
		}
	}
}

impl<T: Id, C: ActiveContext<T>> From<PartialTermDefinition<T, C>> for TermDefinition<T, C> {
	fn from(d: PartialTermDefinition<T, C>) -> TermDefinition<T, C> {
		TermDefinition {
			value: d.value.unwrap(),
			prefix: d.prefix,
			protected: d.protected,
			reverse_property: d.reverse_property,
			typ: d.typ,
			language: d.language,
			direction: d.direction,
			context: d.context,
			nest: d.nest,
			index: d.index,
			container: d.container
		}
	}
}

fn is_valid_type<T: Id>(t: &Key<T>) -> bool {
	match t {
		Key::Keyword(kw) => {
			match kw {
				Keyword::Id | Keyword::JSON | Keyword::None | Keyword::Vocab => true,
				_ => false
			}
		},
		Key::Id(_) => true
	}
}

fn is_gen_delim(c: char) -> bool {
	match c {
		':' | '/' | '?' | '#' | '[' | ']' | '@' => true,
		_ => false
	}
}

fn is_gen_delim_or_blank<T: Id>(t: &Key<T>) -> bool {
	match t {
		Key::Keyword(_) => false,
		Key::Id(id) => {
			if let Some(iri) = id.iri() {
				if let Some(c) = iri.as_str().chars().last() {
					is_gen_delim(c)
				} else {
					false
				}
			} else {
				true
			}
		}
	}
}

fn contains_nz(id: &str, c: char) -> bool {
	if let Some(i) = id.find(c) {
		i > 0
	} else {
		false
	}
}

// fn define<'a>(&mut self, env: &mut DefinitionEnvironment<'a>, term: &str, value: &JsonValue) -> Result<(), Self::Error> {

/// Follows the `https://www.w3.org/TR/json-ld11-api/#create-term-definition` algorithm.
/// Default value for `base_url` is `None`. Default values for `protected` and `override_protected` are `false`.
pub fn define<'a, T: Id, C: MutableActiveContext<T>>(active_context: &'a mut C, local_context: &'a JsonObject, term: &str, defined: &'a mut HashMap<String, bool>, base_url: Option<Iri>, protected: bool, override_protected: bool) -> LocalBoxFuture<'a, Result<(), ContextProcessingError>> {
	let term = term.to_string();
	let base_url = if let Some(base_url) = base_url {
		Some(IriBuf::from(base_url))
	} else {
		None
	};

	async move {
		match defined.get(term.as_str()) {
			// If defined contains the entry term and the associated value is true (indicating
			// that the term definition has already been created), return.
			Some(true) => Ok(()),
			// Otherwise, if the value is false, a cyclic IRI mapping error has been detected and processing is aborted.
			Some(false) => Err(ContextProcessingError::CyclicIriMapping),
			None => {
				// Initialize `value` to a copy of the value associated with the entry `term` in
				// `local_context`.
				if let Some(value) = local_context.get(term.as_str()) {
					// Set the value associated with defined's term entry to false.
					// This indicates that the term definition is now being created but is not yet
					// complete.
					defined.insert(term.to_string(), false);

					// If term is @type, ...
					if term.as_str() == "@type" {
						// ... and processing mode is json-ld-1.0, a keyword
						// redefinition error has been detected and processing is aborted.
						// TODO

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
									_ => return Err(ContextProcessingError::KeywordRedefinition)
								}
							}
						} else {
							return Err(ContextProcessingError::KeywordRedefinition)
						}
					}

					// Otherwise, since keywords cannot be overridden, term MUST NOT be a keyword and
					// a keyword redefinition error has been detected and processing is aborted.
					// If term has the form of a keyword (i.e., it matches the ABNF rule "@"1*ALPHA
					// from [RFC5234]), return; processors SHOULD generate a warning.
					if is_keyword_like(term.as_str()) {
						// TODO warning
						return Ok(())
					}

					// Initialize `previous_definition` to any existing term definition for `term` in
					// `active_context`, removing that term definition from active context.
					let previous_definition = active_context.set(term.as_str(), None);

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
							return Err(ContextProcessingError::InvalidTermDefinition)
						}
					};

					// Create a new term definition, `definition`, initializing `prefix` flag to
					// `false`, `protected` to `protected`, and `reverse_property` to `false`.
					let mut definition = PartialTermDefinition::<T, C>::default();
					definition.protected = protected;

					// If the @protected entry in value is true set the protected flag in
					// definition to true. If the value of @protected is not a boolean, an invalid
					// @protected value error has been detected and processing is aborted.
					// If processing mode is json-ld-1.0, an invalid term definition has been
					// detected and processing is aborted.

					// If the @protected entry in value is true set the protected flag in
					// definition to true.
					if let Some(protected_value) = value.get("@protected") {
						if let JsonValue::Boolean(b) = protected_value {
							if *b {
								definition.protected = true;
							}
						} else {
							// If the value of @protected is not a boolean, an invalid @protected
							// value error has been detected.
							return Err(ContextProcessingError::InvalidProtectedValue)

							// If processing mode is json-ld-1.0, an invalid term definition has
							// been detected and processing is aborted.
							// TODO
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
							let typ = expand_iri(active_context, typ, false, false, local_context, defined).await?;

							// If the expanded type is @json or @none, and processing mode is
							// json-ld-1.0, an invalid type mapping error has been detected and
							// processing is aborted.
							// TODO

							// Otherwise, if the expanded type is neither @id, nor @json, nor
							// @none, nor @vocab, nor an IRI, an invalid type mapping error has
							// been detected and processing is aborted.
							if !is_valid_type(&typ) {
								return Err(ContextProcessingError::InvalidTypeMapping)
							}

							// Set the type mapping for definition to type.
							definition.typ = Some(typ);
						} else {
							return Err(ContextProcessingError::InvalidTypeMapping)
						}
					}

					// If `value` contains the entry @reverse:
					if let Some(reverse_value) = value.get("@reverse") {
						// If `value` contains `@id` or `@nest`, entries, an invalid reverse
						// property error has been detected and processing is aborted.
						if value.get("@id").is_some() || value.get("@nest").is_some() {
							return Err(ContextProcessingError::InvalidReverseProperty)
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
							let mapping = expand_iri(active_context, reverse_value, false, false, local_context, defined).await?;
							if mapping.is_keyword() {
								return Err(ContextProcessingError::InvalidIriMapping)
							} else {
								definition.value = Some(mapping);
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
												_ => return Err(ContextProcessingError::InvalidReverseProperty)
											}
										} else {
											return Err(ContextProcessingError::InvalidReverseProperty)
										}
									},
									_ => return Err(ContextProcessingError::InvalidReverseProperty)
								};
							}

							// Set the `reverse_property` flag of `definition` to `true`.
							definition.reverse_property = true;

							// Set the term definition of `term` in `active_context` to
							// `definition` and the value associated with `defined`'s entry `term`
							// to `true` and return.
							active_context.set(term.as_str(), Some(definition.into()));
							defined.insert(term, true);
							return Ok(())
						} else {
							// If the value associated with the `@reverse` entry is not a string,
							// an invalid IRI mapping error has been detected and processing is
							// aborted.
							return Err(ContextProcessingError::InvalidIriMapping)
						}
					}

					// If `value` contains the entry `@id` and its value does not equal `term`:
					let id_value = value.get("@id");
					if id_value.is_some() && id_value.unwrap().as_str() != Some(term.as_str()) {
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
								definition.value = if let Ok(value) = expand_iri(active_context, id_value, false, false, local_context, defined).await {
									// if it equals `@context`, an invalid keyword alias error has
									// been detected and processing is aborted.
									if value == Key::Keyword(Keyword::Context) {
										return Err(ContextProcessingError::InvalidKeywordAlias)
									}

									Some(value)
								} else {
									// If the resulting IRI mapping is neither a keyword,
									// nor an IRI, nor a blank node identifier, an
									// invalid IRI mapping error has been detected and processing
									// is aborted;
									return Err(ContextProcessingError::InvalidIriMapping)
								};

								// If `term` contains a colon (:) anywhere but as the first or
								// last character of `term`, or if it contains a slash (/)
								// anywhere:
								if contains_nz(term.as_str(), ':') || term.contains('/') {
									// Set the value associated with `defined`'s `term` entry
									// to `true`.
									defined.insert(term.clone(), true);

									// If the result of IRI expanding `term` using
									// `local_context`, and `defined`, is not the same as the
									// IRI mapping of definition, an invalid IRI mapping error
									// has been detected and processing is aborted.
									let expanded_term = expand_iri(active_context, term.as_str(), false, false, local_context, defined).await?;
									if expanded_term != *definition.value.as_ref().unwrap() {
										return Err(ContextProcessingError::InvalidIriMapping)
									}
								}

								// If `term` contains neither a colon (:) nor a slash (/),
								// simple term is true, and if the IRI mapping of definition
								// is either an IRI ending with a gen-delim character,
								// or a blank node identifier, set the `prefix` flag in
								// `definition` to true.
								if !term.contains(':') && term.contains('/') && simple_term && is_gen_delim_or_blank(definition.value.as_ref().unwrap()) {
									definition.prefix = true;
								}
							} else {
								// If the value associated with the `@id` entry is not a
								// string, an invalid IRI mapping error has been detected and
								// processing is aborted.
								return Err(ContextProcessingError::InvalidIriMapping)
							}
						}
					} else if contains_nz(term.as_str(), ':') {
						// Otherwise if the `term` contains a colon (:) anywhere after the first
						// character:
						let i = term.find(':').unwrap();
						let (prefix, suffix) = term.split_at(i);

						// If `term` is a compact IRI with a prefix that is an entry in local
						// context a dependency has been found.
						// Use this algorithm recursively passing `active_context`,
						// `local_context`, the prefix as term, and `defined`.
						define(active_context, local_context, prefix, defined, None, false, false).await?;

						// If `term`'s prefix has a term definition in `active_context`, set the
						// IRI mapping of `definition` to the result of concatenating the value
						// associated with the prefix's IRI mapping and the term's suffix.
						if let Some(prefix_definition) = active_context.get(prefix) {
							if let Some(prefix_iri) = prefix_definition.value.iri() {
								let mut result = prefix_iri.as_str().to_string();
								result.push_str(suffix);
								if let Ok(iri) = Iri::new(result.as_str()) {
									definition.value = Some(Key::Id(Id::from_iri(iri)))
								} else {
									return Err(ContextProcessingError::InvalidIriMapping)
								}
							} else {
								return Err(ContextProcessingError::InvalidIriMapping)
							}
						} else {
							// Otherwise, `term` is an IRI or blank node identifier.
							// Set the IRI mapping of `definition` to `term`.
							if prefix == "_" { // blank node
								definition.value = Some(Key::Id(Id::from_blank_id(suffix)))
							} else {
								if let Ok(iri) = Iri::new(term.as_str()) {
									definition.value = Some(Key::Id(Id::from_iri(iri)))
								} else {
									return Err(ContextProcessingError::InvalidIriMapping)
								}
							}
						}
					} else if term.contains('/') {
						// Term is a relative IRI reference.
						/// Set the IRI mapping of definition to the result of IRI expanding
						// term.
						match expansion::expand_iri(active_context, term.as_str(), false, false) {
							Some(Key::Id(id)) => {
								definition.value = Some(Key::Id(id))
							},
							// If the resulting IRI mapping is not an IRI, an invalid IRI mapping
							// error has been detected and processing is aborted.
							_ => return Err(ContextProcessingError::InvalidIriMapping)
						}
					} else if term == "@type" {
						// Otherwise, if `term` is ``@type`, set the IRI mapping of definition to
						// `@type`.
						definition.value = Some(Key::Keyword(Keyword::Type))
					} else if let Some(vocabulary) = active_context.vocabulary() {
						// Otherwise, if `active_context` has a vocabulary mapping, the IRI mapping
						// of `definition` is set to the result of concatenating the value
						// associated with the vocabulary mapping and `term`.
						// If it does not have a vocabulary mapping, an invalid IRI mapping error
						// been detected and processing is aborted.
						if let Some(vocabulary_iri) = vocabulary.iri() {
							let mut result = vocabulary_iri.as_str().to_string();
							result.push_str(term.as_str());
							if let Ok(iri) = Iri::new(result.as_str()) {
								definition.value = Some(Key::Id(Id::from_iri(iri)))
							} else {
								return Err(ContextProcessingError::InvalidIriMapping)
							}
						} else {
							return Err(ContextProcessingError::InvalidIriMapping)
						}
					}

					// If value contains the entry @container:
					if let Some(container_value) = value.get("@container") {
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
											return Err(ContextProcessingError::InvalidContainerMapping)
										}
									},
									Err(_) => return Err(ContextProcessingError::InvalidContainerMapping)
								}
							} else {
								return Err(ContextProcessingError::InvalidContainerMapping)
							}
						}

						// If the container value is @graph, @id, or @type, or is otherwise not a
						// string, generate an invalid container mapping error and abort processing
						// if processing mode is json-ld-1.0.
						// TODO

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
									Key::Keyword(Keyword::Id) | Key::Keyword(Keyword::Vocab) => (),
									_ => return Err(ContextProcessingError::InvalidTypeMapping)
								}
							} else {
								// If type mapping in definition is undefined, set it to @id.
								definition.typ = Some(Key::Keyword(Keyword::Id))
							}
						}
					}

					// If value contains the entry @index:
					if let Some(index_value) = value.get("@index") {
						// If processing mode is json-ld-1.0 or container mapping does not include
						// `@index`, an invalid term definition has been detected and processing
						// is aborted.
						// TODO json-ld-1.0
						if !definition.container.contains(ContainerType::Index) {
							return Err(ContextProcessingError::InvalidTermDefinition)
						}

						// Initialize `index` to the value associated with the `@index` entry,
						// which MUST be a string expanding to an IRI.
						// Otherwise, an invalid term definition has been detected and processing
						// is aborted.
						if let Some(index) = index_value.as_str() {
							if expansion::expand_iri(active_context, index, false, false).is_none() {
								return Err(ContextProcessingError::InvalidTermDefinition)
							}

							definition.index = Some(index.to_string())
						} else {
							return Err(ContextProcessingError::InvalidTermDefinition)
						}
					}

					// If `value` contains the entry `@context`:
					if let Some(context) = value.get("@context") {
						// If processing mode is json-ld-1.0, an invalid term definition has been
						// detected and processing is aborted.
						// TODO

						// Initialize `context` to the value associated with the @context entry,
						// which is treated as a local context.
						// done.

						// Invoke the Context Processing algorithm using the `active_context`,
						// `context` as local context, `base_url`, and `true` for override
						// protected.
						// if let Err(_) = process_context(active_context, context, base_url, Arc::new(RemoteContexts::new()), true, true).await {
						// 	// If any error is detected, an invalid scoped context error has been
						// 	// detected and processing is aborted.
						// 	return Err(ContextProcessingError::InvalidScopedContext)
						// }

						// Set the local context of definition to context, and base URL to base URL.
						definition.context = Some(Box::new(context.clone()));
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
							definition.language = match language_value {
								JsonValue::Null => None,
								JsonValue::String(_) | JsonValue::Short(_) => {
									// TODO lang tags
									Some(language_value.as_str().unwrap().to_string())
								},
								_ => {
									return Err(ContextProcessingError::InvalidLanguageMapping)
								}
							};
						}

						// If `value` contains the entry `@direction` and does not contain the
						// entry `@type`:
						if let Some(direction_value) = value.get("@direction") {
							// Initialize `direction` to the value associated with the `@direction`
							// entry, which MUST be either null, "ltr", or "rtl".
							definition.direction = match direction_value.as_str() {
								Some("ltr") => Some(Direction::Ltr),
								Some("rtl") => Some(Direction::Rtl),
								_ => {
									if direction_value.is_null() {
										None
									} else {
										// Otherwise, an invalid base direction error has been
										// detected and processing is aborted.
										return Err(ContextProcessingError::InvalidBaseDirection)
									}
								}
							};
						}
					}

					// If value contains the entry @nest:
					if let Some(nest_value) = value.get("@nest") {
						// If processing mode is json-ld-1.0, an invalid term definition has been
						// detected and processing is aborted.
						// TODO json-ld-1.0

						// Initialize `nest` value in `definition` to the value associated with the
						// `@nest` entry, which MUST be a string and MUST NOT be a keyword other
						// than @nest.
						if let Some(nest_value) = nest_value.as_str() {
							if is_keyword(nest_value) && nest_value != "@nest" {
								return Err(ContextProcessingError::InvalidNestValue)
							}

							definition.nest = Some(nest_value.to_string());
						} else {
							// Otherwise, an invalid @nest value error has been detected and
							// processing is aborted.
							return Err(ContextProcessingError::InvalidNestValue)
						}
					}

					// If value contains the entry @prefix:
					if let Some(prefix_value) = value.get("@prefix") {
						// If processing mode is json-ld-1.0, or if `term` contains a colon (:) or
						// slash (/), an invalid term definition has been detected and processing
						// is aborted.
						if term.contains(':') || term.contains('/') {
							// TODO json-ld-1.0
							return Err(ContextProcessingError::InvalidTermDefinition)
						}

						// Set the `prefix` flag to the value associated with the @prefix entry,
						// which MUST be a boolean.
						// Otherwise, an invalid @prefix value error has been detected and
						// processing is aborted.
						if let Some(prefix) = prefix_value.as_bool() {
							definition.prefix = prefix
						} else {
							return Err(ContextProcessingError::InvalidPrefixValue)
						}

						// If the `prefix` flag of `definition` is set to `true`, and its IRI
						// mapping is a keyword, an invalid term definition has been detected and
						// processing is aborted.
						if definition.prefix && definition.value.as_ref().unwrap().is_keyword() {
							return Err(ContextProcessingError::InvalidTermDefinition)
						}
					}

					// If the value contains any entry other than @id, @reverse,
					// @container, @context, @language, @nest, @prefix, or @type, an
					// invalid term definition error has been detected.
					for (key, _) in value.iter() {
						match key {
							"@id" | "@reverse" | "@container" | "@context" | "@language" | "@nest" | "@prefix" | "@type" => (),
							_ => return Err(ContextProcessingError::InvalidTermDefinition)
						}
					}

					// If override protected is false and previous_definition exists and is protected;
					if !override_protected {
						if let Some(previous_definition) = previous_definition {
							if previous_definition.protected {
								// If `definition` is not the same as `previous_definition`
								// (other than the value of protected), a protected term
								// redefinition error has been detected, and processing is aborted.
								if definition != previous_definition {
									return Err(ContextProcessingError::ProtectedTermRedefinition)
								}

								// Set `definition` to `previous definition` to retain the value of
								// protected.
								definition.protected = true;
							}
						}
					}

					// Set the term definition of `term` in `active_context` to `definition` and
					// set the value associated with `defined`'s entry term to true.
					active_context.set(term.as_str(), Some(definition.into()));
					defined.insert(term.clone(), true);
				}

				// if the term is not in `local_context`.
				Ok(())
			}
		}
	}.boxed_local()
}

/// Default values for `document_relative` and `vocab` should be `false`.
pub fn expand_iri<'a, T: Id, C: MutableActiveContext<T>>(active_context: &'a mut C, value: &str, document_relative: bool, vocab: bool, local_context: &'a JsonObject, defined: &'a mut HashMap<String, bool>) -> impl 'a + Future<Output = Result<Key<T>, ContextProcessingError>> {
	let value = value.to_string();
	async move {
		if let Ok(keyword) = Keyword::try_from(value.as_ref()) {
			Ok(Key::Keyword(keyword))
		} else {
			// If value has the form of a keyword, a processor SHOULD generate a warning and return
			// null.
			// TODO

			// If `local_context` is not null, it contains an entry with a key that equals value, and the
			// value of the entry for value in defined is not true, invoke the Create Term Definition
			// algorithm, passing active context, local context, value as term, and defined. This will
			// ensure that a term definition is created for value in active context during Context
			// Processing.
			define(active_context, local_context, value.as_ref(), defined, None, false, false).await?;

			if let Some(term_definition) = active_context.get(value.as_ref()) {
				// If active context has a term definition for value, and the associated IRI mapping
				// is a keyword, return that keyword.

				// If vocab is true and the active context has a term definition for value, return the
				// associated IRI mapping.
				if term_definition.value.is_keyword() || vocab {
					return Ok(term_definition.value.clone())
				}
			}

			// If value contains a colon (:) anywhere after the first character, it is either an IRI,
			// a compact IRI, or a blank node identifier:
			if let Some(index) = value.find(':') {
				if index > 0 {
					// Split value into a prefix and suffix at the first occurrence of a colon (:).
					let (prefix, suffix) = value.split_at(index);

					// If prefix is underscore (_) or suffix begins with double-forward-slash (//),
					// return value as it is already an IRI or a blank node identifier.
					if prefix == "_" {
						return Ok(Key::Id(T::from_blank_id(suffix)))
					}

					if suffix.starts_with("//") {
						if let Ok(iri) = Iri::new(value.as_ref() as &str) {
							return Ok(Key::Id(T::from_iri(iri)))
						} else {
							return Err(ContextProcessingError::InvalidIri)
						}
					}

					// If local context is not null, it contains a `prefix` entry, and the value of the
					// prefix entry in defined is not true, invoke the Create Term Definition
					// algorithm, passing active context, local context, prefix as term, and defined.
					// This will ensure that a term definition is created for prefix in active context
					// during Context Processing.
					define(active_context, local_context, prefix, defined, None, false, false).await?;

					// If active context contains a term definition for prefix having a non-null IRI
					// mapping and the prefix flag of the term definition is true, return the result
					// of concatenating the IRI mapping associated with prefix and suffix.
					if let Some(term_definition) = active_context.get(prefix) {
						if term_definition.prefix {
							if let Some(iri) = term_definition.value.iri() {
								let mut result = iri.as_str().to_string();
								result.push_str(suffix);

								if let Ok(result) = Iri::new(&result) {
									return Ok(Key::Id(T::from_iri(result)))
								} else {
									return Err(ContextProcessingError::InvalidIri)
								}
							}
						}
					}

					// If value has the form of an IRI, return value.
					if let Ok(result) = Iri::new(value.as_ref() as &str) {
						return Ok(Key::Id(T::from_iri(result)))
					}
				}
			}

			// If vocab is true, and active context has a vocabulary mapping, return the result of
			// concatenating the vocabulary mapping with value.
			if vocab {
				if let Some(vocabulary) = active_context.vocabulary() {
					if let Key::Id(id) = vocabulary {
						if let Some(iri) = id.iri() {
							let mut result = iri.as_str().to_string();
							result.push_str(value.as_ref());

							if let Ok(result) = Iri::new(&result) {
								return Ok(Key::Id(T::from_iri(result)))
							} else {
								return Err(ContextProcessingError::InvalidIri)
							}
						} else {
							return Err(ContextProcessingError::InvalidIri)
						}
					} else {
						return Err(ContextProcessingError::InvalidIri)
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
					if let Some(base_iri) = active_context.base_iri() {
						let value = iri_ref.resolved(base_iri);
						return Ok(Key::Id(T::from_iri(value.as_iri())))
					} else {
						return Err(ContextProcessingError::InvalidBaseIri)
					}
				} else {
					return Err(ContextProcessingError::InvalidIriRef)
				}
			}

			// Return value as is.
			Err(ContextProcessingError::InvalidIri) // FIXME better error
		}
	}
}
