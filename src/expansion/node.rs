use std::collections::HashSet;
use mown::Mown;
use iref::Iri;
use json::JsonValue;
use crate::{Keyword, Container, ContainerType, as_array, Id, Key, Property, Node, Value, Literal, Object, ObjectData};
use crate::context::{MutableActiveContext, LocalContext, ContextLoader};

use super::{ExpansionError, Entry, expand_element, expand_literal, expand_iri, filter_top_level_item};

pub async fn expand_node<T: Id, C: MutableActiveContext<T>, L: ContextLoader<C::LocalContext>>(active_context: &C, type_scoped_context: &C, active_property: Option<&str>, expanded_entries: Vec<Entry<'_, (&str, Key<T>)>>, base_url: Option<Iri<'_>>, loader: &mut L, ordered: bool) -> Result<Option<(Node<T>, ObjectData)>, ExpansionError> where C::LocalContext: From<JsonValue> {
	// Initialize two empty maps, `result` and `nests`.
	let mut result: Node<T> = Node::new();
	let mut result_data = ObjectData::new();
	// let mut nests = HashMap::new();
	// let len = expanded_entries.len();

	// For each `key` and `value` in `element`, ordered lexicographically by key
	// if `ordered` is `true`:
	for Entry((key, expanded_key), value) in expanded_entries {
		match expanded_key {
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
							if let Ok(expanded_value) = expand_iri(active_context, value, true, false) {
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
								if let Ok(expanded_ty) = expand_iri(type_scoped_context, ty, true, true) {
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
						let expanded_value = expand_element(active_context, Some("@graph"), value, base_url, loader, ordered, false).await?;
						if !expanded_value.is_empty() {
							result.graph = Some(expanded_value.into_iter().filter(filter_top_level_item).collect());
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
						let expanded_value = expand_element(active_context, active_property, value, base_url, loader, ordered, false).await?;
						if !expanded_value.is_empty() {
							result.included = Some(expanded_value.into_iter().collect());
						}
					},
					// // If expanded property is @language:
					// Keyword::Language => {
					// 	// If result is a map that contains only the entry
					// 	// @language, return null.
					// 	// if len == 1 {
					// 	// 	return Ok(Vec::new())
					// 	// }
					// },
					// If expanded property is @direction:
					Keyword::Direction => {
						panic!("TODO direction")
					},
					// If expanded property is @index:
					Keyword::Index => {
						if let Some(value) = value.as_str() {
							result_data.index = Some(value.to_string())
						} else {
							// If value is not a string, an invalid @index value
							// error has been detected and processing is aborted.
							return Err(ExpansionError::InvalidIndexValue)
						}
					},
					// // If expanded property is @list:
					// Keyword::List => {
					// 	// // If active property is null or @graph, continue with the
					// 	// // next key from element to remove the free-floating list.
					// 	// if active_property.is_none() || active_property == Some("@graph") {
					// 	// 	continue
					// 	// }
					// },
					// // If expanded property is @set
					// Keyword::Set => {
					// 	panic!("TODO set")
					// },
					// If expanded property is @reverse:
					Keyword::Reverse => {
						// If value is not a map, an invalid @reverse value error
						// has been detected and processing is aborted.
						if let JsonValue::Object(value) = value {
							let mut reverse_entries = Vec::with_capacity(value.len());
							for (reverse_prop, reverse_value) in value.iter() {
								reverse_entries.push(Entry(reverse_prop, reverse_value));
							}

							if ordered {
								reverse_entries.sort();
							}

							for Entry(reverse_prop, reverse_value) in reverse_entries {
								match expand_iri(active_context, reverse_prop, false, true) {
									Ok(Key::Keyword(_)) => {
										return Err(ExpansionError::InvalidReverseProperty)
									},
									Ok(Key::Prop(reverse_prop)) => {
										// Something's wrong here.
										let reverse_expanded_value = expand_element(active_context, None, reverse_value, base_url, loader, ordered, false).await?;
										result.insert_all_reverse(reverse_prop, reverse_expanded_value.into_iter())
									},
									_ => ()
								}
							}
						} else {
							return Err(ExpansionError::InvalidReverseValue)
						}
					},
					// If expanded property is @nest
					Keyword::Nest => {
						panic!("TODO nest")
					},
					// When the frameExpansion flag is set, if expanded property is any
					// other framing keyword (@default, @embed, @explicit,
					// @omitDefault, or @requireAll)
					// NOTE we don't handle frame expansion here.
					_ => ()
				}
			},

			Key::Prop(prop) => {
				let mut container_mapping = Mown::Owned(Container::new());

				let key_definition = active_context.get(key);
				let mut is_reverse_property = false;
				if let Some(key_definition) = key_definition {
					is_reverse_property = key_definition.reverse_property;

					// Initialize container mapping to key's container mapping in active context.
					container_mapping = Mown::Borrowed(&key_definition.container);

					// If key's term definition in `active_context` has a type mapping of `@json`,
					// set expanded value to a new map,
					// set the entry `@value` to `value`, and set the entry `@type` to `@json`.
					if key_definition.typ == Some(Key::Keyword(Keyword::JSON)) {
						panic!("TODO json")
					}
				}

				let mut expanded_value;

				// Otherwise, if container mapping includes @language and value is a map then
				// value is expanded from a language map as follows:
				if value.is_object() && container_mapping.contains(ContainerType::Language) {
					// Initialize expanded value to an empty array.
					expanded_value = Vec::new();

					// Initialize direction to the default base direction from active context.
					let mut direction = active_context.default_base_direction();

					// If key's term definition in active context has a
					// direction mapping, update direction with that value.
					if let Some(key_definition) = key_definition {
						if let Some(key_direction) = key_definition.direction {
							direction = key_direction
						}
					}

					// For each key-value pair language-language value in
					// value, ordered lexicographically by language if ordered is true:
					let mut language_entries = Vec::with_capacity(value.len());
					for (language, language_value) in value.entries() {
						language_entries.push(Entry(language, language_value));
					}

					if ordered {
						language_entries.sort();
					}

					for Entry(language, language_value) in language_entries {
						// If language value is not an array set language value to
						// an array containing only language value.
						let language_value = as_array(language_value);

						// For each item in language value:
						for item in language_value {
							match item {
								// If item is null, continue to the next entry in
								// language value.
								JsonValue::Null => (),
								JsonValue::Short(_) | JsonValue::String(_) => {
									let item = item.as_str().unwrap();

									// initialize a new map v consisting of two
									// key-value pairs: (@value-item) and
									// (@language-language).
									let v = Value::Literal(Literal::String {
										data: item.to_string(),
										language: Some(language.to_string()),
										// If direction is not null, add an entry for @direction to v with direction.
										direction: direction
									}, HashSet::new());

									// If item is neither @none nor well-formed
									// according to section 2.2.9 of [BCP47],
									//processors SHOULD issue a warning.
									// TODO warning

									// If language is @none, or expands to
									// @none, remove @language from v.
									// TODO FIXME ?

									// Append v to expanded value.
									expanded_value.push(v.into())
								},
								_ => {
									// item must be a string, otherwise an
									// invalid language map value error has
									// been detected and processing is aborted.
									return Err(ExpansionError::InvalidLanguageMapValue)
								}
							}
						}
					}
				} else if value.is_object() && container_mapping.contains(ContainerType::Index) || container_mapping.contains(ContainerType::Type) || container_mapping.contains(ContainerType::Id) {
					// Otherwise, if container mapping includes @index, @type, or @id and value
					// is a map then value is expanded from an map as follows:

					// Initialize expanded value to an empty array.
					expanded_value = Vec::new();

					// Initialize `index_key` to the key's index mapping in
					// `active_context`, or @index, if it does not exist.
					let index_key = if let Some(key_definition) = key_definition {
						if let Some(index) = &key_definition.index {
							index.as_str()
						} else {
							"@index"
						}
					} else {
						"@index"
					};

					// For each key-value pair index-index value in value,
					// ordered lexicographically by index if ordered is true:
					let mut entries = Vec::with_capacity(value.len());
					for (key, value) in value.entries() {
						entries.push(Entry(key, value))
					}

					if ordered {
						entries.sort();
					}

					for Entry(index, index_value) in &entries {
						// If container mapping includes @id or @type,
						// initialize `map_context` to the `previous_context`
						// from `active_context` if it exists, otherwise, set
						// `map_context` to `active_context`.
						let mut map_context = Mown::Borrowed(active_context);
						if container_mapping.contains(ContainerType::Type) || container_mapping.contains(ContainerType::Id) {
							if let Some(previous_context) = active_context.previous_context() {
								map_context = Mown::Borrowed(previous_context)
							}
						}

						// If container mapping includes @type and
						// index's term definition in map context has a
						// local context, update map context to the result of
						// the Context Processing algorithm, passing
						// map context as active context the value of the
						// index's local context as local context and base URL
						// from the term definition for index in map context.
						if container_mapping.contains(ContainerType::Type) {
							if let Some(index_definition) = map_context.get(index) {
								if let Some(local_context) = &index_definition.context {
									let base_url = index_definition.base_url.as_ref().map(|url| url.as_iri());
									map_context = Mown::Owned(local_context.process(map_context.as_ref(), loader, base_url, false, false, true).await?)
								}
							}
						}

						// Otherwise, set map context to active context.
						// TODO What?

						// Initialize `expanded_index` to the result of IRI
						// expanding index.
						let expanded_index = match expand_iri(active_context, index, false, true) {
							Ok(Key::Keyword(Keyword::None)) => None,
							Ok(Key::Keyword(kw)) => Some(kw.into_str().to_string()),
							Ok(Key::Prop(Property::Id(id))) => Some(id.iri().as_str().to_string()),
							Ok(Key::Prop(Property::Blank(_))) => return Err(ExpansionError::InvalidIndexValue),
							Err(index) => Some(index)
						};

						// If index value is not an array set index value to
						// an array containing only index value.
						// let index_value = as_array(index_value);

						// Initialize index value to the result of using this
						// algorithm recursively, passing map context as
						// active context, key as active property,
						// index value as element, base URL, and the
						// frameExpansion and ordered flags.
						let index_value = expand_element(map_context.as_ref(), Some(key), index_value, base_url, loader, ordered, false).await?;

						// For each item in index value:
						for mut item in index_value {
							// If container mapping includes @graph,
							// and item is not a graph object, set item to
							// a new map containing the key-value pair
							// @graph-item, ensuring that the value is
							// represented using an array.
							if container_mapping.contains(ContainerType::Graph) && !item.is_graph() {
								let mut graph = HashSet::new();
								graph.insert(item);

								let mut node = Node::new();
								node.graph = Some(graph);

								item = node.into();
							}

							if expanded_index.is_some() {
								// If `container_mapping` includes @index,
								// index key is not @index, and expanded index is
								// not @none:
								// TODO the @none part.
								if container_mapping.contains(ContainerType::Index) && index_key != "@index" {
									// Initialize re-expanded index to the result
									// of calling the Value Expansion algorithm,
									// passing the active context, index key as
									// active property, and index as value.
									let re_expanded_index = expand_literal(active_context, Some(index_key), &JsonValue::String(index.to_string()))?;
									let _re_expanded_index = if let Object::Value(Value::Literal(Literal::String { data, .. }, _), _) = re_expanded_index {
										data
									} else {
										return Err(ExpansionError::InvalidIndexValue)
									};

									// Initialize expanded index key to the result
									// of IRI expanding index key.
									let _expanded_index_key = expand_iri(active_context, index_key, false, true);

									// Initialize index property values to the
									// concatenation of re-expanded index with any
									// existing values of expanded index key in
									// item.
									// let index_property_values = ...;
									panic!("I don't know what to do here...")

									// Add the key-value pair (expanded index
									// key-index property values) to item.
									//

									// If item is a value object, it MUST NOT
									// contain any extra properties; an invalid
									// value object error has been detected and
									// processing is aborted.
								} else if container_mapping.contains(ContainerType::Index) && item.data().index.is_none() {
									// Otherwise, if container mapping includes
									// @index, item does not have an entry @index,
									// and expanded index is not @none, add the
									// key-value pair (@index-index) to item.
									item.data_mut().index = Some(index.to_string())
								} else if container_mapping.contains(ContainerType::Id) && item.id().is_none() {
									// Otherwise, if container mapping includes
									// @id item does not have the entry @id,
									// and expanded index is not @none, add the
									// key-value pair (@id-expanded index) to
									// item, where expanded index is set to the
									// result of IRI expanding index using true for
									// document relative and false for vocab.
									//
									panic!("TODO 2")
								} else if container_mapping.contains(ContainerType::Type) {
									// Otherwise, if container mapping includes
									// @type and expanded index is not @none,
									// initialize types to a new array consisting
									// of expanded index followed by any existing
									// values of @type in item. Add the key-value
									// pair (@type-types) to item.
									panic!("TODO 3")
								}
							}

							// Append item to expanded value.
							expanded_value.push(item)
						}
					}
				} else {
					// Otherwise, initialize expanded value to the result of using this
					// algorithm recursively, passing active context, key for active property,
					// value for element, base URL, and the frameExpansion and ordered flags.
					expanded_value = expand_element(active_context, Some(key), value, base_url, loader, ordered, false).await?;
				}

				// If container mapping includes @list and expanded value is
				// not already a list object, convert expanded value to a list
				// object by first setting it to an array containing only
				// expanded value if it is not already an array, and then by
				// setting it to a map containing the key-value pair
				// @list-expanded value.
				if container_mapping.contains(ContainerType::List) {
					if expanded_value.len() != 1 || !expanded_value[0].is_list() {
						expanded_value = vec![Value::List(expanded_value).into()];
					}
				}

				// If container mapping includes @graph, and includes neither
				// @id nor @index, convert expanded value into an array, if
				// necessary, then convert each value ev in expanded value
				// into a graph object:
				if container_mapping.contains(ContainerType::Graph) {
					panic!("TODO graph container")
				}

				// If the term definition associated to key indicates that it
				// is a reverse property:
				if is_reverse_property {
					// We must filter out anything that is not an object.
					for value in &expanded_value {
						match value {
							Object::Value(_, _) => return Err(ExpansionError::InvalidReversePropertyValue),
							_ => ()
						}
					}

					result.insert_all_reverse(prop, expanded_value.into_iter());
				} else {
					// Otherwise, key is not a reverse property use add value
					// to add expanded value to the expanded property entry in
					// result using true for as array.
					result.insert_all(prop, expanded_value.into_iter());
				}
			}
		}
	}

	// For each key nesting-key in nests, ordered lexicographically
	// if ordered is true:
	// FIXME TODO

	// If result contains the entry @value:
	// The result must not contain any entries other than @direction, @index,
	// @language, @type, and @value.

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
	if active_property == None || active_property == Some("@graph") {
		// If `result` is a map which is empty, or contains only the entries `@value`
		// or `@list`, set `result` to null.
		// => drop values

		// Otherwise, if result is a map whose only entry is @id, set result to null.
		if result.is_empty() {
			return Ok(None);
		}
	} else {
		if result.is_empty() && result.id.is_none() {
			return Ok(None);
		}
	}

	Ok(Some((result, result_data)))
}
