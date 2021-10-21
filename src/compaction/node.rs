use super::{add_value, compact_iri, compact_property, optional_string, JsonSrc, Options};
use crate::{
	context::{self, Inversible, Loader, Local},
	syntax::{Container, ContainerType, Keyword, Term, Type},
	util::{AsAnyJson, JsonFrom},
	ContextMut, Error, Id, Node, ProcessingMode, Reference,
};
use cc_traits::Len;

/// Compact the given indexed node.
pub async fn compact_indexed_node_with<
	J: JsonSrc,
	K: JsonFrom<J>,
	T: Sync + Send + Id,
	C: ContextMut<T>,
	L: Loader,
	M,
>(
	node: &Node<J, T>,
	index: Option<&str>,
	mut active_context: Inversible<T, &C>,
	type_scoped_context: Inversible<T, &C>,
	active_property: Option<&str>,
	loader: &mut L,
	options: Options,
	meta: M,
) -> Result<K, Error>
where
	C: Sync + Send,
	C::LocalContext: Send + Sync + From<L::Output>,
	L: Sync + Send,
	M: Send + Sync + Clone + Fn(Option<&J::MetaData>) -> K::MetaData,
{
	// If active context has a previous context, the active context is not propagated.
	// If element does not contain an @value entry, and element does not consist of
	// a single @id entry, set active context to previous context from active context,
	// as the scope of a term-scoped context does not apply when processing new node objects.
	if !(node.is_empty() && node.id().is_some()) {
		// does not consist of a single @id entry
		if let Some(previous_context) = active_context.previous_context() {
			active_context = Inversible::new(previous_context)
		}
	}

	// If the term definition for active property in active context has a local context:
	// FIXME https://github.com/w3c/json-ld-api/issues/502
	//       Seems that the term definition should be looked up in `type_scoped_context`.
	let mut active_context = active_context.into_borrowed();
	if let Some(active_property) = active_property {
		if let Some(active_property_definition) = type_scoped_context.get(active_property) {
			if let Some(local_context) = &active_property_definition.context {
				active_context = Inversible::new(
					local_context
						.process_with(
							*active_context.as_ref(),
							loader,
							active_property_definition.base_url(),
							context::ProcessingOptions::from(options).with_override(),
						)
						.await?
						.into_inner(),
				)
				.into_owned()
			}
		}
	}

	// let inside_reverse = active_property == Some("@reverse");
	let mut result = K::Object::default();

	if !node.types().is_empty() {
		// If element has an @type entry, create a new array compacted types initialized by
		// transforming each expanded type of that entry into its compacted form by IRI
		// compacting expanded type. Then, for each term in compacted types ordered
		// lexicographically:
		let mut compacted_types = Vec::new();
		for ty in node.types() {
			let compacted_ty = compact_iri::<J, _, _>(
				type_scoped_context.clone(),
				&ty.clone().into_term(),
				true,
				false,
				options,
			)?;
			compacted_types.push(compacted_ty)
		}

		compacted_types.sort_by(|a, b| a.as_ref().unwrap().cmp(b.as_ref().unwrap()));

		for term in &compacted_types {
			if let Some(term_definition) = type_scoped_context.get(term.as_ref().unwrap().as_str())
			{
				if let Some(local_context) = &term_definition.context {
					let processing_options =
						context::ProcessingOptions::from(options).without_propagation();
					active_context = Inversible::new(
						local_context
							.process_with(
								*active_context.as_ref(),
								loader,
								term_definition.base_url(),
								processing_options,
							)
							.await?
							.into_inner(),
					)
					.into_owned()
				}
			}
		}
	}

	// For each key expanded property and value expanded value in element, ordered
	// lexicographically by expanded property if ordered is true:
	let mut expanded_entries: Vec<_> = node.properties.iter().collect();
	if options.ordered {
		expanded_entries.sort_by(|(a, _), (b, _)| a.as_str().cmp(b.as_str()))
	}

	// If expanded property is @id:
	if let Some(id) = &node.id {
		let id = id.clone().into_term();

		if node.is_empty() {
			// This captures step 7:
			// If element has an @value or @id entry and the result of using the
			// Value Compaction algorithm, passing active context, active property,
			// and element as value is a scalar, or the term definition for active property
			// has a type mapping of @json, return that result.
			//
			// in the Value Compaction Algorithm, step 7:
			// If value has an @id entry and has no other entries other than @index:
			//
			// If the type mapping of active property is set to @id,
			// set result to the result of IRI compacting the value associated with the
			// @id entry using false for vocab.
			let type_mapping = match active_property {
				Some(prop) => match active_context.get(prop) {
					Some(def) => def.typ.as_ref(),
					None => None,
				},
				None => None,
			};

			if type_mapping == Some(&Type::Id) {
				let compacted_value =
					compact_iri::<J, _, _>(active_context.as_ref(), &id, false, false, options)?;
				return Ok(compacted_value
					.map(|s| generic_json::Value::String(s.as_str().into()))
					.unwrap_or(generic_json::Value::Null)
					.with(meta(None)));
			}

			// Otherwise, if the type mapping of active property is set to @vocab,
			// set result to the result of IRI compacting the value associated with the @id entry.
			if type_mapping == Some(&Type::Vocab) {
				let compacted_value =
					compact_iri::<J, _, _>(active_context.as_ref(), &id, true, false, options)?;
				return Ok(optional_string(compacted_value, meta(None)));
			}
		}

		// If expanded value is a string, then initialize compacted value by IRI
		// compacting expanded value with vocab set to false.
		let compacted_value =
			compact_iri::<J, _, _>(active_context.as_ref(), &id, false, false, options)?;

		// Initialize alias by IRI compacting expanded property.
		let alias = compact_iri::<J, _, _>(
			active_context.as_ref(),
			&Term::Keyword(Keyword::Id),
			true,
			false,
			options,
		)?;

		// Add an entry alias to result whose value is set to compacted value and continue
		// to the next expanded property.
		if let Some(key) = alias {
			result.insert(
				K::new_key(key.as_str(), meta(None)),
				optional_string(compacted_value, meta(None)),
			);
		}
	}

	compact_types::<J, K, _, _, _>(
		&mut result,
		&node.types,
		active_context.as_ref(),
		type_scoped_context.clone(),
		options,
		meta.clone(),
	)?;

	// If expanded property is @reverse:
	if !node.reverse_properties.is_empty() {
		// Initialize compacted value to the result of using this algorithm recursively,
		// passing active context, @reverse for active property,
		// expanded value for element, and the compactArrays and ordered flags.
		let active_property = "@reverse";
		if let Some(active_property_definition) = active_context.get(active_property) {
			if let Some(local_context) = &active_property_definition.context {
				active_context = Inversible::new(
					local_context
						.process_with(
							*active_context.as_ref(),
							loader,
							active_property_definition.base_url(),
							context::ProcessingOptions::from(options).with_override(),
						)
						.await?
						.into_inner(),
				)
				.into_owned()
			}
		}

		let mut reverse_result = K::Object::default();
		for (expanded_property, expanded_value) in &node.reverse_properties {
			compact_property::<J, K, _, _, _, _, _, _>(
				&mut reverse_result,
				expanded_property.clone().into(),
				expanded_value,
				active_context.as_ref(),
				loader,
				true,
				options,
				meta.clone(),
			)
			.await?;
		}

		// For each property and value in compacted value:
		let mut reverse_map = K::Object::default();
		for (property, mut mapped_value) in reverse_result.iter_mut() {
			let mut value = K::null(meta(None));
			std::mem::swap(&mut value, &mut *mapped_value);

			// If the term definition for property in the active context indicates that
			// property is a reverse property
			if let Some(term_definition) = active_context.get(property.as_ref()) {
				if term_definition.reverse_property {
					// Initialize as array to true if the container mapping for property in
					// the active context includes @set, otherwise the negation of compactArrays.
					let as_array = term_definition.container.contains(ContainerType::Set)
						|| !options.compact_arrays;

					// Use add value to add value to the property entry in result using as array.
					add_value(&mut result, property.as_ref(), value, as_array, || {
						meta(None)
					});
					continue;
				}
			}

			reverse_map.insert(K::new_key(&**property, meta(None)), value);
		}

		if !reverse_map.is_empty() {
			// Initialize alias by IRI compacting @reverse.
			let alias = compact_iri::<J, _, _>(
				active_context.as_ref(),
				&Term::Keyword(Keyword::Reverse),
				true,
				false,
				options,
			)?;

			// Set the value of the alias entry of result to compacted value.
			result.insert(
				K::new_key(alias.unwrap().as_str(), meta(None)),
				K::object(reverse_map, meta(None)),
			);
		}
	}

	// If expanded property is @index and active property has a container mapping in
	// active context that includes @index,
	if let Some(index) = index {
		let mut index_container = false;
		if let Some(active_property) = active_property {
			if let Some(active_property_definition) = active_context.get(active_property) {
				if active_property_definition
					.container
					.contains(ContainerType::Index)
				{
					// then the compacted result will be inside of an @index container,
					// drop the @index entry by continuing to the next expanded property.
					index_container = true;
				}
			}
		}

		if !index_container {
			// Initialize alias by IRI compacting expanded property.
			let alias = compact_iri::<J, _, _>(
				active_context.as_ref(),
				&Term::Keyword(Keyword::Index),
				true,
				false,
				options,
			)?;

			// Add an entry alias to result whose value is set to expanded value and continue with the next expanded property.
			result.insert(
				K::new_key(alias.unwrap().as_str(), meta(None)),
				index.as_json_with(meta(None)),
			);
		}
	}

	if let Some(graph) = &node.graph {
		compact_property::<J, K, _, _, _, _, _, _>(
			&mut result,
			Term::Keyword(Keyword::Graph),
			graph,
			active_context.as_ref(),
			loader,
			false,
			options,
			meta.clone(),
		)
		.await?
	}

	for (expanded_property, expanded_value) in expanded_entries {
		compact_property::<J, K, _, _, _, _, _, _>(
			&mut result,
			expanded_property.clone().into(),
			expanded_value,
			active_context.as_ref(),
			loader,
			false,
			options,
			meta.clone(),
		)
		.await?
	}

	if let Some(included) = &node.included {
		compact_property::<J, K, _, _, _, _, _, _>(
			&mut result,
			Term::Keyword(Keyword::Included),
			included,
			active_context.as_ref(),
			loader,
			false,
			options,
			meta.clone(),
		)
		.await?
	}

	Ok(K::object(result, meta(None)))
}

/// Compact the given list of types into the given `result` compacted object.
fn compact_types<
	J: JsonSrc,
	K: JsonFrom<J>,
	T: Sync + Send + Id,
	C: ContextMut<T>,
	M: Clone + Fn(Option<&J::MetaData>) -> K::MetaData,
>(
	result: &mut K::Object,
	types: &[Reference<T>],
	active_context: Inversible<T, &C>,
	type_scoped_context: Inversible<T, &C>,
	options: Options,
	meta: M,
) -> Result<(), Error> {
	// If expanded property is @type:
	if !types.is_empty() {
		// If expanded value is a string,
		// then initialize compacted value by IRI compacting expanded value using
		// type-scoped context for active context.
		let compacted_value = if types.len() == 1 {
			optional_string(
				compact_iri::<J, _, _>(
					type_scoped_context.clone(),
					&types[0].clone().into_term(),
					true,
					false,
					options,
				)?,
				meta(None),
			)
		} else {
			// Otherwise, expanded value must be a @type array:
			// Initialize compacted value to an empty array.
			let mut compacted_value = Vec::with_capacity(types.len());

			// For each item expanded type in expanded value:
			for ty in types {
				let ty = ty.clone().into_term();

				// Set term by IRI compacting expanded type using type-scoped context for active context.
				let compacted_ty =
					compact_iri::<J, _, _>(type_scoped_context.clone(), &ty, true, false, options)?;

				// Append term, to compacted value.
				compacted_value.push(optional_string(compacted_ty, meta(None)))
			}

			K::array(compacted_value.into_iter().collect(), meta(None))
		};

		// Initialize alias by IRI compacting expanded property.
		let alias = compact_iri::<J, _, _>(
			active_context.clone(),
			&Term::Keyword(Keyword::Type),
			true,
			false,
			options,
		)?
		.unwrap();

		// Initialize as array to true if processing mode is json-ld-1.1 and the
		// container mapping for alias in the active context includes @set,
		// otherwise to the negation of compactArrays.
		let container_mapping = match active_context.get(alias.as_str()) {
			Some(def) => def.container,
			None => Container::None,
		};
		let as_array = (options.processing_mode == ProcessingMode::JsonLd1_1
			&& container_mapping.contains(ContainerType::Set))
			|| !options.compact_arrays;

		// Use add value to add compacted value to the alias entry in result using as array.
		add_value(result, alias.as_str(), compacted_value, as_array, || {
			meta(None)
		})
	}

	Ok(())
}
