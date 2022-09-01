use std::hash::Hash;
use json_ld_core::{NamespaceMut, Reference, Loader, ContextLoader, Context, Indexed, Node, Value, object, Type, Container, ContainerKind, Term, IndexedObject, context::Nest};
use json_ld_syntax::Keyword;
use json_ld_context_processing::Process;
use locspan::Meta;
use crate::{Options, MetaError, Error, compact_iri, compact_key, compact_iri_with, compact_collection_with, add_value};

async fn compact_property_list<I, B, M, C, N, L>(
	vocabulary: &mut N,
	Meta(list, meta): Meta<&[IndexedObject<I, B, M>], &M>,
	expanded_index: Option<&json_ld_syntax::Entry<String, M>>,
	nest_result: &mut json_syntax::Object<M>,
	container: Container,
	as_array: bool,
	item_active_property: Meta<&str, &M>,
	active_context: &Context<I, B, C>,
	loader: &mut L,
	options: Options
) -> Result<(), MetaError<M, L::ContextError>>
where
	N: Send + Sync + NamespaceMut<I, B>,
	I: Clone + Hash + Eq + Send + Sync,
	B: Clone + Hash + Eq + Send + Sync,
	M: Clone + Send + Sync,
	C: Process<I, B, M>,
	L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
	L::Output: Into<Value<M, C>>,
	L::Context: Into<C>
{
	// If expanded item is a list object:
	let mut compacted_item = compact_collection_with(
		vocabulary,
		Meta(list.iter(), meta),
		active_context,
		active_context,
		Some(item_active_property),
		loader,
		options
	)
	.await?;

	// If compacted item is not an array,
	// then set `compacted_item` to an array containing only `compacted_item`.
	if !compacted_item.is_array() {
		let mut array = json_syntax::Array::default();
		array.push(compacted_item);
		compacted_item = Meta(json_syntax::Value::Array(array), meta.clone())
	}

	// If container does not include @list:
	if !container.contains(ContainerKind::List) {
		// Convert `compacted_item` to a list object by setting it to
		// a map containing an entry where the key is the result of
		// IRI compacting @list and the value is the original
		// compacted item.
		let key = compact_key(
			vocabulary,
			active_context,
			Meta(&Term::Keyword(Keyword::List), meta),
			true,
			false,
			options,
		).map_err(Meta::cast)?;
		let mut compacted_item_list_object = json_syntax::Object::default();
		compacted_item_list_object.insert(
			key.unwrap(),
			compacted_item,
		);

		// If `expanded_item` contains the entry @index-value,
		// then add an entry to compacted item where the key is
		// the result of IRI compacting @index and value is value.
		if let Some(index) = expanded_index {
			let key = compact_key(
				vocabulary,
				active_context,
				Meta(&Term::Keyword(Keyword::Index), &index.key_metadata),
				true,
				false,
				options,
			).map_err(Meta::cast)?;
			
			let Meta(index_value, meta) = &index.value;

			compacted_item_list_object.insert(
				key.unwrap(),
				Meta(json_syntax::Value::String(index_value.as_str().into()), meta.clone())
			);
		}

		compacted_item = Meta(json_syntax::Value::Object(compacted_item_list_object), meta.clone());

		// Use add value to add `compacted_item` to
		// the `item_active_property` entry in `nest_result` using `as_array`.
		add_value(
			nest_result,
			item_active_property,
			compacted_item,
			as_array
		)
	} else {
		// Otherwise, set the value of the item active property entry in nest result to compacted item.
		nest_result.insert(
			Meta(item_active_property.0.into(), item_active_property.1.clone()),
			compacted_item
		);
	}

	Ok(())
}

async fn compact_property_graph<I, B, M, C, N, L>(
	vocabulary: &mut N,
	Meta(node, meta): Meta<&Node<I, B, M>, &M>,
	expanded_index: Option<&str>,
	nest_result: &mut json_syntax::Object<M>,
	container: Container,
	as_array: bool,
	item_active_property: &str,
	active_context: &Context<I, B, C>,
	loader: &mut L,
	options: Options
) -> Result<(), Error<L::ContextError>>
where
	N: Send + Sync + NamespaceMut<I, B>,
	I: Clone + Hash + Eq + Send + Sync,
	B: Clone + Hash + Eq + Send + Sync,
	M: Clone + Send + Sync,
	C: Process<I, B, M>,
	L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
	L::Output: Into<Value<M, C>>,
	L::Context: Into<C>
{
	// If expanded item is a graph object
	let mut compacted_item = node
		.graph
		.as_ref()
		.unwrap()
		.compact_full(
			active_context,
			active_context,
			Some(item_active_property),
			loader,
			options,
			meta.clone(),
		)
		.await?;

	// If `container` includes @graph and @id:
	if container.contains(ContainerKind::Graph) && container.contains(ContainerKind::Id) {
		// Initialize `map_object` to the value of `item_active_property`
		// in `nest_result`, initializing it to a new empty map,
		// if necessary.
		if nest_result.get(item_active_property).is_none() {
			nest_result.insert(
				K::new_key(item_active_property, meta(None)),
				K::object(K::Object::default(), meta(None)),
			);
		}

		let mut map_object = nest_result.get_mut(item_active_property).unwrap();
		let map_object = map_object.as_object_mut().unwrap();

		// Initialize `map_key` by IRI compacting the value of @id in
		// `expanded_item` or @none if no such value exists
		// with `vocab` set to false if there is an @id entry in
		// `expanded_item`.
		let (id_value, vocab): (Term<I, B>, bool) = match node.id() {
			Some(term) => (term.clone().into_term(), false),
			None => (Term::Keyword(Keyword::None), true),
		};

		let map_key =
			compact_iri::<J, _, _>(active_context, &id_value, vocab, false, options)?.unwrap();

		// Use `add_value` to add `compacted_item` to
		// the `map_key` entry in `map_object` using `as_array`.
		add_value(
			map_object,
			map_key.as_str(),
			compacted_item,
			as_array,
			|| meta(None),
		)
	} else if container.contains(ContainerKind::Graph)
		&& container.contains(ContainerKind::Index)
		&& node.is_simple_graph()
	{
		// Initialize `map_object` to the value of `item_active_property`
		// in `nest_result`, initializing it to a new empty map,
		// if necessary.
		if nest_result.get(item_active_property).is_none() {
			nest_result.insert(
				K::new_key(item_active_property, meta(None)),
				K::object(K::Object::default(), meta(None)),
			);
		}

		let mut map_object = nest_result.get_mut(item_active_property).unwrap();
		let map_object = map_object.as_object_mut().unwrap();

		// Initialize `map_key` the value of @index in `expanded_item`
		// or @none, if no such value exists.
		let map_key = expanded_index.unwrap_or("@none");

		// Use `add_value` to add `compacted_item` to
		// the `map_key` entry in `map_object` using `as_array`.
		add_value(map_object, map_key, compacted_item, as_array, || meta(None))
	} else if container.contains(ContainerKind::Graph) && node.is_simple_graph() {
		// Otherwise, if `container` includes @graph and
		// `expanded_item` is a simple graph object
		// the value cannot be represented as a map object.

		// If `compacted_item` is an array with more than one value,
		// it cannot be directly represented,
		// as multiple objects would be interpreted as different named graphs.
		// Set `compacted_item` to a new map,
		// containing the key from IRI compacting @included and
		// the original `compacted_item` as the value.
		compacted_item = match compacted_item.into_parts() {
			(generic_json::Value::Array(items), items_meta) if items.len() > 1 => {
				let key = compact_iri::<J, _, _>(
					active_context,
					&Term::Keyword(Keyword::Included),
					true,
					false,
					options,
				)?
				.unwrap();
				let mut map = K::Object::default();
				map.insert(
					K::new_key(key.as_str(), meta(None)),
					K::array(items, items_meta),
				);
				K::object(map, meta(None))
			}
			(item, item_meta) => K::new(item, item_meta),
		};

		// Use `add_value` to add `compacted_item` to the
		// `item_active_property` entry in `nest_result` using `as_array`.
		add_value(
			nest_result,
			item_active_property,
			compacted_item,
			as_array,
			|| meta(None),
		)
	} else {
		// Otherwise, `container` does not include @graph or
		// otherwise does not match one of the previous cases.

		// Set `compacted_item` to a new map containing the key from
		// IRI compacting @graph using the original `compacted_item` as a value.
		let key = compact_iri::<J, _, _>(
			active_context,
			&Term::Keyword(Keyword::Graph),
			true,
			false,
			options,
		)?
		.unwrap();
		let mut map = K::Object::default();
		map.insert(K::new_key(key.as_str(), meta(None)), compacted_item);

		// If `expanded_item` contains an @id entry,
		// add an entry in `compacted_item` using the key from
		// IRI compacting @id using the value of
		// IRI compacting the value of @id in `expanded_item` using
		// false for vocab.
		if let Some(id) = node.id() {
			let key = compact_iri::<J, _, _>(
				active_context,
				&Term::Keyword(Keyword::Id),
				false,
				false,
				options,
			)?
			.unwrap();
			let value = compact_iri::<J, _, _>(
				active_context,
				&id.clone().into_term(),
				false,
				false,
				options,
			)?;
			map.insert(
				K::new_key(key.as_str(), meta(None)),
				match value {
					Some(s) => K::string(s.as_str().into(), meta(None)),
					None => K::null(meta(None)),
				},
			);
		}

		// If `expanded_item` contains an @index entry,
		// add an entry in `compacted_item` using the key from
		// IRI compacting @index and the value of @index in `expanded_item`.
		if let Some(index) = expanded_index {
			let key = compact_iri::<J, _, _>(
				active_context,
				&Term::Keyword(Keyword::Index),
				true,
				false,
				options,
			)?
			.unwrap();
			map.insert(
				K::new_key(key.as_str(), meta(None)),
				K::string(index.into(), meta(None)),
			);
		}

		// Use `add_value` to add `compacted_item` to the
		// `item_active_property` entry in `nest_result` using `as_array`.
		let compacted_item = K::object(map, meta(None));
		add_value(
			nest_result,
			item_active_property,
			compacted_item,
			as_array,
			|| meta(None),
		)
	}

	Ok(())
}

fn select_nest_result<'a, I, B, M, C, N, E>(
	vocabulary: &N,
	result: &'a mut json_syntax::Object<M>,
	active_context: &Context<I, B, C>,
	item_active_property: &str,
	compact_arrays: bool
) -> Result<(&'a mut json_syntax::Object<M>, Container, bool), MetaError<M, E>>
where
	N: Send + Sync + NamespaceMut<I, B>,
	I: Clone + Hash + Eq + Send + Sync,
	B: Clone + Hash + Eq + Send + Sync,
	M: Clone + Send + Sync
{
	let (nest_result, container) = match active_context.get(item_active_property) {
		Some(term_definition) => {
			let nest_result = match &term_definition.nest {
				Some(nest_term) => {
					// If nest term is not @nest,
					// or a term in the active context that expands to @nest,
					// an invalid @nest value error has been detected,
					// and processing is aborted.
					if *nest_term != Nest::Nest {
						match active_context.get(nest_term.as_ref()) {
							Some(term_def)
								if term_def.value == Some(Term::Keyword(Keyword::Nest)) => {}
							_ => return Err(Error::InvalidNestValue),
						}
					}

					// If result does not have a nest_term entry,
					// initialize it to an empty map.
					if result.get(nest_term).is_none() {
						result.insert(
							K::new_key(nest_term.as_str(), meta()),
							K::object(K::Object::default(), meta()),
						);
					}

					// Initialize `nest_result` to the value of `nest_term` in result.
					let value: ValueMut<'a, K> = result.get_mut(nest_term).unwrap().into();
					let sub_object: &'a mut K::Object = value.into_object_mut().unwrap();
					sub_object
					// SubObject::Sub(result.get_mut(nest_term).unwrap().as_object_mut().unwrap())
				}
				None => {
					// Otherwise, initialize `nest_result` to result.
					result
				}
			};

			(nest_result, term_definition.container)
		}
		None => (result, Container::None),
	};

	// Initialize container to container mapping for item active property
	// in active context, or to a new empty array,
	// if there is no such container mapping.
	// DONE.

	// Initialize `as_array` to true if `container` includes @set,
	// or if `item_active_property` is @graph or @list,
	// otherwise the negation of `options.compact_arrays`.
	let as_array = if container.contains(ContainerKind::Set)
		|| item_active_property == "@graph"
		|| item_active_property == "@list"
	{
		true
	} else {
		!compact_arrays
	};

	Ok((nest_result, container, as_array))
}

/// Compact the given property into the `result` compacted object.
pub async fn compact_property<'a, T, O, I, B, M, C, N, L>(
	vocabulary: &mut N,
	result: &mut json_syntax::Object<M>,
	expanded_property: Meta<Term<I, B>, M>,
	expanded_value: O,
	active_context: &Context<I, B, C>,
	loader: &mut L,
	inside_reverse: bool,
	options: Options
) -> Result<(), MetaError<M, L::ContextError>>
where
	T: 'a + object::Any<I, B, M> + Sync + Send,
	O: IntoIterator<Item = &'a Meta<Indexed<T, M>, M>>,
	N: Send + Sync + NamespaceMut<I, B>,
	I: Clone + Hash + Eq + Send + Sync,
	B: Clone + Hash + Eq + Send + Sync,
	M: 'a + Clone + Send + Sync,
	C: Process<I, B, M>,
	L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
	L::Output: Into<Value<M, C>>,
	L::Context: Into<C>
{
	let mut is_empty = true;

	// For each item `expanded_item` in `expanded value`
	for expanded_item in expanded_value {
		is_empty = false;
		// Initialize `item_active_property` by IRI compacting `expanded_property`
		// using `expanded_item` for value and `inside_reverse` for `reverse`.
		let item_active_property = compact_iri_with(
			vocabulary,
			active_context,
			Meta(&expanded_property.0, &expanded_property.1),
			expanded_item,
			true,
			inside_reverse,
			options,
		).map_err(Meta::cast)?;

		// If the term definition for `item_active_property` in the active context
		// has a nest value entry (nest term)
		if let Some(item_active_property) = item_active_property {
			let (nest_result, container, as_array) =
				select_nest_result(
					vocabulary,
					result,
					active_context,
					item_active_property.as_str(),
					options.compact_arrays
				)?;

			// Initialize `compacted_item` to the result of using this algorithm
			// recursively, passing `active_context`, `item_active_property` for
			// `active_property`, `expanded_item` for `element`, along with the
			// `compact_arrays` and `ordered_flags`.
			// If `expanded_item` is a list object or a graph object,
			// use the value of the @list or @graph entries, respectively,
			// for `element` instead of `expanded_item`.
			match expanded_item.inner().as_ref() {
				object::Ref::List(list) => {
					compact_property_list::<J, K, _, _, _, _>(
						list,
						expanded_item.index(),
						nest_result,
						container,
						as_array,
						item_active_property.as_str(),
						active_context,
						loader,
						options,
						meta.clone(),
					)
					.await?
				}
				object::Ref::Node(node) if node.is_graph() => {
					compact_property_graph::<J, K, _, _, _, _>(
						node,
						expanded_item.index(),
						nest_result,
						container,
						as_array,
						item_active_property.as_str(),
						active_context,
						loader,
						options,
						meta.clone(),
					)
					.await?
				}
				_ => {
					let mut compacted_item: K = expanded_item
						.compact_full(
							active_context,
							active_context,
							Some(item_active_property.as_str()),
							loader,
							options,
							meta.clone(),
						)
						.await?;

					// if container includes @language, @index, @id,
					// or @type and container does not include @graph:
					if !container.contains(ContainerKind::Graph)
						&& (container.contains(ContainerKind::Language)
							|| container.contains(ContainerKind::Index)
							|| container.contains(ContainerKind::Id)
							|| container.contains(ContainerKind::Type))
					{
						// Initialize `map_object` to the value of
						// `item_active_property` in `nest_result`,
						// initializing it to a new empty map, if necessary.
						if nest_result.get(item_active_property.as_str()).is_none() {
							nest_result.insert(
								K::new_key(item_active_property.as_str(), meta(None)),
								K::empty_object(meta(None)),
							);
						}

						let mut map_object =
							nest_result.get_mut(item_active_property.as_str()).unwrap();
						let map_object = map_object.as_object_mut().unwrap();

						// Initialize container key by IRI compacting either
						// @language, @index, @id, or @type based on the contents of container.
						let container_type = if container.contains(ContainerKind::Language) {
							ContainerKind::Language
						} else if container.contains(ContainerKind::Index) {
							ContainerKind::Index
						} else if container.contains(ContainerKind::Id) {
							ContainerKind::Id
						} else {
							ContainerKind::Type
						};

						let mut container_key = compact_iri::<J, _, _>(
							active_context,
							&Term::Keyword(container_type.into()),
							true,
							false,
							options,
						)?;

						// Initialize `index_key` to the value of index mapping in
						// the term definition associated with `item_active_property`
						// in active context, or @index, if no such value exists.
						let index_key = match active_context.get(item_active_property.as_str()) {
							Some(def) if def.index.is_some() => def.index.as_ref().unwrap(),
							_ => "@index",
						};

						// If `container` includes @language and `expanded_item`
						// contains a @value entry, then set `compacted_item` to
						// the value associated with its @value entry.
						// Set `map_key` to the value of @language in `expanded_item`,
						// if any.
						let map_key = if container_type == ContainerKind::Language
							&& expanded_item.is_value()
						{
							if let object::Ref::Value(value) = expanded_item.inner().as_ref() {
								compacted_item = value_value(value, meta.clone())
							}

							expanded_item.language().map(|lang| lang.to_string())
						} else if container_type == ContainerKind::Index {
							if index_key == "@index" {
								// Otherwise, if `container` includes @index and
								// `index_key` is @index, set `map_key` to the value of
								// @index in `expanded_item`, if any.
								expanded_item.index().map(|index| index.to_string())
							} else {
								// Otherwise, if `container` includes @index and
								// `index_key` is not @index:

								// Reinitialize `container_key` by
								// IRI compacting `index_key`.
								container_key = compact_iri::<J, _, _>(
									active_context,
									&Term::Ref(Reference::Invalid(index_key.to_string())),
									true,
									false,
									options,
								)?;

								// Set `map_key` to the first value of
								// `container_key` in `compacted_item`, if any.
								let (map_key, remaining_values) = match compacted_item
									.as_value_mut()
								{
									generic_json::ValueMut::Object(map) => {
										match map.remove(container_key.as_ref().unwrap().as_str()) {
											Some(value) => match value.into_parts() {
												(generic_json::Value::String(s), _) => {
													(Some((*s).to_string()), Vec::new())
												}
												(generic_json::Value::Array(values), _) => {
													let mut values = values.into_iter();
													match values.next() {
														Some(first_value) => (
															first_value
																.as_str()
																.map(|v| v.to_string()),
															values.collect(),
														),
														None => (None, values.collect()),
													}
												}
												(other_value, meta) => {
													(None, vec![K::new(other_value, meta)])
												}
											},
											None => (None, Vec::new()),
										}
									}
									_ => (None, Vec::new()),
								};

								// If there are remaining values in `compacted_item`
								// for container key, use `add_value` to add
								// those remaining values to the `container_key`
								// in `compacted_item`.
								// Otherwise, remove that entry from compacted item.
								if !remaining_values.is_empty() {
									if let Some(map) = compacted_item.as_object_mut() {
										for value in remaining_values {
											add_value(
												map,
												container_key.as_ref().unwrap().as_str(),
												value,
												false,
												|| meta(None),
											)
										}
									}
								}

								map_key
							}
						} else if container_type == ContainerKind::Id {
							// Otherwise, if `container` includes @id,
							// set `map_key` to the value of `container_key` in
							// `compacted_item` and remove `container_key` from
							// `compacted_item`.
							compacted_item
								.as_object_mut()
								.and_then(|map| {
									map.remove(container_key.unwrap().as_str())
										.map(|value| value.as_str().map(|s| s.to_string()))
								})
								.flatten()
						} else {
							// Otherwise, if container includes @type:

							// Set `map_key` to the first value of `container_key` in
							// `compacted_item`, if any.
							let (map_key, remaining_values) = match compacted_item.as_object_mut() {
								Some(map) => {
									match map.remove(container_key.as_ref().unwrap().as_str()) {
										Some(value) => match value.into_parts() {
											(generic_json::Value::String(s), _) => {
												(Some((*s).to_string()), Vec::new())
											}
											(generic_json::Value::Array(values), _) => {
												let mut values = values.into_iter();
												match values.next() {
													Some(first_value) => (
														first_value.as_str().map(|v| v.to_string()),
														values.collect(),
													),
													None => (None, values.collect()),
												}
											}
											(other_value, meta) => {
												(None, vec![K::new(other_value, meta)])
											}
										},
										None => (None, Vec::new()),
									}
								}
								_ => (None, Vec::new()),
							};

							// If there are remaining values in `compacted_item` for
							// `container_key`, use `add_value` to add those
							// remaining values to the `container_key` in
							// `compacted_item`.
							// Otherwise, remove that entry from compacted item.
							if !remaining_values.is_empty() {
								if let Some(map) = compacted_item.as_object_mut() {
									for value in remaining_values {
										add_value(
											map,
											container_key.as_ref().unwrap().as_str(),
											value,
											false,
											|| meta(None),
										)
									}
								}
							}

							// If `compacted_item` contains a single entry with a key
							// expanding to @id, set `compacted_item` to the result of
							// using this algorithm recursively,
							// passing `active_context`, `item_active_property` for
							// `active_property`, and a map composed of the single
							// entry for @id from `expanded_item` for `element`.
							if let Some(map) = compacted_item.as_object() {
								if map.len() == 1 && map.get("@id").is_some() {
									let obj: Object<J, T> = Object::Node(Node::with_id(
										expanded_item.id().unwrap().clone(),
									));
									compacted_item = obj
										.compact_indexed(
											None,
											active_context,
											active_context,
											Some(item_active_property.as_str()),
											loader,
											options,
											meta.clone(),
										)
										.await?
								}
							}

							map_key
						};

						// If `map_key` is null, set it to the result of
						// IRI compacting @none.
						let map_key = match map_key {
							Some(key) => key,
							None => {
								let key = compact_iri::<J, _, _>(
									active_context,
									&Term::Keyword(Keyword::None),
									true,
									false,
									options,
								)?;
								key.unwrap()
							}
						};

						// Use `add_value` to add `compacted_item` to
						// the `map_key` entry in `map_object` using `as_array`.
						add_value(map_object, &map_key, compacted_item, as_array, || {
							meta(None)
						})
					} else {
						// Otherwise, use `add_value` to add `compacted_item` to the
						// `item_active_property` entry in `nest_result` using `as_array`.
						add_value(
							nest_result,
							item_active_property.as_str(),
							compacted_item,
							as_array,
							|| meta(None),
						)
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
		let item_active_property = compact_iri_with::<J, _, _, _>(
			active_context,
			&expanded_property,
			&Indexed::new(Object::Node(Node::new()), None),
			true,
			inside_reverse,
			options,
		)?;

		// If the term definition for `item_active_property` in the active context
		// has a nest value entry (nest term):
		if let Some(item_active_property) = item_active_property {
			let (nest_result, _, _) = select_nest_result::<K, _, _, _>(
				result,
				active_context,
				item_active_property.as_str(),
				options.compact_arrays,
				|| meta(None),
			)?;

			// Use `add_value` to add an empty array to the `item_active_property` entry in
			// `nest_result` using true for `as_array`.
			add_value(
				nest_result,
				item_active_property.as_str(),
				K::empty_array(meta(None)),
				true,
				|| meta(None),
			)
		}
	}

	Ok(())
}
