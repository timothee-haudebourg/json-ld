use crate::{
	algorithms::{
		compaction::{
			object::value::{add_value, value_value},
			CompactFragment, CompactIndexedFragment, Compactor,
		},
		ProcessingEnvironment,
	},
	object::{AnyObject, ListObject, Ref},
	syntax::{context::Nest, Container, ContainerItem, Keyword},
	Error, Id, Indexed, NodeObject, Object, Term,
};

impl Compactor<'_> {
	async fn compact_property_list(
		&self,
		env: &mut impl ProcessingEnvironment,
		list: &ListObject,
		expanded_index: Option<&str>,
		nest_result: &mut json_syntax::Object,
		container: Container,
		as_array: bool,
		item_active_property: &str,
	) -> Result<(), Error> {
		// If expanded item is a list object:
		let mut compacted_item = Box::pin(
			self.with_type_scoped_context(self.active_context)
				.with_active_property(Some(item_active_property))
				.compact_collection_with(env, list.iter()),
		)
		.await?;

		// If compacted item is not an array,
		// then set `compacted_item` to an array containing only `compacted_item`.
		if !compacted_item.is_array() {
			let array = vec![compacted_item];
			compacted_item = json_syntax::Value::Array(array)
		}

		// If container does not include @list:
		if !container.contains(ContainerItem::List) {
			// Convert `compacted_item` to a list object by setting it to
			// a map containing an entry where the key is the result of
			// IRI compacting @list and the value is the original
			// compacted item.
			let key = self.compact_key(&Term::Keyword(Keyword::List), true, false)?;
			let mut compacted_item_list_object = json_syntax::Object::default();
			compacted_item_list_object.insert(key.unwrap(), compacted_item);

			// If `expanded_item` contains the entry @index-value,
			// then add an entry to compacted item where the key is
			// the result of IRI compacting @index and value is value.
			if let Some(index) = expanded_index {
				let key = self.compact_key(&Term::Keyword(Keyword::Index), true, false)?;

				compacted_item_list_object
					.insert(key.unwrap(), json_syntax::Value::String(index.into()));
			}

			compacted_item = json_syntax::Value::Object(compacted_item_list_object);

			// Use add value to add `compacted_item` to
			// the `item_active_property` entry in `nest_result` using `as_array`.
			add_value(nest_result, item_active_property, compacted_item, as_array)
		} else {
			// Otherwise, set the value of the item active property entry in nest result to compacted item.
			nest_result.insert(item_active_property, compacted_item);
		}

		Ok(())
	}

	#[allow(clippy::too_many_arguments)]
	async fn compact_property_graph(
		&self,
		env: &mut impl ProcessingEnvironment,
		node: &NodeObject,
		expanded_index: Option<&str>,
		nest_result: &mut json_syntax::Object,
		container: Container,
		as_array: bool,
		item_active_property: &str,
	) -> Result<(), Error> {
		// If expanded item is a graph object
		let mut compacted_item = Box::pin(
			node.graph().unwrap().compact_fragment(
				env,
				&self
					.with_type_scoped_context(self.active_context)
					.with_active_property(Some(item_active_property)),
			),
		)
		.await?;

		// If `container` includes @graph and @id:
		if container.contains(ContainerItem::Graph) && container.contains(ContainerItem::Id) {
			// Initialize `map_object` to the value of `item_active_property`
			// in `nest_result`, initializing it to a new empty map,
			// if necessary.
			if nest_result
				.get_unique(item_active_property)
				.ok()
				.unwrap()
				.is_none()
			{
				nest_result.insert(
					item_active_property,
					crate::syntax::Object::default().into(),
				);
			}

			let map_object = nest_result
				.get_unique_mut(item_active_property)
				.ok()
				.unwrap()
				.unwrap();
			let map_object = map_object.as_object_mut().unwrap();

			// Initialize `map_key` by IRI compacting the value of @id in
			// `expanded_item` or @none if no such value exists
			// with `vocab` set to false if there is an @id entry in
			// `expanded_item`.
			let (id_value, vocab) = match &node.id {
				Some(entry) => (entry.clone().into_term(), false),
				None => (Term::Keyword(Keyword::None), true),
			};

			let map_key = self.compact_iri(&id_value, vocab, false)?.unwrap();

			// Use `add_value` to add `compacted_item` to
			// the `map_key` entry in `map_object` using `as_array`.
			add_value(map_object, &map_key, compacted_item, as_array)
		} else if container.contains(ContainerItem::Graph)
			&& container.contains(ContainerItem::Index)
			&& node.is_simple_graph()
		{
			// Initialize `map_object` to the value of `item_active_property`
			// in `nest_result`, initializing it to a new empty map,
			// if necessary.
			if nest_result
				.get_unique(item_active_property)
				.ok()
				.unwrap()
				.is_none()
			{
				nest_result.insert(
					item_active_property,
					crate::syntax::Object::default().into(),
				);
			}

			let map_object = nest_result
				.get_unique_mut(item_active_property)
				.ok()
				.unwrap()
				.unwrap();
			let map_object = map_object.as_object_mut().unwrap();

			// Initialize `map_key` the value of @index in `expanded_item`
			// or @none, if no such value exists.
			let map_key = expanded_index.unwrap_or("@none");

			// Use `add_value` to add `compacted_item` to
			// the `map_key` entry in `map_object` using `as_array`.
			add_value(map_object, map_key, compacted_item, as_array)
		} else if container.contains(ContainerItem::Graph) && node.is_simple_graph() {
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
				json_syntax::Value::Array(items) if items.len() > 1 => {
					let key = self
						.compact_iri(&Term::Keyword(Keyword::Included), true, false)?
						.unwrap();
					let mut map = json_syntax::Object::default();
					map.insert(key, json_syntax::Value::Array(items));
					json_syntax::Value::Object(map)
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
			let key = self
				.compact_iri(&Term::Keyword(Keyword::Graph), true, false)?
				.unwrap();
			let mut map = json_syntax::Object::default();
			map.insert(key, compacted_item);

			// If `expanded_item` contains an @id entry,
			// add an entry in `compacted_item` using the key from
			// IRI compacting @id using the value of
			// IRI compacting the value of @id in `expanded_item` using
			// false for vocab.
			if let Some(id_entry) = &node.id {
				let key = self
					.compact_iri(&Term::Keyword(Keyword::Id), false, false)?
					.unwrap();
				let id: Term = id_entry.clone().into();
				let value = self.compact_iri(&id, false, false)?;
				map.insert(
					key,
					match value {
						Some(s) => s.into(),
						None => json_syntax::Value::Null,
					},
				);
			}

			// If `expanded_item` contains an @index entry,
			// add an entry in `compacted_item` using the key from
			// IRI compacting @index and the value of @index in `expanded_item`.
			if let Some(index_entry) = expanded_index {
				let key = self
					.compact_iri(&Term::Keyword(Keyword::Index), true, false)?
					.unwrap();
				map.insert(key, index_entry.into());
			}

			// Use `add_value` to add `compacted_item` to the
			// `item_active_property` entry in `nest_result` using `as_array`.
			let compacted_item = json_syntax::Value::Object(map);
			add_value(nest_result, item_active_property, compacted_item, as_array)
		}

		Ok(())
	}

	fn select_nest_result<'a>(
		&self,
		result: &'a mut json_syntax::Object,
		item_active_property: &str,
		compact_arrays: bool,
	) -> Result<(&'a mut json_syntax::Object, Container, bool), Error> {
		let (nest_result, container) = match self.active_context.get(item_active_property) {
			Some(term_definition) => {
				let nest_result = match term_definition.nest() {
					Some(nest_term) => {
						// If nest term is not @nest,
						// or a term in the active context that expands to @nest,
						// an invalid @nest value error has been detected,
						// and processing is aborted.
						if *nest_term != Nest::Nest {
							match self.active_context.get(nest_term.as_str()) {
								Some(term_def)
									if term_def.value() == Some(&Term::Keyword(Keyword::Nest)) => {}
								_ => return Err(Error::InvalidNestValue),
							}
						}

						// If result does not have a nest_term entry,
						// initialize it to an empty map.
						if result
							.get_unique(nest_term.as_str())
							.ok()
							.unwrap()
							.is_none()
						{
							result
								.insert(nest_term.as_str(), json_syntax::Object::default().into());
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

				(nest_result, term_definition.container())
			}
			None => (result, Container::Null),
		};

		// Initialize container to container mapping for item active property
		// in active context, or to a new empty array,
		// if there is no such container mapping.
		// DONE.

		// Initialize `as_array` to true if `container` includes @set,
		// or if `item_active_property` is @graph or @list,
		// otherwise the negation of `options.compact_arrays`.
		let as_array = if container.contains(ContainerItem::Set)
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
	#[allow(clippy::too_many_arguments)]
	pub async fn compact_property<'a, O, T>(
		&self,
		env: &mut impl ProcessingEnvironment,
		result: &mut json_syntax::Object,
		expanded_property: Term,
		expanded_value: O,
		inside_reverse: bool,
	) -> Result<(), Error>
	where
		O: IntoIterator<Item = &'a Indexed<T>>,
		T: 'a + AnyObject,
	{
		let mut is_empty = true;

		// For each item `expanded_item` in `expanded value`
		for expanded_item in expanded_value {
			is_empty = false;
			// Initialize `item_active_property` by IRI compacting `expanded_property`
			// using `expanded_item` for value and `inside_reverse` for `reverse`.
			let item_active_property = self.compact_iri_with(
				&expanded_property,
				true,
				inside_reverse,
				Some(expanded_item),
			)?;

			// If the term definition for `item_active_property` in the active context
			// has a nest value entry (nest term)
			if let Some(item_active_property) = item_active_property {
				let (nest_result, container, as_array) = self.select_nest_result(
					result,
					&item_active_property,
					self.options.compact_arrays,
				)?;

				// Initialize `compacted_item` to the result of using this algorithm
				// recursively, passing `active_context`, `item_active_property` for
				// `active_property`, `expanded_item` for `element`, along with the
				// `compact_arrays` and `ordered_flags`.
				// If `expanded_item` is a list object or a graph object,
				// use the value of the @list or @graph entries, respectively,
				// for `element` instead of `expanded_item`.
				match expanded_item.inner().as_ref() {
					Ref::List(list) => {
						self.compact_property_list(
							env,
							list,
							expanded_item.index(),
							nest_result,
							container,
							as_array,
							&item_active_property,
						)
						.await?
					}
					Ref::Node(node) if node.is_graph() => {
						self.compact_property_graph(
							env,
							node,
							expanded_item.index(),
							nest_result,
							container,
							as_array,
							&item_active_property,
						)
						.await?
					}
					_ => {
						let mut compacted_item = Box::pin(
							expanded_item.compact_fragment(
								env,
								&self
									.with_type_scoped_context(self.active_context)
									.with_active_property(Some(&item_active_property)),
							),
						)
						.await?;

						// if container includes @language, @index, @id,
						// or @type and container does not include @graph:
						if !container.contains(ContainerItem::Graph)
							&& (container.contains(ContainerItem::Language)
								|| container.contains(ContainerItem::Index)
								|| container.contains(ContainerItem::Id)
								|| container.contains(ContainerItem::Type))
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
								nest_result.insert(
									item_active_property.clone(),
									json_syntax::Object::default().into(),
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
							let container_type = if container.contains(ContainerItem::Language) {
								ContainerItem::Language
							} else if container.contains(ContainerItem::Index) {
								ContainerItem::Index
							} else if container.contains(ContainerItem::Id) {
								ContainerItem::Id
							} else {
								ContainerItem::Type
							};

							let mut container_key = self.compact_iri(
								&Term::Keyword(container_type.into()),
								true,
								false,
							)?;

							// Initialize `index_key` to the value of index mapping in
							// the term definition associated with `item_active_property`
							// in active context, or @index (None in our case), if no such value exists.
							let index_key =
								match self.active_context.get(item_active_property.as_str()) {
									Some(def) if def.index().is_some() => def.index(),
									_ => None,
								};

							// If `container` includes @language and `expanded_item`
							// contains a @value entry, then set `compacted_item` to
							// the value associated with its @value entry.
							// Set `map_key` to the value of @language in `expanded_item`,
							// if any.
							let map_key = if container_type == ContainerItem::Language
								&& expanded_item.is_value()
							{
								if let Ref::Value(value) = expanded_item.inner().as_ref() {
									compacted_item = value_value(value)
								}

								expanded_item.language().map(|lang| lang.to_string())
							} else if container_type == ContainerItem::Index {
								match index_key {
									Some(index_key) => {
										// Otherwise, if `container` includes @index and
										// `index_key` is not @index:

										// Reinitialize `container_key` by
										// IRI compacting `index_key`.
										container_key = self.compact_iri(
											&Term::Id(Id::Invalid(index_key.to_string())),
											true,
											false,
										)?;

										// Set `map_key` to the first value of
										// `container_key` in `compacted_item`, if any.
										let (map_key, remaining_values) = match &mut compacted_item
										{
											json_syntax::Value::Object(map) => {
												match map
													.remove_unique(
														container_key.as_ref().unwrap().as_str(),
													)
													.ok()
													.unwrap()
												{
													Some((_, value)) => match value {
														json_syntax::Value::String(s) => {
															(Some(s.to_string()), Vec::new())
														}
														json_syntax::Value::Array(values) => {
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
														container_key.as_deref().unwrap(),
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
										expanded_item.index().map(ToOwned::to_owned)
									}
								}
							} else if container_type == ContainerItem::Id {
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
											.map(|(_, value)| value.as_str().map(ToOwned::to_owned))
									})
									.flatten()
							} else {
								// Otherwise, if container includes @type:

								// Set `map_key` to the first value of `container_key` in
								// `compacted_item`, if any.
								let (map_key, remaining_values) = match compacted_item
									.as_object_mut()
								{
									Some(map) => {
										match map
											.remove_unique(container_key.as_ref().unwrap().as_str())
											.ok()
											.unwrap()
										{
											Some((_, value)) => match value {
												json_syntax::Value::String(s) => {
													(Some((*s).to_string()), Vec::new())
												}
												json_syntax::Value::Array(values) => {
													let mut values = values.into_iter();
													match values.next() {
														Some(first_value) => (
															first_value
																.as_str()
																.map(ToOwned::to_owned),
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
												container_key.as_deref().unwrap(),
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
									if map.len() == 1
										&& map.get_unique("@id").ok().unwrap().is_some()
									{
										let obj = Object::node(NodeObject::new_with_id(Some(
											expanded_item.id().unwrap().clone(),
										)));
										compacted_item = Box::pin(
											obj.compact_indexed_fragment(
												env,
												&self
													.with_type_scoped_context(self.active_context)
													.with_active_property(Some(
														&item_active_property,
													)),
												None,
											),
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
									let key = self.compact_iri(
										&Term::Keyword(Keyword::None),
										true,
										false,
									)?;
									key.unwrap()
								}
							};

							// Use `add_value` to add `compacted_item` to
							// the `map_key` entry in `map_object` using `as_array`.
							add_value(map_object, &map_key, compacted_item, as_array)
						} else {
							// Otherwise, use `add_value` to add `compacted_item` to the
							// `item_active_property` entry in `nest_result` using `as_array`.
							add_value(nest_result, &item_active_property, compacted_item, as_array)
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
			let item_active_property = self.compact_iri_with(
				&expanded_property,
				true,
				inside_reverse,
				Some(&Indexed::new(Object::node(NodeObject::new()), None)),
			)?;

			// If the term definition for `item_active_property` in the active context
			// has a nest value entry (nest term):
			if let Some(item_active_property) = item_active_property {
				let (nest_result, _, _) = self.select_nest_result(
					result,
					&item_active_property,
					self.options.compact_arrays,
				)?;

				// Use `add_value` to add an empty array to the `item_active_property` entry in
				// `nest_result` using true for `as_array`.
				add_value(nest_result, &item_active_property, Vec::new().into(), true)
			}
		}

		Ok(())
	}
}
