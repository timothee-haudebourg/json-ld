mod iri;

pub use iri::*;

// type Any; // the toplevel class
//
// pub fn expand_value(active_context: &Context, value: &JsonValue, document_relative: bool, vocab: bool) {
// 	// ...
// }
//
// pub fn expand(mut active_context: &ActiveContext, active_property: Option<Property>, element: &JsonValue, mut frame_expansion: bool, ordered: bool, from_map: bool) -> Result<Vec<Ref<Any>>> {
// 	// If element is null, return null.
// 	if element.is_null() {
// 		return Ok(vec![]);
// 	}
//
// 	// If active property is @default, initialize the frameExpansion flag to false.
// 	if active_property == Some(Property::Default) {
// 		frame_expansion = false;
// 	}
//
// 	// If active_property has a term definition in active context with a local context, initialize
// 	// property-scoped context to that local context.
// 	let property_scoped_context = None;
// 	if let Some(active_property) = active_property {
// 		if let Some(def) = active_context.get(active_property) {
// 			if let Some(local_context) = def.local_context {
// 				property_scoped_context = Some(local_context);
// 			}
// 		}
// 	}
//
// 	match element {
// 		JsonValue::Null => unreachable!(),
//
// 		// If element is a scalar,
// 		JsonValue::Boolean(_) | JsonValue::Number(_) | JsonValue::Short(_) | JsonValue::String(_) => {
// 			// If active property is null or @graph, drop the free-floating scalar by returning null.
// 			match active_property {
// 				None | Some(Property::Graph) => {
// 					return Ok(vec![])
// 				},
// 				_ => ()
// 			}
//
// 			// If property-scoped context is defined, set active context to the result of the Context
// 			// Processing algorithm, passing active context and property-scoped context as local context.
// 			if let Some(property_scoped_context) = property_scoped_context {
// 				active_context = property_scoped_context;
// 			}
//
// 			// Return the result of the Value Expansion algorithm, passing the active context,
// 			// active property, and element as value.
// 			return expand_value(active_context, element, false, false);
// 		},
//
// 		// If element is an array,
// 		JsonValue::Array(items) => {
// 			// Initialize an empty array, result.
// 			let result = Vec::new();
//
// 			// For each item in element:
// 			for item in items {
// 				// Initialize expanded item to the result of using this algorithm recursively,
// 				// passing active context, active property, item as element, the frameExpansion
// 				// ordered, and from map flags.
// 				let mut expanded_item = expand(active_context, active_property, item, frame_expansion, ordered, from_map)?;
//
// 				// If the container mapping of active_property includes @list, and expanded item is
// 				// an array, set expanded item to a new map containing the entry @list where the
// 				// value is the original expanded item.
// 				if expanded_item.len() > 1 {
// 					if has_list_container(active_property) {
// 						panic!("TODO")
// 					} else {
// 						// If expanded item is an array, append each of its items to result.
// 						// Otherwise, if expanded item is not null, append it to result.
// 						result.append(&mut expanded_item);
// 					}
// 				}
// 			}
//
// 			return Ok(result)
// 		},
//
// 		// Otherwise element is a map.
// 		JsonValue::Object(element) => {
// 			// If active_context has a previous context, the active context is not propagated.
// 			// If from_map is undefined or false, and element does not contain an entry expanding
// 			// to @value, and element does not consist of a single entry expanding to @id, set
// 			// active_context to previous context from active_context, as the scope of a
// 			// term-scoped context does not apply when processing new node objects.
// 			if !from_map && !id_only_entry(element) && !has_value_entry(element) {
// 				active_context = active_context.previous_context;
// 			}
//
// 			// If property-scoped context is defined, set active context to the result of the
// 			// Context Processing algorithm, passing active context and property-scoped context as
// 			// local context.
// 			if let Some(property_scoped_context) = property_scoped_context {
// 				active_context = property_scoped_context;
// 			}
//
// 			// If element contains the entry @context, set active context to the result of the
// 			// Context Processing algorithm, passing active context and the value of the @context
// 			// entry as local context.
// 			if let Some(context_entry) = map.get("@context") {
// 				active_context = process_context(active_context, context_entry)?;
// 			}
//
// 			// Initialize type-scoped context to active context. This is used for expanding values
// 			// that may be relevant to any previous type-scoped context.
// 			let type_scoped_context = active_context;
//
// 			// For each key and value in element ordered lexicographically by key where key expands
// 			// to @type using the IRI Expansion algorithm, passing active context, key for value,
// 			// and true for vocab:
// 			for (key, value) in element {
// 				let expanded_key = expand_iri(active_context, key, false, true)?;
// 				if expanded_key == "@type" {
// 					// Convert value into an array, if necessary.
//
// 					// For each term which is a value of value ordered lexicographically, if term
// 					// is a string, and term's term definition in active context has a local
// 					// context, set active context to the result to the result of the Context
// 					// Processing algorithm, passing active context, and the value of the term's
// 					// local context as local context.
// 					for term in as_array(value) {
// 						if let Some(term) = term.as_string() {
// 							if let Some(def) = active_context.get(term) {
// 								if let Some(local_context) = def.local_context {
// 									active_context = process_context(active_context, local_context)?;
// 								}
// 							}
// 						}
// 					}
// 				}
// 			}
//
// 			// Initialize two empty maps, result and nests. Initialize input_type to the last value
// 			// of the first entry in element expanding to @type (if any), ordering entries
// 			// lexicographically by key.
// 			let mut result = json::object::Object::new();
// 			let mut nests = json::object::Object::new();
// 			let input_type = find_input_type(element)?;
//
// 			// For each key and value in element, ordered lexicographically by key if ordered is true:
// 			for (key, value) in element {
// 				// If key is @context, continue to the next key.
// 				if key == "@context" {
// 					continue
// 				}
//
// 				// Initialize expanded_property to the result of using the IRI Expansion algorithm,
// 				// passing active context, key for value, and true for vocab.
// 				let expanded_property = expand_iri(active_context, key, false, true)?;
//
// 				// If expanded_property is null or it neither contains a colon (:) nor it is a
// 				// keyword, drop key by continuing to the next key.
// 				use PropertyOrKeyword::*;
// 				match as_property_or_keyword(expanded_property) {
// 					// If expanded property is a keyword:
// 					Some(Keyword(kw)) => {
// 						// If result already has an expanded_property entry, other than @included
// 						// or @type (unless processing mode is json-ld-1.0), a colliding keywords
// 						// error has been detected and processing is aborted.
// 						// TODO
//
// 						match kw {
// 							// If active property equals @reverse, an invalid reverse property map
// 							// error has been detected and processing is aborted.
// 							Keyword::Reverse => {
// 								return Err(Error::InvalidReverseProperty.into());
// 							},
//
// 							// If expanded property is @id:
// 							Keyword::Id => {
// 								// If value is not a string, an invalid @id value error has been
// 								// detected and processing is aborted. When the frameExpansion flag
// 								// is set, value MAY be an empty map, or an array of one or more
// 								// strings.
// 								if let Some(id) = value.as_string() {
// 									// Otherwise, set expanded_value to the result of using the
// 									// IRI Expansion algorithm, passing active context, value, and
// 									// true for document_relative. When the frameExpansion flag is
// 									// set, expanded value will be an array of one or more of the
// 									// values, with string values expanded using the IRI Expansion
// 									// Algorithm as above.
// 									let expanded_value = expand_iri(active_context, value, true, false)?;
// 									// TODO what to do with expanded_value?
// 								} else {
// 									// TODO frameExpansion
// 									return Err(Error::InvalidIdValue.into());
// 								}
// 							},
//
// 							// If expanded property is @type:
// 							Keyword::Type => {
// 								// If value is neither a string nor an array of strings, an invalid
// 								// type value error has been detected and processing is aborted.
// 								// When the frameExpansion flag is set, value MAY be an empty map,
// 								// or a default object where the value of @default is restricted to
// 								// be an IRI. All other values mean that invalid type value error
// 								// has been detected and processing is aborted.
// 								// NOTE: checked later on.
//
// 								if frame_expansion && is_empty_map(value) {
// 									// If value is an empty map, set expanded value to value.
// 									// TODO
// 									panic!("Frame Expansion is not handled yet.")
// 								} else if frame_expansion && is_default_object(value) {
// 									// Otherwise, if value is a default object, set expanded value to a
// 									// new default object with the value of @default set to the result
// 									// of using the IRI Expansion algorithm, passing type-scoped
// 									// context for active context, true for vocab, and true for
// 									// document relative to expand that value.
// 									// TODO
// 									panic!("Frame Expansion is not handled yet.")
// 								} else {
// 									// Otherwise, set expanded_value to the result of using the IRI
// 									// Expansion algorithm, passing type-scoped context for active
// 									// context, true for vocab, and true for document relative to
// 									// expand the value or each of its items.
// 									let mut expanded_value = Vec::new();
// 									for ty in as_array(value) {
// 										if let Some(ty) = ty.as_str() {
// 											let expanded_type = expand_iri(type_scoped_context, ty, true, true)?;
// 											expanded_value.push(expanded_type);
// 										} else {
// 											return Err(Error::InvalidTypeValue.into())
// 										}
// 									}
//
// 									// If result already has an entry for @type, prepend the value
// 									// of @type in result to expanded_value, transforming it into
// 									// an array, if necessary.
// 									if let Some(types) = result.get_mut(Keyword(Keyword::Type)) {
// 										panic!("TODO: types already defined");
// 									}
// 									// TODO what to do with expanded_value?
// 								}
// 							},
//
// 							// If expanded property is @graph,
// 							Keyword::Graph => {
// 								// Set expanded_value to the result of using this algorithm
// 								// recursively passing active_context, @graph for active property,
// 								// value for element, and the frameExpansion and ordered flags,
// 								// ensuring that expanded_value is an array of one or more maps.
// 								let expanded_value = expand(active_context, Some(PropertyOrKeyword::Keyword(Keyword::Graph)), value, frame_expansion, ordered, false)?;
// 								// TODO what to do with expanded_value?
// 							},
//
// 							// If expanded property is @included:
// 							Keyword::Included => {
// 								// If processing mode is json-ld-1.0, continue with the next key
// 								// from element.
// 								// TODO
//
// 								// Set expanded value to the result of using this algorithm
// 								// recursively passing active context, active property, value for
// 								// element, and the frameExpansion and ordered flags, ensuring that
// 								// the result is an array.
// 								let expanded_value = expand(active_context, active_property, value, frame_expansion, ordered, false)?;
// 								// TODO what to do with expanded_value?
//
// 								// If any element of expanded value is not a node object, an
// 								// invalid @included value error has been detected and processing
// 								// is aborted.
// 								// TODO: I stopped HERE
// 							},
//
// 							// TODO
// 						}
// 					},
//
// 					Some(Property(expanded_property)) => {
// 						// TODO
// 					},
// 					None => () // skip.
// 				}
// 			}
// 		}
// 	}
// }
//
// /// Default value for the flags are false.
// fn expand_iri(active_context: &Context, value: JsonValue, document_relative: bool, vocab: bool) {
// 	// ...
// }
