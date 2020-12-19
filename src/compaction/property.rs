use json::JsonValue;
use crate::{
	Id,
	ContextMut,
	Indexed,
	object,
	Object,
	Node,
	Lenient,
	Error,
	ErrorCode,
	context::{
		Loader,
		Inversible
	},
	syntax::{
		Keyword,
		Container,
		ContainerType,
		Term
	}
};
use super::{
	Compact,
	CompactIndexed,
	Options,
	compact_iri,
	compact_iri_with,
	compact_collection_with,
	add_value,
	value_value
};

async fn compact_property_list<T: Sync + Send + Id, C: ContextMut<T>, L: Loader>(list: &[Indexed<Object<T>>], expanded_index: Option<&str>, nest_result: &mut json::object::Object, container: Container, as_array: bool, item_active_property: &str, active_context: Inversible<T, &C>, loader: &mut L, options: Options) -> Result<(), Error> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
	// If expanded item is a list object:
	let mut compacted_item = compact_collection_with(list.iter(), active_context.clone(), active_context.clone(), Some(item_active_property), loader, options).await?;

	// If compacted item is not an array,
	// then set `compacted_item` to an array containing only `compacted_item`.
	if !compacted_item.is_array() {
		compacted_item = JsonValue::Array(vec![compacted_item])
	}

	// If container does not include @list:
	if !container.contains(ContainerType::List) {
		// Convert `compacted_item` to a list object by setting it to
		// a map containing an entry where the key is the result of
		// IRI compacting @list and the value is the original
		// compacted item.
		let key = compact_iri(active_context.clone(), Keyword::List, true, false, options)?;
		let mut list_object = json::object::Object::new();
		list_object.insert(key.as_str().unwrap(), compacted_item);
		compacted_item = JsonValue::Object(list_object);

		// If `expanded_item` contains the entry @index-value,
		// then add an entry to compacted item where the key is
		// the result of IRI compacting @index and value is value.
		if let Some(index) = expanded_index {
			let key = compact_iri(active_context.clone(), Keyword::Index, true, false, options)?;
			match compacted_item {
				JsonValue::Object(ref mut obj) => obj.insert(key.as_str().unwrap(), index.into()),
				_ => unreachable!()
			}
		}

		// Use add value to add `compacted_item` to
		// the `item_active_property` entry in `nest_result` using `as_array`.
		add_value(nest_result, item_active_property, compacted_item, as_array)
	} else {
		// Otherwise, set the value of the item active property entry in nest result to compacted item.
		nest_result.insert(item_active_property, compacted_item)
	}

	Ok(())
}

async fn compact_property_graph<T: Sync + Send + Id, C: ContextMut<T>, L: Loader>(node: &Node<T>, expanded_index: Option<&str>, nest_result: &mut json::object::Object, container: Container, as_array: bool, item_active_property: &str, active_context: Inversible<T, &C>, loader: &mut L, options: Options) -> Result<(), Error> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
	// If expanded item is a graph object
	let mut compacted_item = node.graph.as_ref().unwrap().compact_with(active_context.clone(), active_context.clone(), Some(item_active_property), loader, options).await?;

	// If `container` includes @graph and @id:
	if container.contains(ContainerType::Graph) && container.contains(ContainerType::Id) {
		// Initialize `map_object` to the value of `item_active_property`
		// in `nest_result`, initializing it to a new empty map,
		// if necessary.
		if !nest_result.get(item_active_property).is_some() {
			nest_result.insert(item_active_property, JsonValue::new_object())
		}

		let map_object = match nest_result.get_mut(item_active_property) {
			Some(JsonValue::Object(map)) => map,
			_ => unreachable!()
		};

		// Initialize `map_key` by IRI compacting the value of @id in
		// `expanded_item` or @none if no such value exists
		// with `vocab` set to false if there is an @id entry in
		// `expanded_item`.
		let (id_value, vocab): (Lenient<Term<T>>, bool) = match node.id() {
			Some(term) => (term.clone().cast(), false),
			None => (Lenient::Ok(Term::Keyword(Keyword::None)), true)
		};

		let map_key = compact_iri(active_context, &id_value, vocab, false, options)?;

		// Use `add_value` to add `compacted_item` to
		// the `map_key` entry in `map_object` using `as_array`.
		add_value(map_object, map_key.as_str().unwrap(), compacted_item, as_array)
	} else if container.contains(ContainerType::Graph) && container.contains(ContainerType::Index) && node.is_simple_graph() {
		// Initialize `map_object` to the value of `item_active_property`
		// in `nest_result`, initializing it to a new empty map,
		// if necessary.
		if !nest_result.get(item_active_property).is_some() {
			nest_result.insert(item_active_property, JsonValue::new_object())
		}

		let map_object = match nest_result.get_mut(item_active_property) {
			Some(JsonValue::Object(map)) => map,
			_ => unreachable!()
		};

		// Initialize `map_key` the value of @index in `expanded_item`
		// or @none, if no such value exists.
		let map_key = match expanded_index {
			Some(index) => index,
			None => "@none"
		};

		// Use `add_value` to add `compacted_item` to
		// the `map_key` entry in `map_object` using `as_array`.
		add_value(map_object, map_key, compacted_item, as_array)
	} else if container.contains(ContainerType::Graph) && node.is_simple_graph() {
		// Otherwise, if `container` includes @graph and
		// `expanded_item` is a simple graph object
		// the value cannot be represented as a map object.

		// If `compacted_item` is an array with more than one value,
		// it cannot be directly represented,
		// as multiple objects would be interpreted as different named graphs.
		// Set `compacted_item` to a new map,
		// containing the key from IRI compacting @included and
		// the original `compacted_item` as the value.
		compacted_item = match compacted_item {
			JsonValue::Array(items) if items.len() > 1 => {
				let key = compact_iri(active_context, Keyword::Included, true, false, options)?;
				let mut map = json::object::Object::new();
				map.insert(key.as_str().unwrap(), JsonValue::Array(items));
				JsonValue::Object(map)
			},
			item => item
		};

		// Use `add_value` to add `compacted_item` to the
		// `item_active_property` entry in `nest_result` using `as_array`.
		add_value(nest_result, item_active_property, compacted_item, as_array)
	} else {
		// Otherwise, `container` does not include @graph or
		// otherwise does not match one of the previous cases.

		// Set `compacted_item` to a new map containing the key from
		// IRI compacting @graph using the original `compacted_item` as a value.
		let key = compact_iri(active_context.clone(), Keyword::Graph, true, false, options)?;
		let mut map = json::object::Object::new();
		map.insert(key.as_str().unwrap(), compacted_item);

		// If `expanded_item` contains an @id entry,
		// add an entry in `compacted_item` using the key from
		// IRI compacting @id using the value of
		// IRI compacting the value of @id in `expanded_item` using
		// false for vocab.
		if let Some(id) = node.id() {
			let key = compact_iri(active_context.clone(), Keyword::Id, false, false, options)?;
			let value = compact_iri(active_context.clone(), id, false, false, options)?;
			map.insert(key.as_str().unwrap(), value);
		}

		// If `expanded_item` contains an @index entry,
		// add an entry in `compacted_item` using the key from
		// IRI compacting @index and the value of @index in `expanded_item`.
		if let Some(index) = expanded_index {
			let key = compact_iri(active_context.clone(), Keyword::Index, true, false, options)?;
			map.insert(key.as_str().unwrap(), index.into());
		}

		// Use `add_value` to add `compacted_item` to the
		// `item_active_property` entry in `nest_result` using `as_array`.
		let compacted_item = JsonValue::Object(map);
		add_value(nest_result, item_active_property, compacted_item, as_array)
	}

	Ok(())
}

fn select_nest_result<'a, T: Id, C: ContextMut<T>>(result: &'a mut json::object::Object, active_context: Inversible<T, &C>, item_active_property: &str, compact_arrays: bool) -> Result<(&'a mut json::object::Object, Container, bool), Error> {
	let (nest_result, container) = match active_context.get(item_active_property) {
		Some(term_definition) => {
			let nest_result = match &term_definition.nest {
				Some(nest_term) => {
					// If nest term is not @nest,
					// or a term in the active context that expands to @nest,
					// an invalid @nest value error has been detected,
					// and processing is aborted.
					if nest_term != "@nest" {
						match active_context.get(nest_term.as_ref()) {
							Some(term_def) if term_def.value == Some(Term::Keyword(Keyword::Nest)) => (),
							_ => return Err(ErrorCode::InvalidNestValue.into())
						}
					}

					// If result does not have a nest_term entry,
					// initialize it to an empty map.
					if result.get(nest_term).is_none() {
						result.insert(nest_term, JsonValue::new_object())
					}

					// Initialize `nest_result` to the value of `nest_term` in result.
					match result.get_mut(nest_term) {
						Some(JsonValue::Object(map)) => map,
						_ => unreachable!()
					}
				},
				None => {
					// Otherwise, initialize `nest_result` to result.
					result
				}
			};

			(nest_result, term_definition.container)
		},
		None => {
			(result, Container::None)
		}
	};

	// Initialize container to container mapping for item active property
	// in active context, or to a new empty array,
	// if there is no such container mapping.
	// DONE.

	// Initialize `as_array` to true if `container` includes @set,
	// or if `item_active_property` is @graph or @list,
	// otherwise the negation of `options.compact_arrays`.
	let as_array = if container.contains(ContainerType::Set) || item_active_property == "@graph" || item_active_property == "@list" {
		true
	} else {
		!compact_arrays
	};

	Ok((nest_result, container, as_array))
}

/// Compact the given property into the `result` compacted object.
pub async fn compact_property<'a, T: 'a + Sync + Send + Id, N: 'a + object::Any<T> + Sync + Send, O: IntoIterator<Item=&'a Indexed<N>>, C: ContextMut<T>, L: Loader>(result: &mut json::object::Object, expanded_property: Term<T>, expanded_value: O, active_context: Inversible<T, &C>, loader: &mut L, inside_reverse: bool, options: Options)
-> Result<(), Error> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
	let lenient_expanded_property: Lenient<Term<T>> = expanded_property.into();
	let mut is_empty = true;

	// For each item `expanded_item` in `expanded value`
	for expanded_item in expanded_value {
		is_empty = false;
		// Initialize `item_active_property` by IRI compacting `expanded_property`
		// using `expanded_item` for value and `inside_reverse` for `reverse`.
		let item_active_property = compact_iri_with(active_context.clone(), &lenient_expanded_property, expanded_item, true, inside_reverse, options)?;

		// If the term definition for `item_active_property` in the active context
		// has a nest value entry (nest term)
		if let Some(item_active_property) = item_active_property.as_str() {
			let (nest_result, container, as_array) = select_nest_result(result, active_context.clone(), item_active_property, options.compact_arrays)?;

			// Initialize `compacted_item` to the result of using this algorithm
			// recursively, passing `active_context`, `item_active_property` for
			// `active_property`, `expanded_item` for `element`, along with the
			// `compact_arrays` and `ordered_flags`.
			// If `expanded_item` is a list object or a graph object,
			// use the value of the @list or @graph entries, respectively,
			// for `element` instead of `expanded_item`.
			match expanded_item.inner().as_ref() {
				object::Ref::List(list) => {
					compact_property_list(list, expanded_item.index(), nest_result, container, as_array, item_active_property, active_context.clone(), loader, options).await?
				},
				object::Ref::Node(node) if node.is_graph() => {
					compact_property_graph(node, expanded_item.index(), nest_result, container, as_array, item_active_property, active_context.clone(), loader, options).await?
				},
				_ => {
					let mut compacted_item = expanded_item.compact_with(active_context.clone(), active_context.clone(), Some(item_active_property), loader, options).await?;

					// if container includes @language, @index, @id,
					// or @type and container does not include @graph:
					if !container.contains(ContainerType::Graph) && (container.contains(ContainerType::Language) || container.contains(ContainerType::Index) || container.contains(ContainerType::Id) || container.contains(ContainerType::Type)) {
						// Initialize `map_object` to the value of
						// `item_active_property` in `nest_result`,
						// initializing it to a new empty map, if necessary.
						if !nest_result.get(item_active_property).is_some() {
							nest_result.insert(item_active_property, JsonValue::new_object())
						}

						let map_object = match nest_result.get_mut(item_active_property) {
							Some(JsonValue::Object(map)) => map,
							_ => unreachable!()
						};

						// Initialize container key by IRI compacting either
						// @language, @index, @id, or @type based on the contents of container.
						let container_type = if container.contains(ContainerType::Language) {
							ContainerType::Language
						} else if container.contains(ContainerType::Index) {
							ContainerType::Index
						} else if container.contains(ContainerType::Id) {
							ContainerType::Id
						} else {
							ContainerType::Type
						};

						let mut container_key = compact_iri(active_context.clone(), &Lenient::Ok(Term::Keyword(container_type.into())), true, false, options)?;

						// Initialize `index_key` to the value of index mapping in
						// the term definition associated with `item_active_property`
						// in active context, or @index, if no such value exists.
						let index_key = match active_context.get(item_active_property) {
							Some(def) if def.index.is_some() => def.index.as_ref().unwrap(),
							_ => "@index"
						};

						// If `container` includes @language and `expanded_item`
						// contains a @value entry, then set `compacted_item` to
						// the value associated with its @value entry.
						// Set `map_key` to the value of @language in `expanded_item`,
						// if any.
						let map_key = if container_type == ContainerType::Language && expanded_item.is_value() {
							if let object::Ref::Value(value) = expanded_item.inner().as_ref() {
								compacted_item = value_value(value)
							}

							match expanded_item.language() {
								Some(lang) => Some(lang.to_string()),
								None => None
							}
						} else if container_type == ContainerType::Index {
							if index_key == "@index" {
								// Otherwise, if `container` includes @index and
								// `index_key` is @index, set `map_key` to the value of
								// @index in `expanded_item`, if any.
								match expanded_item.index() {
									Some(index) => Some(index.to_string()),
									None => None
								}
							} else {
								// Otherwise, if `container` includes @index and
								// `index_key` is not @index:

								// Reinitialize `container_key` by
								// IRI compacting `index_key`.
								let lenient_index : Lenient<Term<T>> = Lenient::Unknown(index_key.to_string());
								container_key = compact_iri(active_context.clone(), &lenient_index, true, false, options)?;

								// Set `map_key` to the first value of
								// `container_key` in `compacted_item`, if any.
								let (map_key, remaining_values) = match &mut compacted_item {
									JsonValue::Object(map) => match map.remove(container_key.as_str().unwrap()) {
										Some(value) => match value {
											JsonValue::Short(_) | JsonValue::String(_) => {
												(Some(value.as_str().unwrap().to_string()), Vec::new())
											},
											JsonValue::Array(mut values) => {
												values.reverse();
												match values.pop() {
													Some(first_value) => {
														values.reverse();
														(first_value.as_str().map(|v| v.to_string()), values)
													},
													None => {
														values.reverse();
														(None, values)
													}
												}
											},
											other_value => (None, vec![other_value])
										},
										None => (None, Vec::new())
									},
									_ => (None, Vec::new())
								};

								// If there are remaining values in `compacted_item`
								// for container key, use `add_value` to add
								// those remaining values to the `container_key`
								// in `compacted_item`.
								// Otherwise, remove that entry from compacted item.
								if !remaining_values.is_empty() {
									match &mut compacted_item {
										JsonValue::Object(map) => {
											for value in remaining_values {
												add_value(map, container_key.as_str().unwrap(), value, false)
											}
										},
										_ => ()
									}
								}

								map_key
							}
						} else if container_type == ContainerType::Id {
							// Otherwise, if `container` includes @id,
							// set `map_key` to the value of `container_key` in
							// `compacted_item` and remove `container_key` from
							// `compacted_item`.
							match &mut compacted_item {
								JsonValue::Object(map) => match map.remove(container_key.as_str().unwrap()) {
									Some(JsonValue::String(str)) => Some(str.to_string()),
									Some(JsonValue::Short(str)) => Some(str.to_string()),
									_ => None
								},
								_ => None
							}
						} else {
							// Otherwise, if container includes @type:

							// Set `map_key` to the first value of `container_key` in
							// `compacted_item`, if any.
							let (map_key, remaining_values) = match &mut compacted_item {
								JsonValue::Object(map) => match map.remove(container_key.as_str().unwrap()) {
									Some(value) => match value {
										JsonValue::Short(_) | JsonValue::String(_) => {
											(Some(value.as_str().unwrap().to_string()), Vec::new())
										},
										JsonValue::Array(mut values) => {
											values.reverse();
											match values.pop() {
												Some(first_value) => {
													values.reverse();
													(first_value.as_str().map(|v| v.to_string()), values)
												},
												None => {
													values.reverse();
													(None, values)
												}
											}
										},
										other_value => (None, vec![other_value])
									},
									None => (None, Vec::new())
								},
								_ => (None, Vec::new())
							};

							// If there are remaining values in `compacted_item` for
							// `container_key`, use `add_value` to add those
							// remaining values to the `container_key` in
							// `compacted_item`.
							// Otherwise, remove that entry from compacted item.
							if !remaining_values.is_empty() {
								match &mut compacted_item {
									JsonValue::Object(map) => {
										for value in remaining_values {
											add_value(map, container_key.as_str().unwrap(), value, false)
										}
									},
									_ => ()
								}
							}

							// If `compacted_item` contains a single entry with a key
							// expanding to @id, set `compacted_item` to the result of
							// using this algorithm recursively,
							// passing `active_context`, `item_active_property` for
							// `active_property`, and a map composed of the single
							// entry for @id from `expanded_item` for `element`.
							if let JsonValue::Object(map) = &compacted_item {
								if map.len() == 1 {
									if let Some(_) = map.get("@id") {
										let obj = Object::Node(Node::with_id(expanded_item.id().unwrap().clone()));
										compacted_item = obj.compact_indexed_with(None, active_context.clone(), active_context.clone(), Some(item_active_property), loader, options).await?
									}
								}
							}

							map_key
						};

						// If `map_key` is null, set it to the result of
						// IRI compacting @none.
						let map_key = match map_key {
							Some(key) => key,
							None => {
								let key = compact_iri(active_context.clone(), Keyword::None, true, false, options)?;
								key.as_str().unwrap().to_string()
							}
						};

						// Use `add_value` to add `compacted_item` to
						// the `map_key` entry in `map_object` using `as_array`.
						add_value(map_object, &map_key, compacted_item, as_array)
					} else {
						// Otherwise, use `add_value` to add `compacted_item` to the
						// `item_active_property` entry in `nest_result` using `as_array`.
						add_value(nest_result, item_active_property, compacted_item, as_array)
					}
				}
			};
		}
	}

	// If expanded value is an empty array:
	if is_empty {
		// Initialize `item_active_property` by IRI compacting
		// `expanded_property` using `expanded_value` for `value` and
		// `inside_reverse` for `reverse`.
		let item_active_property = compact_iri_with(active_context.clone(), &lenient_expanded_property, &Indexed::new(Object::Node(Node::new()), None), true, inside_reverse, options)?;

		// If the term definition for `item_active_property` in the active context
		// has a nest value entry (nest term):
		if let Some(item_active_property) = item_active_property.as_str() {
			let (nest_result, _, _) = select_nest_result(result, active_context.clone(), item_active_property, options.compact_arrays)?;

			// Use `add_value` to add an empty array to the `item_active_property` entry in
			// `nest_result` using true for `as_array`.
			add_value(nest_result, item_active_property, JsonValue::Array(Vec::new()), true)
		}
	}

	Ok(())
}