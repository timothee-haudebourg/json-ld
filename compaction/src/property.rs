use crate::{
	add_value, compact_collection_with, compact_iri, compact_iri_with, compact_key, value_value,
	CompactFragment, CompactIndexedFragment, Error, MetaError, Options,
};
use json_ld_context_processing::ProcessMeta;
use json_ld_core::{
	context::Nest,
	object::{self, List},
	Container, ContainerKind, Context, ContextLoader, Indexed, Loader, Node, Object, Reference,
	Term,
};
use json_ld_syntax::Keyword;
use locspan::Meta;
use rdf_types::VocabularyMut;
use std::hash::Hash;

async fn compact_property_list<I, B, M, C, N, L>(
	vocabulary: &mut N,
	Meta(list, meta): Meta<&List<I, B, M>, &M>,
	expanded_index: Option<&json_ld_syntax::Entry<String, M>>,
	nest_result: &mut json_syntax::Object<M>,
	container: Container,
	as_array: bool,
	item_active_property: Meta<&str, &M>,
	active_context: &Context<I, B, C, M>,
	loader: &mut L,
	options: Options,
) -> Result<(), MetaError<M, L::ContextError>>
where
	N: Send + Sync + VocabularyMut<I, B>,
	I: Clone + Hash + Eq + Send + Sync,
	B: Clone + Hash + Eq + Send + Sync,
	M: Clone + Send + Sync,
	C: ProcessMeta<I, B, M>,
	L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
	L::Context: Into<C>,
{
	// If expanded item is a list object:
	let mut compacted_item = compact_collection_with(
		vocabulary,
		Meta(list.iter(), meta),
		active_context,
		active_context,
		Some(item_active_property),
		loader,
		options,
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
		)
		.map_err(Meta::cast)?;
		let mut compacted_item_list_object = json_syntax::Object::default();
		compacted_item_list_object.insert(key.unwrap(), compacted_item);

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
			)
			.map_err(Meta::cast)?;

			let Meta(index_value, meta) = &index.value;

			compacted_item_list_object.insert(
				key.unwrap(),
				Meta(
					json_syntax::Value::String(index_value.as_str().into()),
					meta.clone(),
				),
			);
		}

		compacted_item = Meta(
			json_syntax::Value::Object(compacted_item_list_object),
			meta.clone(),
		);

		// Use add value to add `compacted_item` to
		// the `item_active_property` entry in `nest_result` using `as_array`.
		add_value(nest_result, item_active_property, compacted_item, as_array)
	} else {
		// Otherwise, set the value of the item active property entry in nest result to compacted item.
		nest_result.insert(
			Meta(
				item_active_property.0.into(),
				item_active_property.1.clone(),
			),
			compacted_item,
		);
	}

	Ok(())
}

async fn compact_property_graph<I, B, M, C, N, L>(
	vocabulary: &mut N,
	Meta(node, meta): Meta<&Node<I, B, M>, &M>,
	expanded_index: Option<&json_ld_syntax::Entry<String, M>>,
	nest_result: &mut json_syntax::Object<M>,
	container: Container,
	as_array: bool,
	item_active_property: Meta<&str, &M>,
	active_context: &Context<I, B, C, M>,
	loader: &mut L,
	options: Options,
) -> Result<(), MetaError<M, L::ContextError>>
where
	N: Send + Sync + VocabularyMut<I, B>,
	I: Clone + Hash + Eq + Send + Sync,
	B: Clone + Hash + Eq + Send + Sync,
	M: Clone + Send + Sync,
	C: ProcessMeta<I, B, M>,
	L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
	L::Context: Into<C>,
{
	// If expanded item is a graph object
	let mut compacted_item = node
		.graph()
		.unwrap()
		.compact_fragment_full(
			vocabulary,
			active_context,
			active_context,
			Some(item_active_property),
			loader,
			options,
		)
		.await?;

	// If `container` includes @graph and @id:
	if container.contains(ContainerKind::Graph) && container.contains(ContainerKind::Id) {
		// Initialize `map_object` to the value of `item_active_property`
		// in `nest_result`, initializing it to a new empty map,
		// if necessary.
		if nest_result
			.get_unique(*item_active_property)
			.ok()
			.unwrap()
			.is_none()
		{
			nest_result.insert(
				Meta(
					item_active_property.0.into(),
					item_active_property.1.clone(),
				),
				Meta(json_ld_syntax::Object::default().into(), meta.clone()),
			);
		}

		let map_object = nest_result
			.get_unique_mut(*item_active_property)
			.ok()
			.unwrap()
			.unwrap();
		let map_object = map_object.as_object_mut().unwrap();

		// Initialize `map_key` by IRI compacting the value of @id in
		// `expanded_item` or @none if no such value exists
		// with `vocab` set to false if there is an @id entry in
		// `expanded_item`.
		let (id_value, vocab): (Meta<Term<I, B>, M>, bool) = match node.id_entry() {
			Some(entry) => (entry.value.clone().cast(), false),
			None => (Meta(Term::Keyword(Keyword::None), meta.clone()), true),
		};

		let map_key = compact_iri(
			vocabulary,
			active_context,
			id_value.borrow(),
			vocab,
			false,
			options,
		)
		.map_err(Meta::cast)?
		.unwrap();

		// Use `add_value` to add `compacted_item` to
		// the `map_key` entry in `map_object` using `as_array`.
		add_value(
			map_object,
			map_key.borrow().map(String::as_str),
			compacted_item,
			as_array,
		)
	} else if container.contains(ContainerKind::Graph)
		&& container.contains(ContainerKind::Index)
		&& node.is_simple_graph()
	{
		// Initialize `map_object` to the value of `item_active_property`
		// in `nest_result`, initializing it to a new empty map,
		// if necessary.
		if nest_result
			.get_unique(*item_active_property)
			.ok()
			.unwrap()
			.is_none()
		{
			nest_result.insert(
				Meta(
					item_active_property.0.into(),
					item_active_property.1.clone(),
				),
				Meta(json_ld_syntax::Object::default().into(), meta.clone()),
			);
		}

		let map_object = nest_result
			.get_unique_mut(*item_active_property)
			.ok()
			.unwrap()
			.unwrap();
		let map_object = map_object.as_object_mut().unwrap();

		// Initialize `map_key` the value of @index in `expanded_item`
		// or @none, if no such value exists.
		let map_key = expanded_index
			.map(|e| e.value.borrow().map(String::as_str))
			.unwrap_or_else(|| Meta("@none", meta));

		// Use `add_value` to add `compacted_item` to
		// the `map_key` entry in `map_object` using `as_array`.
		add_value(map_object, map_key, compacted_item, as_array)
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
		compacted_item = match compacted_item {
			Meta(json_syntax::Value::Array(items), items_meta) if items.len() > 1 => {
				let key = compact_iri(
					vocabulary,
					active_context,
					Meta(&Term::Keyword(Keyword::Included), &items_meta),
					true,
					false,
					options,
				)
				.map_err(Meta::cast)?
				.unwrap();
				let mut map = json_syntax::Object::default();
				map.insert(
					key.cast(),
					Meta(json_syntax::Value::Array(items), items_meta.clone()),
				);
				Meta(json_syntax::Value::Object(map), items_meta)
			}
			item => item,
		};

		// Use `add_value` to add `compacted_item` to the
		// `item_active_property` entry in `nest_result` using `as_array`.
		add_value(nest_result, item_active_property, compacted_item, as_array)
	} else {
		// Otherwise, `container` does not include @graph or
		// otherwise does not match one of the previous cases.

		// Set `compacted_item` to a new map containing the key from
		// IRI compacting @graph using the original `compacted_item` as a value.
		let key = compact_iri(
			vocabulary,
			active_context,
			Meta(&Term::Keyword(Keyword::Graph), meta),
			true,
			false,
			options,
		)
		.map_err(Meta::cast)?
		.unwrap();
		let mut map = json_syntax::Object::default();
		map.insert(key.cast(), compacted_item);

		// If `expanded_item` contains an @id entry,
		// add an entry in `compacted_item` using the key from
		// IRI compacting @id using the value of
		// IRI compacting the value of @id in `expanded_item` using
		// false for vocab.
		if let Some(id_entry) = node.id_entry() {
			let key = compact_iri(
				vocabulary,
				active_context,
				Meta(&Term::Keyword(Keyword::Id), &id_entry.key_metadata),
				false,
				false,
				options,
			)
			.map_err(Meta::cast)?
			.unwrap();
			let id: Meta<Term<I, B>, M> = id_entry.value.clone().cast();
			let value = compact_iri(
				vocabulary,
				active_context,
				id.borrow(),
				false,
				false,
				options,
			)
			.map_err(Meta::cast)?;
			map.insert(
				key.cast(),
				match value {
					Some(s) => s.cast(),
					None => Meta(json_syntax::Value::Null, meta.clone()),
				},
			);
		}

		// If `expanded_item` contains an @index entry,
		// add an entry in `compacted_item` using the key from
		// IRI compacting @index and the value of @index in `expanded_item`.
		if let Some(index_entry) = expanded_index {
			let key = compact_iri(
				vocabulary,
				active_context,
				Meta(&Term::Keyword(Keyword::Index), &index_entry.key_metadata),
				true,
				false,
				options,
			)
			.map_err(Meta::cast)?
			.unwrap();
			map.insert(key.cast(), index_entry.value.clone().cast());
		}

		// Use `add_value` to add `compacted_item` to the
		// `item_active_property` entry in `nest_result` using `as_array`.
		let compacted_item = Meta(json_syntax::Value::Object(map), meta.clone());
		add_value(nest_result, item_active_property, compacted_item, as_array)
	}

	Ok(())
}

fn select_nest_result<'a, I, B, M, C, E>(
	result: &'a mut json_syntax::Object<M>,
	active_context: &Context<I, B, C, M>,
	item_active_property: Meta<&str, &M>,
	compact_arrays: bool,
) -> Result<(&'a mut json_syntax::Object<M>, Container, bool), MetaError<M, E>>
where
	I: Clone + Hash + Eq + Send + Sync,
	B: Clone + Hash + Eq + Send + Sync,
	M: Clone + Send + Sync,
{
	let (nest_result, container) = match active_context.get(*item_active_property) {
		Some(term_definition) => {
			let nest_result = match &term_definition.nest {
				Some(nest_term) => {
					// If nest term is not @nest,
					// or a term in the active context that expands to @nest,
					// an invalid @nest value error has been detected,
					// and processing is aborted.
					if *nest_term.value != Nest::Nest {
						match active_context.get(nest_term.as_str()) {
							Some(term_def)
								if term_def.value == Some(Term::Keyword(Keyword::Nest)) => {}
							_ => {
								return Err(Meta(
									Error::InvalidNestValue,
									nest_term.key_metadata.clone(),
								))
							}
						}
					}

					// If result does not have a nest_term entry,
					// initialize it to an empty map.
					let meta = nest_term.key_metadata.clone();
					if result
						.get_unique(nest_term.as_str())
						.ok()
						.unwrap()
						.is_none()
					{
						result.insert(
							nest_term.value.clone().map(|k| k.as_str().into()),
							Meta(json_syntax::Object::default().into(), meta),
						);
					}

					// Initialize `nest_result` to the value of `nest_term` in result.
					let value = result
						.get_unique_mut(nest_term.as_str())
						.ok()
						.unwrap()
						.unwrap();
					let sub_object = value.as_object_mut().unwrap();
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
		|| *item_active_property == "@graph"
		|| *item_active_property == "@list"
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
	active_context: &Context<I, B, C, M>,
	loader: &mut L,
	inside_reverse: bool,
	options: Options,
) -> Result<(), MetaError<M, L::ContextError>>
where
	T: 'a + object::Any<I, B, M> + Sync + Send,
	O: IntoIterator<Item = &'a Meta<Indexed<T, M>, M>>,
	N: Send + Sync + VocabularyMut<I, B>,
	I: Clone + Hash + Eq + Send + Sync,
	B: Clone + Hash + Eq + Send + Sync,
	M: 'a + Clone + Send + Sync,
	C: ProcessMeta<I, B, M>,
	L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
	L::Context: Into<C>,
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
		)
		.map_err(Meta::cast)?;

		// If the term definition for `item_active_property` in the active context
		// has a nest value entry (nest term)
		if let Some(item_active_property) = item_active_property {
			let (nest_result, container, as_array) = select_nest_result(
				result,
				active_context,
				item_active_property.borrow().map(String::as_str),
				options.compact_arrays,
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
					compact_property_list(
						vocabulary,
						Meta(list, expanded_item.metadata()),
						expanded_item.index_entry(),
						nest_result,
						container,
						as_array,
						item_active_property.borrow().map(String::as_str),
						active_context,
						loader,
						options,
					)
					.await?
				}
				object::Ref::Node(node) if node.is_graph() => {
					compact_property_graph(
						vocabulary,
						Meta(node, expanded_item.metadata()),
						expanded_item.index_entry(),
						nest_result,
						container,
						as_array,
						item_active_property.borrow().map(String::as_str),
						active_context,
						loader,
						options,
					)
					.await?
				}
				_ => {
					let mut compacted_item = expanded_item
						.compact_fragment_full(
							vocabulary,
							active_context,
							active_context,
							Some(item_active_property.borrow().map(String::as_str)),
							loader,
							options,
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
						if nest_result
							.get_unique(item_active_property.as_str())
							.ok()
							.unwrap()
							.is_none()
						{
							let meta = item_active_property.metadata().clone();
							nest_result.insert(
								item_active_property.clone().cast(),
								Meta(json_syntax::Object::default().into(), meta),
							);
						}

						let map_object = nest_result
							.get_unique_mut(item_active_property.as_str())
							.ok()
							.unwrap()
							.unwrap();
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

						let mut container_key = compact_iri(
							vocabulary,
							active_context,
							Meta(
								&Term::Keyword(container_type.into()),
								compacted_item.metadata(),
							),
							true,
							false,
							options,
						)
						.map_err(Meta::cast)?;

						// Initialize `index_key` to the value of index mapping in
						// the term definition associated with `item_active_property`
						// in active context, or @index (None in our case), if no such value exists.
						let index_key = match active_context.get(item_active_property.as_str()) {
							Some(def) if def.index.is_some() => def.index.as_ref(),
							_ => None,
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
								compacted_item = value_value(value, expanded_item.metadata())
							}

							expanded_item.language().map(|lang| {
								Meta(lang.to_string(), expanded_item.metadata().clone())
							})
						} else if container_type == ContainerKind::Index {
							match index_key {
								Some(index_key) => {
									// Otherwise, if `container` includes @index and
									// `index_key` is not @index:

									// Reinitialize `container_key` by
									// IRI compacting `index_key`.
									container_key = compact_iri(
										vocabulary,
										active_context,
										Meta(
											&Term::Ref(Reference::Invalid(index_key.to_string())),
											index_key.metadata(),
										),
										true,
										false,
										options,
									)
									.map_err(Meta::cast)?;

									// Set `map_key` to the first value of
									// `container_key` in `compacted_item`, if any.
									let (map_key, remaining_values) = match compacted_item
										.value_mut()
									{
										json_syntax::Value::Object(map) => {
											match map
												.remove_unique(
													container_key.as_ref().unwrap().as_str(),
												)
												.ok()
												.unwrap()
											{
												Some(entry) => match entry.value {
													Meta(json_syntax::Value::String(s), meta) => (
														Some(Meta(s.to_string(), meta.clone())),
														Vec::new(),
													),
													Meta(json_syntax::Value::Array(values), _) => {
														let mut values = values.into_iter();
														match values.next() {
															Some(first_value) => (
																first_value.as_str().map(|v| {
																	Meta(
																		v.to_string(),
																		first_value
																			.metadata()
																			.clone(),
																	)
																}),
																values.collect(),
															),
															None => (None, values.collect()),
														}
													}
													other => (None, vec![other]),
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
													container_key
														.as_ref()
														.unwrap()
														.borrow()
														.map(String::as_str),
													value,
													false,
												)
											}
										}
									}

									map_key
								}
								None => {
									// Otherwise, if `container` includes @index and
									// `index_key` is @index, set `map_key` to the value of
									// @index in `expanded_item`, if any.
									expanded_item.index_entry().map(|entry| {
										Meta(entry.value.to_string(), entry.metadata().clone())
									})
								}
							}
						} else if container_type == ContainerKind::Id {
							// Otherwise, if `container` includes @id,
							// set `map_key` to the value of `container_key` in
							// `compacted_item` and remove `container_key` from
							// `compacted_item`.
							compacted_item
								.as_object_mut()
								.and_then(|map| {
									map.remove_unique(container_key.unwrap().as_str())
										.ok()
										.unwrap()
										.map(|entry| {
											entry.value.as_str().map(|s| {
												Meta(s.to_string(), entry.value.metadata().clone())
											})
										})
								})
								.flatten()
						} else {
							// Otherwise, if container includes @type:

							// Set `map_key` to the first value of `container_key` in
							// `compacted_item`, if any.
							let (map_key, remaining_values) = match compacted_item.as_object_mut() {
								Some(map) => {
									match map
										.remove_unique(container_key.as_ref().unwrap().as_str())
										.ok()
										.unwrap()
									{
										Some(entry) => match entry.value {
											Meta(json_syntax::Value::String(s), meta) => (
												Some(Meta((*s).to_string(), meta.clone())),
												Vec::new(),
											),
											Meta(json_syntax::Value::Array(values), _) => {
												let mut values = values.into_iter();
												match values.next() {
													Some(first_value) => (
														first_value.as_str().map(|v| {
															Meta(
																v.to_string(),
																first_value.metadata().clone(),
															)
														}),
														values.collect(),
													),
													None => (None, values.collect()),
												}
											}
											other => (None, vec![other]),
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
											container_key
												.as_ref()
												.unwrap()
												.borrow()
												.map(String::as_str),
											value,
											false,
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
								if map.len() == 1 && map.get_unique("@id").ok().unwrap().is_some() {
									let obj = Object::Node(Node::with_id(
										expanded_item.id_entry().unwrap().clone(),
									));
									compacted_item = obj
										.compact_indexed_fragment(
											vocabulary,
											compacted_item.metadata(),
											None,
											active_context,
											active_context,
											Some(item_active_property.borrow().map(String::as_str)),
											loader,
											options,
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
								let key = compact_iri(
									vocabulary,
									active_context,
									Meta(&Term::Keyword(Keyword::None), compacted_item.metadata()),
									true,
									false,
									options,
								)
								.map_err(Meta::cast)?;
								key.unwrap()
							}
						};

						// Use `add_value` to add `compacted_item` to
						// the `map_key` entry in `map_object` using `as_array`.
						add_value(
							map_object,
							map_key.borrow().map(String::as_str),
							compacted_item,
							as_array,
						)
					} else {
						// Otherwise, use `add_value` to add `compacted_item` to the
						// `item_active_property` entry in `nest_result` using `as_array`.
						add_value(
							nest_result,
							item_active_property.borrow().map(String::as_str),
							compacted_item,
							as_array,
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
		let item_active_property = compact_iri_with(
			vocabulary,
			active_context,
			expanded_property.borrow(),
			&Indexed::new(Object::Node(Node::new()), None),
			true,
			inside_reverse,
			options,
		)
		.map_err(Meta::cast)?;

		// If the term definition for `item_active_property` in the active context
		// has a nest value entry (nest term):
		if let Some(item_active_property) = item_active_property {
			let (nest_result, _, _) = select_nest_result(
				result,
				active_context,
				item_active_property.borrow().map(String::as_str),
				options.compact_arrays,
			)?;

			// Use `add_value` to add an empty array to the `item_active_property` entry in
			// `nest_result` using true for `as_array`.
			add_value(
				nest_result,
				item_active_property.borrow().map(String::as_str),
				Meta(
					json_syntax::Object::default().into(),
					item_active_property.metadata().clone(),
				),
				true,
			)
		}
	}

	Ok(())
}
