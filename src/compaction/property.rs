use super::{
	add_value, compact_collection_with, compact_iri, compact_iri_with, value_value, Compact,
	CompactIndexed, JsonSrc, Options,
};
use crate::{
	context::{Inversible, Loader},
	object,
	syntax::{Container, ContainerType, Keyword, Term},
	util::JsonFrom,
	ContextMut, Error, ErrorCode, Id, Indexed, Node, Object, Reference,
};
use cc_traits::Len;
use generic_json::{JsonBuild, JsonClone, JsonHash, JsonIntoMut, JsonMut, ValueMut};

async fn compact_property_list<
	J: JsonClone + JsonHash,
	K: JsonFrom<J>,
	T: Sync + Send + Id,
	C: ContextMut<T>,
	L: Loader,
	M,
>(
	list: &[Indexed<Object<J, T>>],
	expanded_index: Option<&str>,
	nest_result: &mut K::Object,
	container: Container,
	as_array: bool,
	item_active_property: &str,
	active_context: Inversible<T, &C>,
	loader: &mut L,
	options: Options,
	meta: M,
) -> Result<(), Error>
where
	J: JsonSrc,
	C: Sync + Send,
	C::LocalContext: Send + Sync + From<L::Output>,
	L: Sync + Send,
	M: Send + Sync + Clone + Fn(Option<&J::MetaData>) -> K::MetaData,
{
	// If expanded item is a list object:
	let mut compacted_item: K = compact_collection_with(
		list.iter(),
		active_context.clone(),
		active_context.clone(),
		Some(item_active_property),
		loader,
		options,
		meta.clone(),
	)
	.await?;

	// If compacted item is not an array,
	// then set `compacted_item` to an array containing only `compacted_item`.
	if !compacted_item.is_array() {
		let mut array = K::Array::default();
		array.push_back(compacted_item);
		compacted_item = K::array(array, meta(None))
	}

	// If container does not include @list:
	if !container.contains(ContainerType::List) {
		// Convert `compacted_item` to a list object by setting it to
		// a map containing an entry where the key is the result of
		// IRI compacting @list and the value is the original
		// compacted item.
		let key = compact_iri::<J, T, C>(
			active_context.clone(),
			&Term::Keyword(Keyword::List),
			true,
			false,
			options,
		)?;
		let mut compacted_item_list_object = K::Object::default();
		compacted_item_list_object.insert(key.unwrap().as_str().into(), compacted_item);

		// If `expanded_item` contains the entry @index-value,
		// then add an entry to compacted item where the key is
		// the result of IRI compacting @index and value is value.
		if let Some(index) = expanded_index {
			let key = compact_iri::<J, T, C>(
				active_context.clone(),
				&Term::Keyword(Keyword::Index),
				true,
				false,
				options,
			)?;
			compacted_item_list_object.insert(
				key.unwrap().as_str().into(),
				K::string(index.into(), meta(None)),
			);
		}

		compacted_item = K::object(compacted_item_list_object, meta(None));

		// Use add value to add `compacted_item` to
		// the `item_active_property` entry in `nest_result` using `as_array`.
		add_value(
			nest_result,
			item_active_property,
			compacted_item,
			as_array,
			|| meta(None),
		)
	} else {
		// Otherwise, set the value of the item active property entry in nest result to compacted item.
		nest_result.insert(item_active_property.into(), compacted_item);
	}

	Ok(())
}

async fn compact_property_graph<
	J: JsonSrc,
	K: JsonFrom<J>,
	T: Sync + Send + Id,
	C: ContextMut<T>,
	L: Loader,
	M,
>(
	node: &Node<J, T>,
	expanded_index: Option<&str>,
	nest_result: &mut K::Object,
	container: Container,
	as_array: bool,
	item_active_property: &str,
	active_context: Inversible<T, &C>,
	loader: &mut L,
	options: Options,
	meta: M,
) -> Result<(), Error>
where
	C: Sync + Send,
	C::LocalContext: Send + Sync + From<L::Output>,
	L: Sync + Send,
	M: Send + Sync + Clone + Fn(Option<&J::MetaData>) -> K::MetaData,
{
	// If expanded item is a graph object
	let mut compacted_item: K = node
		.graph
		.as_ref()
		.unwrap()
		.compact_with(
			active_context.clone(),
			active_context.clone(),
			Some(item_active_property),
			loader,
			options,
			meta.clone(),
		)
		.await?;

	// If `container` includes @graph and @id:
	if container.contains(ContainerType::Graph) && container.contains(ContainerType::Id) {
		// Initialize `map_object` to the value of `item_active_property`
		// in `nest_result`, initializing it to a new empty map,
		// if necessary.
		if nest_result.get(item_active_property).is_none() {
			nest_result.insert(
				item_active_property.into(),
				K::object(K::Object::default(), meta(None)),
			);
		}

		let mut map_object = nest_result.get_mut(item_active_property).unwrap();
		let map_object = map_object.as_object_mut().unwrap();

		// Initialize `map_key` by IRI compacting the value of @id in
		// `expanded_item` or @none if no such value exists
		// with `vocab` set to false if there is an @id entry in
		// `expanded_item`.
		let (id_value, vocab): (Term<T>, bool) = match node.id() {
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
	} else if container.contains(ContainerType::Graph)
		&& container.contains(ContainerType::Index)
		&& node.is_simple_graph()
	{
		// Initialize `map_object` to the value of `item_active_property`
		// in `nest_result`, initializing it to a new empty map,
		// if necessary.
		if nest_result.get(item_active_property).is_none() {
			nest_result.insert(
				item_active_property.into(),
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
				map.insert(key.as_str().into(), K::array(items, items_meta));
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
			active_context.clone(),
			&Term::Keyword(Keyword::Graph),
			true,
			false,
			options,
		)?
		.unwrap();
		let mut map = K::Object::default();
		map.insert(key.as_str().into(), compacted_item);

		// If `expanded_item` contains an @id entry,
		// add an entry in `compacted_item` using the key from
		// IRI compacting @id using the value of
		// IRI compacting the value of @id in `expanded_item` using
		// false for vocab.
		if let Some(id) = node.id() {
			let key = compact_iri::<J, _, _>(
				active_context.clone(),
				&Term::Keyword(Keyword::Id),
				false,
				false,
				options,
			)?
			.unwrap();
			let value = compact_iri::<J, _, _>(
				active_context.clone(),
				&id.clone().into_term(),
				false,
				false,
				options,
			)?;
			map.insert(
				key.as_str().into(),
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
				active_context.clone(),
				&Term::Keyword(Keyword::Index),
				true,
				false,
				options,
			)?
			.unwrap();
			map.insert(key.as_str().into(), K::string(index.into(), meta(None)));
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

// pub enum SubObject<'o, K: JsonMut> {
// 	Root(&'o mut K::Object),
// 	Sub(<K::Object as cc_traits::CollectionMut>::ItemMut<'o>, &'o mut K::Object)
// }

// impl<'o, K: JsonMut> std::ops::Deref for SubObject<'o, K> {
// 	type Target = K::Object;

// 	fn deref(&self) -> &Self::Target {
// 		match self {
// 			Self::Root(o) => o,
// 			Self::Sub(_, o) => o
// 		}
// 	}
// }

// impl<'o, K: JsonMut> std::ops::DerefMut for SubObject<'o, K> {
// 	fn deref_mut(&mut self) -> &mut Self::Target {
// 		match self {
// 			Self::Root(o) => o,
// 			Self::Sub(_, o) => o
// 		}
// 	}
// }

fn select_nest_result<'a, K: 'a + JsonBuild + JsonMut + JsonIntoMut, T: Id, C: ContextMut<T>, M>(
	result: &'a mut K::Object,
	active_context: Inversible<T, &C>,
	item_active_property: &str,
	compact_arrays: bool,
	meta: M,
) -> Result<(&'a mut K::Object, Container, bool), Error>
where
	M: Fn() -> K::MetaData,
{
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
							Some(term_def)
								if term_def.value == Some(Term::Keyword(Keyword::Nest)) => {}
							_ => return Err(ErrorCode::InvalidNestValue.into()),
						}
					}

					// If result does not have a nest_term entry,
					// initialize it to an empty map.
					if result.get(nest_term).is_none() {
						result.insert(
							nest_term.as_str().into(),
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
	let as_array = if container.contains(ContainerType::Set)
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
pub async fn compact_property<
	'a,
	J: JsonSrc,
	K: JsonFrom<J>,
	T: 'a + Sync + Send + Id,
	N: 'a + object::Any<J, T> + Sync + Send,
	O: IntoIterator<Item = &'a Indexed<N>>,
	C: ContextMut<T>,
	L: Loader,
	M: Send + Sync + Clone + Fn(Option<&J::MetaData>) -> K::MetaData,
>(
	result: &mut K::Object,
	expanded_property: Term<T>,
	expanded_value: O,
	active_context: Inversible<T, &C>,
	loader: &mut L,
	inside_reverse: bool,
	options: Options,
	meta: M,
) -> Result<(), Error>
where
	C: Sync + Send,
	C::LocalContext: Send + Sync + From<L::Output>,
	L: Sync + Send,
{
	let mut is_empty = true;

	// For each item `expanded_item` in `expanded value`
	for expanded_item in expanded_value {
		is_empty = false;
		// Initialize `item_active_property` by IRI compacting `expanded_property`
		// using `expanded_item` for value and `inside_reverse` for `reverse`.
		let item_active_property = compact_iri_with(
			active_context.clone(),
			&expanded_property,
			expanded_item,
			true,
			inside_reverse,
			options,
		)?;

		// If the term definition for `item_active_property` in the active context
		// has a nest value entry (nest term)
		if let Some(item_active_property) = item_active_property {
			let (nest_result, container, as_array): (&'_ mut K::Object, _, _) =
				select_nest_result::<K, _, _, _>(
					result,
					active_context.clone(),
					item_active_property.as_str(),
					options.compact_arrays,
					|| meta(None),
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
						active_context.clone(),
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
						active_context.clone(),
						loader,
						options,
						meta.clone(),
					)
					.await?
				}
				_ => {
					let mut compacted_item: K = expanded_item
						.compact_with(
							active_context.clone(),
							active_context.clone(),
							Some(item_active_property.as_str()),
							loader,
							options,
							meta.clone(),
						)
						.await?;

					// if container includes @language, @index, @id,
					// or @type and container does not include @graph:
					if !container.contains(ContainerType::Graph)
						&& (container.contains(ContainerType::Language)
							|| container.contains(ContainerType::Index)
							|| container.contains(ContainerType::Id)
							|| container.contains(ContainerType::Type))
					{
						// Initialize `map_object` to the value of
						// `item_active_property` in `nest_result`,
						// initializing it to a new empty map, if necessary.
						if nest_result.get(item_active_property.as_str()).is_none() {
							nest_result.insert(
								item_active_property.as_str().into(),
								K::empty_object(meta(None)),
							);
						}

						let mut map_object =
							nest_result.get_mut(item_active_property.as_str()).unwrap();
						let map_object = map_object.as_object_mut().unwrap();

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

						let mut container_key = compact_iri::<J, _, _>(
							active_context.clone(),
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
						let map_key = if container_type == ContainerType::Language
							&& expanded_item.is_value()
						{
							if let object::Ref::Value(value) = expanded_item.inner().as_ref() {
								compacted_item = value_value(value, meta.clone())
							}

							expanded_item.language().map(|lang| lang.to_string())
						} else if container_type == ContainerType::Index {
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
								let lenient_index: Term<T> =
									Term::Ref(Reference::Invalid(index_key.to_string()));
								container_key = compact_iri::<J, _, _>(
									active_context.clone(),
									&lenient_index,
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
													(Some(s.as_ref().to_string()), Vec::new())
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
						} else if container_type == ContainerType::Id {
							// Otherwise, if `container` includes @id,
							// set `map_key` to the value of `container_key` in
							// `compacted_item` and remove `container_key` from
							// `compacted_item`.
							compacted_item
								.as_object_mut()
								.map(|map| {
									map.remove(container_key.unwrap().as_str())
										.map(|value| value.as_str().map(|s| s.to_string()))
								})
								.flatten()
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
												(Some(s.as_ref().to_string()), Vec::new())
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
										.compact_indexed_with(
											None,
											active_context.clone(),
											active_context.clone(),
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
									active_context.clone(),
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
						add_value(map_object, &map_key, compacted_item, as_array, || meta(None))
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
			active_context.clone(),
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
				active_context.clone(),
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
