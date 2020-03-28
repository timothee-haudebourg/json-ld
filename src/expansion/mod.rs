mod iri;
mod value;

use std::ops::Deref;
use mown::Mown;
use futures::future::{LocalBoxFuture, FutureExt};
use iref::{Iri, IriBuf};
use json::JsonValue;
use std::collections::HashMap;
use crate::{Keyword, Direction, Container, ContainerType, as_array, Id, Key, Node, Value, Object};
use crate::context::{ActiveContext, MutableActiveContext, LocalContext, ContextLoader, ContextProcessingError};
pub use iri::*;
pub use value::*;

#[derive(Clone, Copy, Debug)]
pub enum ExpansionError {
	ContextProcessing(ContextProcessingError),
	InvalidIri,
	InvalidReverseProperty,
	CollidingKeywords,
	InvalidIdValue,
	InvalidTypeValue
}

impl From<ContextProcessingError> for ExpansionError {
	fn from(e: ContextProcessingError) -> ExpansionError {
		ExpansionError::ContextProcessing(e)
	}
}

use std::cmp::{Ord, Ordering};

// #[derive(PartialEq, Eq)]
// enum EntryKey<'a, T: Id> {
// 	Normal(&'a str),
// 	Expanded(Key<T>)
// }
//
// impl<'a, T: Id> PartialOrd for EntryKey<'a, T> {
// 	fn partial_cmp(&self, other: &EntryKey<'a, T>) -> Option<Ordering> {
// 		use EntryKey::*;
// 		match (self, other) {
// 			(Normal(a), Normal(b)) => a.partial_cmp(b),
// 			(Expanded(a), Expanded(b)) => a.partial_cmp(b),
// 			(_, _) => None
// 		}
// 	}
// }
//
// impl<'a, T: Id> Ord for Entry<'a, T> {
// 	fn cmp(&self, other: &EntryKey<'a, T>) -> Ordering {
// 		self.0.partial_cmp(&other.0).unwrap()
// 	}
// }

#[derive(PartialEq, Eq)]
struct Entry<'a, T>(T, &'a JsonValue);

impl<'a, T: PartialOrd> PartialOrd for Entry<'a, T> {
	fn partial_cmp(&self, other: &Entry<'a, T>) -> Option<Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<'a, T: Ord> Ord for Entry<'a, T> {
	fn cmp(&self, other: &Entry<'a, T>) -> Ordering {
		self.0.cmp(&other.0)
	}
}

/// https://www.w3.org/TR/json-ld11-api/#expansion-algorithm
/// The default specified value for `ordered` and `from_map` is `false`.
pub fn expand<'a, T: Id, C: MutableActiveContext<T>, L: ContextLoader<C::LocalContext>>(active_context: &'a C, active_property: Option<&'a str>, element: &'a JsonValue, base_url: Option<Iri>, loader: &'a mut L, ordered: bool, from_map: bool) -> LocalBoxFuture<'a, Result<Option<Vec<Object<T>>>, ExpansionError>> where C::LocalContext: From<JsonValue> {
	let base_url = base_url.map(|url| IriBuf::from(url));

	async move {
		let base_url = base_url.as_ref().map(|url| url.as_iri());

		// If `element` is null, return null.
		if element.is_null() {
			return Ok(None)
		}

		let active_property_definition = active_context.get_opt(active_property);

		// // If `active_property` is `@default`, initialize the `frame_expansion` flag to `false`.
		// if active_property == Some("@default") {
		// 	frame_expansion = false;
		// }

		// If `active_property` has a term definition in `active_context` with a local context,
		// initialize property-scoped context to that local context.
		let property_scoped_context = if let Some(definition) = active_property_definition {
			definition.context.as_ref()
		} else {
			None
		};

		match element {
			JsonValue::Null => unreachable!(),
			JsonValue::Array(element) => {
				// Initialize an empty array, result.
				let mut result = Vec::new();

				// For each item in element:
				for item in element {
					// Initialize `expanded_item` to the result of using this algorithm
					// recursively, passing `active_context`, `active_property`, `item` as element,
					// `base_url`, the `frame_expansion`, `ordered`, and `from_map` flags.
					let expanded_items: Option<Vec<Object<T>>> = expand(active_context, active_property, item, base_url, loader, ordered, from_map).await?;

					// If the container mapping of `active_property` includes `@list`, and
					// `expanded_item` is an array, set `expanded_item` to a new map containing
					// the entry `@list` where the value is the original `expanded_item`.
					if let Some(expanded_items) = expanded_items {
						let is_list = if let Some(definition) = active_property_definition {
							definition.container.contains(ContainerType::List)
						} else {
							false
						};

						if is_list {
							result.push(Object::Value(Value::List(expanded_items)));
						} else {
							// If `expanded_item` is an array, append each of its items to result.
							for item in expanded_items {
								result.push(item)
							}
						}
					}
				}

				// Return result.
				return Ok(Some(result))
			},

			JsonValue::Object(element) => {
				// We will need to consider expanded keys, and maybe ordered keys.
				let mut entries: Vec<Entry<&'a str>> = Vec::with_capacity(element.len());
				for (key, value) in element.iter() {
					entries.push(Entry(key, value));
				}

				if ordered {
					entries.sort()
				}

				let mut context_entry = None;
				let mut value_entry = None;
				let mut id_entry = None;

				let mut expanded_entries = Vec::with_capacity(element.len());
				let mut type_entries = Vec::new();
				for Entry(key, value) in entries.iter() {
					if let Some(expanded_key) = expand_iri(active_context, key, false, false) {
						match expanded_key {
							Key::Keyword(Keyword::Context) => {
								context_entry = Some(value)
							},
							Key::Keyword(Keyword::Value) => {
								value_entry = Some(value)
							},
							Key::Keyword(Keyword::Id) => {
								id_entry = Some(value)
							},
							Key::Keyword(Keyword::Type) => {
								type_entries.push(Entry(key, value));
							},
							_ => ()
						}

						expanded_entries.push(Entry(expanded_key, value))
					}
				}

				type_entries.sort();

				// Otherwise element is a map.
				// If `active_context` has a `previous_context`, the active context is not
				// propagated.
				let mut active_context = Mown::Borrowed(active_context);
				if let Some(previous_context) = active_context.previous_context() {
					// If `from_map` is undefined or false, and `element` does not contain an entry
					// expanding to `@value`, and `element` does not consist of a single entry
					// expanding to `@id` (where entries are IRI expanded), set active context to
					// previous context from active context, as the scope of a term-scoped context
					// does not apply when processing new Object objects.
					if !from_map && value_entry.is_none() && !(element.len() == 1 && id_entry.is_some()) {
						active_context = Mown::Owned(previous_context.clone())
					}
				}

				// If `property_scoped_context` is defined, set `active_context` to the result of
				// the Context Processing algorithm, passing `active_context`,
				// `property_scoped_context` as `local_context`, `base_url` from the term
				// definition for `active_property`, in `active_context` and `true` for
				// `override_protected`.
				if let Some(property_scoped_context) = property_scoped_context {
					active_context = Mown::Owned(property_scoped_context.process(active_context.as_ref(), loader, base_url, false, true, true).await?);
				}

				// If `element` contains the entry `@context`, set `active_context` to the result
				// of the Context Processing algorithm, passing `active_context`, the value of the
				// `@context` entry as `local_context` and `base_url`.
				if let Some(local_context) = context_entry {
					active_context = Mown::Owned(local_context.process(active_context.as_ref(), loader, base_url, false, false, true).await?);
				}

				// Initialize `type_scoped_context` to `active_context`.
				// This is used for expanding values that may be relevant to any previous
				// type-scoped context.
				let type_scoped_context = active_context.as_ref();
				let mut active_context = Mown::Borrowed(active_context.as_ref());

				// For each `key` and `value` in `element` ordered lexicographically by key where
				// key IRI expands to @type:
				for Entry(_, value) in &type_entries {
					// Convert `value` into an array, if necessary.
					let value = as_array(value);

					// For each `term` which is a value of `value` ordered lexicographically,
					let mut sorted_value = Vec::with_capacity(value.len());
					for term in value {
						if let Some(term) = term.as_str() {
							sorted_value.push(term);
						}
					}
					sorted_value.sort();

					// if `term` is a string, and `term`'s term definition in `type_scoped_context`
					// has a `local_context`,
					for term in sorted_value {
						if let Some(term_definition) = type_scoped_context.get(term) {
							if let Some(local_context) = &term_definition.context {
								// set `active_context` to the result of
								// Context Processing algorithm, passing `active_context`, the value of the
								// `term`'s local context as `local_context`, `base_url` from the term
								// definition for value in `active_context`, and `false` for `propagate`.
								let base_url = term_definition.base_url.as_ref().map(|url| url.as_iri());
								active_context = Mown::Owned(local_context.process(active_context.as_ref(), loader, base_url, false, false, false).await?);
							}
						}
					}
				}

				// Initialize two empty maps, `result` and `nests`.
				let mut result: Node<T> = Node::new();
				// let mut nests = HashMap::new();

				// Initialize `input_type` to expansion of the last value of the first entry in
				// `element` expanding to `@type` (if any), ordering entries lexicographically by
				// key.
				// Both the key and value of the matched entry are IRI expanded.
				let input_type = if let Some(Entry(_, value)) = type_entries.first() {
					if let Some(input_type) = as_array(value).last() {
						if let Some(input_type) = input_type.as_str() {
							expand_iri(active_context.as_ref(), input_type, false, false)
						} else {
							None
						}
					} else {
						None
					}
				} else {
					None
				};

				// For each `key` and `value` in `element`, ordered lexicographically by key
				// if `ordered` is `true`:
				for Entry(key, value) in expanded_entries {
					println!("analyszing {} = {}", key, value);
					match key {
						// If key is @context, continue to the next key.
						Key::Keyword(Keyword::Context) => (),
						// Initialize `expanded_property` to the result of IRI expanding `key`.

						// If `expanded_property` is `null` or it neither contains a colon (:)
						// nor it is a keyword, drop key by continuing to the next key.
						// (already done)

						// If `expanded_property` is a keyword:
						Key::Keyword(expanded_property) => {
							// If `active_property` equals `@reverse`, an invalid reverse property
							// map error has been detected and processing is aborted.
							if active_property == Some("@reverse") {
								return Err(ExpansionError::InvalidReverseProperty)
							}

							// If `result` already has an `expanded_property` entry, other than
							// `@included` or `@type` (unless processing mode is json-ld-1.0), a
							// colliding keywords error has been detected and processing is
							// aborted.
							if let Some(expanded_property) = result.expanded_property.as_ref() {
								if *expanded_property != Key::Keyword(Keyword::Included) && *expanded_property != Key::Keyword(Keyword::Type) {
									// TODO json-ld-1.0
									return Err(ExpansionError::CollidingKeywords)
								}
							}

							match expanded_property {
								// If `expanded_property` is @id:
								Keyword::Id => {
									// If `value` is not a string, an invalid @id value error has
									// been detected and processing is aborted.
									if let Some(value) = value.as_str() {
										// Otherwise, set `expanded_value` to the result of IRI
										// expanding value using true for document relative and
										// false for vocab.
										if let Some(expanded_value) = expand_iri(active_context.as_ref(), value, true, false) {
											result.id = Some(expanded_value);
										}
									} else {
										return Err(ExpansionError::InvalidIdValue)
									}
								},
								// If expanded property is @type:
								Keyword::Type => {
									// If value is neither a string nor an array of strings, an
									// invalid type value error has been detected and processing
									// is aborted.
									let value = as_array(value);
									// Set `expanded_value` to the result of IRI expanding each
									// of its values using `type_scoped_context` for active
									// context, and true for document relative.
									for ty in value {
										if let Some(ty) = ty.as_str() {
											info!("expand type {}", ty);
											if let Some(expanded_ty) = expand_iri(type_scoped_context, ty, true, false) {
												result.types.push(expanded_ty);
											}
										} else {
											return Err(ExpansionError::InvalidTypeValue)
										}
									}
								},
								// If expanded property is @graph
								Keyword::Graph => {
									// Set `expanded_value` to the result of using this algorithm
									// recursively passing `active_context`, `@graph` for active
									// property, `value` for element, `base_url`, and the
									// `frame_expansion` and `ordered` flags, ensuring that
									// `expanded_value` is an array of one or more maps.
									if let Some(expanded_value) = expand(active_context.as_ref(), Some("@graph"), value, base_url, loader, ordered, false).await? {
										result.graph = Some(expanded_value)
									}
								},
								// If expanded property is @included:
								Keyword::Included => {
									// If processing mode is json-ld-1.0, continue with the next
									// key from element.
									// TODO processing mode

									// Set `expanded_value` to the result of using this algorithm
									// recursively passing `active_context`, `active_property`,
									// `value` for element, `base_url`, and the `frame_expansion`
									// and `ordered` flags, ensuring that the result is an array.
									if let Some(expanded_value) = expand(active_context.as_ref(), active_property, value, base_url, loader, ordered, false).await? {
										result.included = Some(expanded_value)
									}
								},
								// If expanded property is @value:
								Keyword::Value => {
									panic!("TODO");
									// If `input_type` is `@json`, set expanded value to value.
									// if input_type == Some(Key::Keyword(Keyword::JSON)) {
									// 	// If processing mode is json-ld-1.0, an invalid value object
									// 	// value error has been detected and processing is aborted.
									// 	// TODO processing mode.
									//
									// 	panic!("TODO")
									// } else {
									// 	// Otherwise, if value is not a scalar or null, an invalid
									// 	// value object value error has been detected and
									// 	// processing is aborted.
									//
									// 	// ...
									// }
								},
								// If expanded property is @language:
								Keyword::Language => {
									panic!("TODO")
								},
								// If expanded property is @direction:
								Keyword::Direction => {
									panic!("TODO")
								},
								// If expanded property is @index:
								Keyword::Index => {
									panic!("TODO")
								},
								// If expanded property is @list:
								Keyword::List => {
									panic!("TODO")
								},
								// If expanded property is @set
								Keyword::Set => {
									panic!("TODO")
								},
								// If expanded property is @reverse:
								Keyword::Reverse => {
									panic!("TODO")
								},
								// If expanded property is @nest
								Keyword::Nest => {
									panic!("TODO")
								},
								// When the frameExpansion flag is set, if expanded property is any
								// other framing keyword (@default, @embed, @explicit,
								// @omitDefault, or @requireAll)
								// NOTE we don't handle frame expansion here.
								_ => ()
							}
						},

						_ => {
							let mut container_mapping = Mown::Owned(Container::new());

							if let Some(key_iri) = key.iri() {
								let mut is_reverse_property = false;
								if let Some(key_definition) = active_context.get(key_iri.as_str()) {
									is_reverse_property = key_definition.reverse_property;

									// Initialize container mapping to key's container mapping in active context.
									container_mapping = Mown::Borrowed(&key_definition.container);

									// If key's term definition in `active_context` has a type mapping of `@json`,
									// set expanded value to a new map,
									// set the entry `@value` to `value`, and set the entry `@type` to `@json`.
									if key_definition.typ == Some(Key::Keyword(Keyword::JSON)) {
										panic!("TODO")
									}
								}

								let expanded_value;

								// Otherwise, if container mapping includes @language and value is a map then
								// value is expanded from a language map as follows:
								if container_mapping.contains(ContainerType::Language) {
									panic!("TODO")
								} else if container_mapping.contains(ContainerType::Index) || container_mapping.contains(ContainerType::Type) || container_mapping.contains(ContainerType::Id) {
									// Otherwise, if container mapping includes @index, @type, or @id and value
									// is a map then value is expanded from an map as follows:
									panic!("TODO")
								} else {
									// Otherwise, initialize expanded value to the result of using this
									// algorithm recursively, passing active context, key for active property,
									// value for element, base URL, and the frameExpansion and ordered flags.
									expanded_value = expand(active_context.as_ref(), Some(key_iri.as_str()), value, base_url, loader, ordered, false).await?;
								}

								if let Some(expanded_value) = expanded_value {
									// If container mapping includes @list and expanded value is
									// not already a list object, convert expanded value to a list
									// object by first setting it to an array containing only
									// expanded value if it is not already an array, and then by
									// setting it to a map containing the key-value pair
									// @list-expanded value.
									if container_mapping.contains(ContainerType::List) {
										panic!("TODO")
									}

									// If container mapping includes @graph, and includes neither
									// @id nor @index, convert expanded value into an array, if
									// necessary, then convert each value ev in expanded value
									// into a graph object:
									if container_mapping.contains(ContainerType::Graph) {
										panic!("TODO")
									}

									// If the term definition associated to key indicates that it
									// is a reverse property:
									if is_reverse_property {
										panic!("TODO")
									} else {
										// Otherwise, key is not a reverse property use add value
										// to add expanded value to the expanded property entry in
										// result using true for as array.
										panic!("TODO")
									}
								}
							}
						}
					}
				}

				// For each key nesting-key in nests, ordered lexicographically
				// if ordered is true:
				// FIXME TODO

				// If result contains the entry @value:
				// FIXME TODO

				// Otherwise, if result contains the entry @type and its
				// associated value is not an array, set it to an array
				// containing only the associated value.
				// FIXME TODO

				// Otherwise, if result contains the entry @set or @list:
				// FIXME TODO

				// If result is a map that contains only the entry @language,
				// return null.
				// FIXME TODO

				// If active property is null or @graph, drop free-floating
				// values as follows:
				// FIXME TODO

				return Ok(Some(vec![Object::Node(result)]))
			},

			_ => {
				// If element is a scalar (bool, int, string, null),
				// If `active_property` is `null` or `@graph`, drop the free-floating scalar by
				// returning null.
				if active_property.is_none() || active_property == Some("@graph") {
					return Ok(None)
				}

				// If `property_scoped_context` is defined, set `active_context` to the result of the
				// Context Processing algorithm, passing `active_context`, `property_scoped_context` as
				// local context, and `base_url` from the term definition for `active_property` in
				// `active context`.
				let active_context = if let Some(property_scoped_context) = property_scoped_context {
					// FIXME it is unclear what we should use as `base_url` if there is no term definition for `active_context`.
					let base_url = if let Some(definition) = active_context.get_opt(active_property) {
						if let Some(base_url) = &definition.base_url {
							Some(base_url.as_iri())
						} else {
							None
						}
					} else {
						None
					};

					let result = property_scoped_context.process(active_context, loader, base_url, false, false, true).await?;
					Mown::Owned(result)
				} else {
					Mown::Borrowed(active_context)
				};

				// Return the result of the Value Expansion algorithm, passing the `active_context`,
				// `active_property`, and `element` as value.
				let mut result = Vec::new();
				result.push(Object::Value(expand_value(active_context.as_ref(), active_property, element)?));
				return Ok(Some(result))
			}
		}
	}.boxed_local()
}
