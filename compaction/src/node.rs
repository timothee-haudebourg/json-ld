use crate::{add_value, compact_iri, compact_property, MetaError, Options};
use contextual::WithContext;
use json_ld_context_processing::{
	Options as ProcessingOptions, Process, ProcessMeta, ProcessingMode,
};
use json_ld_core::{
	object::node::TypeEntry, Container, ContainerKind, Context, ContextLoader, Loader, Node,
	Reference, Term, Type,
};
use json_ld_syntax::{Entry, Keyword};
use locspan::{Meta, Stripped};
use mown::Mown;
use rdf_types::VocabularyMut;
use std::hash::Hash;

fn optional_string<M: Clone>(s: Option<Meta<String, M>>, meta: &M) -> json_syntax::MetaValue<M> {
	s.map(Meta::cast)
		.unwrap_or_else(|| Meta(json_syntax::Value::Null, meta.clone()))
}

/// Compact the given indexed node.
pub async fn compact_indexed_node_with<I, B, M, C, N, L>(
	vocabulary: &mut N,
	Meta(node, meta): Meta<&Node<I, B, M>, &M>,
	index: Option<&Entry<String, M>>,
	mut active_context: &Context<I, B, C, M>,
	type_scoped_context: &Context<I, B, C, M>,
	active_property: Option<Meta<&str, &M>>,
	loader: &mut L,
	options: Options,
) -> Result<json_syntax::MetaValue<M>, MetaError<M, L::ContextError>>
where
	N: Send + Sync + VocabularyMut<I, B>,
	I: Clone + Hash + Eq + Send + Sync,
	B: Clone + Hash + Eq + Send + Sync,
	M: Clone + Send + Sync,
	C: ProcessMeta<I, B, M>,
	L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
	L::Context: Into<C>,
{
	// If active context has a previous context, the active context is not propagated.
	// If element does not contain an @value entry, and element does not consist of
	// a single @id entry, set active context to previous context from active context,
	// as the scope of a term-scoped context does not apply when processing new node objects.
	if !(node.is_empty() && node.id().is_some()) {
		// does not consist of a single @id entry
		if let Some(previous_context) = active_context.previous_context() {
			active_context = previous_context
		}
	}

	// If the term definition for active property in active context has a local context:
	// FIXME https://github.com/w3c/json-ld-api/issues/502
	//       Seems that the term definition should be looked up in `type_scoped_context`.
	let mut active_context = Mown::Borrowed(active_context);
	if let Some(active_property) = active_property {
		if let Some(active_property_definition) = type_scoped_context.get(*active_property) {
			if let Some(local_context) = &active_property_definition.context {
				active_context = Mown::Owned(
					local_context
						.process_with(
							vocabulary,
							active_context.as_ref(),
							loader,
							active_property_definition.base_url().cloned(),
							ProcessingOptions::from(options).with_override(),
						)
						.await
						.map_err(Meta::cast)?
						.into_processed(),
				)
			}
		}
	}

	// let inside_reverse = active_property == Some("@reverse");
	let mut result = json_syntax::Object::default();

	if !node.types().is_empty() {
		// If element has an @type entry, create a new array compacted types initialized by
		// transforming each expanded type of that entry into its compacted form by IRI
		// compacting expanded type. Then, for each term in compacted types ordered
		// lexicographically:
		let mut compacted_types = Vec::new();
		for ty in node.types() {
			let compacted_ty = compact_iri(
				vocabulary,
				type_scoped_context,
				ty.clone().map(Reference::into_term).borrow(),
				true,
				false,
				options,
			)
			.map_err(Meta::cast)?;
			compacted_types.push(compacted_ty)
		}

		compacted_types.sort_by(|a, b| a.as_ref().unwrap().cmp(b.as_ref().unwrap()));

		for term in &compacted_types {
			if let Some(term_definition) = type_scoped_context.get(term.as_ref().unwrap().as_str())
			{
				if let Some(local_context) = &term_definition.context {
					let processing_options = ProcessingOptions::from(options).without_propagation();
					active_context = Mown::Owned(
						local_context
							.process_with(
								vocabulary,
								active_context.as_ref(),
								loader,
								term_definition.base_url().cloned(),
								processing_options,
							)
							.await
							.map_err(Meta::cast)?
							.into_processed(),
					)
				}
			}
		}
	}

	// For each key expanded property and value expanded value in element, ordered
	// lexicographically by expanded property if ordered is true:
	let mut expanded_entries: Vec<_> = node.properties().iter().collect();
	if options.ordered {
		let vocabulary: &N = vocabulary;
		expanded_entries.sort_by(|(a, _), (b, _)| {
			(**a)
				.with(vocabulary)
				.as_str()
				.cmp((**b).with(vocabulary).as_str())
		})
	}

	// If expanded property is @id:
	if let Some(id_entry) = node.id_entry() {
		let id = id_entry.value.clone().map(Reference::into_term);

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
				Some(prop) => match active_context.get(*prop) {
					Some(def) => def.typ.as_ref(),
					None => None,
				},
				None => None,
			};

			if type_mapping == Some(&Type::Id) {
				let compacted_value = compact_iri(
					vocabulary,
					active_context.as_ref(),
					id.borrow(),
					false,
					false,
					options,
				)
				.map_err(Meta::cast)?;
				return Ok(optional_string(compacted_value, id.metadata()));
			}

			// Otherwise, if the type mapping of active property is set to @vocab,
			// set result to the result of IRI compacting the value associated with the @id entry.
			if type_mapping == Some(&Type::Vocab) {
				let compacted_value = compact_iri(
					vocabulary,
					active_context.as_ref(),
					id.borrow(),
					true,
					false,
					options,
				)
				.map_err(Meta::cast)?;
				return Ok(optional_string(compacted_value, id.metadata()));
			}
		}

		// If expanded value is a string, then initialize compacted value by IRI
		// compacting expanded value with vocab set to false.
		let compacted_value = compact_iri(
			vocabulary,
			active_context.as_ref(),
			id.borrow(),
			false,
			false,
			options,
		)
		.map_err(Meta::cast)?;

		// Initialize alias by IRI compacting expanded property.
		let alias = compact_iri(
			vocabulary,
			active_context.as_ref(),
			Meta(&Term::Keyword(Keyword::Id), &id_entry.key_metadata),
			true,
			false,
			options,
		)
		.map_err(Meta::cast)?;

		// Add an entry alias to result whose value is set to compacted value and continue
		// to the next expanded property.
		if let Some(key) = alias {
			result.insert(
				key.clone().cast(),
				optional_string(compacted_value, id.metadata()),
			);
		}
	}

	compact_types(
		vocabulary,
		&mut result,
		node.type_entry(),
		active_context.as_ref(),
		type_scoped_context,
		options,
	)?;

	// If expanded property is @reverse:
	if let Some(reverse_properties) = node.reverse_properties_entry() {
		if !reverse_properties.is_empty() {
			// Initialize compacted value to the result of using this algorithm recursively,
			// passing active context, @reverse for active property,
			// expanded value for element, and the compactArrays and ordered flags.
			let active_property = "@reverse";
			if let Some(active_property_definition) = active_context.get(active_property) {
				if let Some(local_context) = &active_property_definition.context {
					active_context = Mown::Owned(
						local_context
							.value
							.process_with(
								vocabulary,
								active_context.as_ref(),
								loader,
								active_property_definition.base_url().cloned(),
								ProcessingOptions::from(options).with_override(),
							)
							.await
							.map_err(Meta::cast)?
							.into_processed(),
					)
				}
			}

			let mut reverse_result = json_syntax::Object::default();
			for (expanded_property, expanded_value) in reverse_properties.iter() {
				compact_property(
					vocabulary,
					&mut reverse_result,
					expanded_property.cloned().cast(),
					expanded_value.iter().map(Stripped::as_ref),
					active_context.as_ref(),
					loader,
					true,
					options,
				)
				.await?;
			}

			// For each property and value in compacted value:
			let mut reverse_map = json_syntax::Object::default();
			for (property, mapped_value) in reverse_result.iter_mut() {
				let mut value = Meta(json_syntax::Value::Null, mapped_value.metadata().clone());
				std::mem::swap(&mut value, &mut *mapped_value);

				// If the term definition for property in the active context indicates that
				// property is a reverse property
				if let Some(term_definition) = active_context.get(property.as_str()) {
					if term_definition.reverse_property {
						// Initialize as array to true if the container mapping for property in
						// the active context includes @set, otherwise the negation of compactArrays.
						let as_array = term_definition.container.contains(ContainerKind::Set)
							|| !options.compact_arrays;

						// Use add value to add value to the property entry in result using as array.
						add_value(
							&mut result,
							property.borrow().map(AsRef::as_ref),
							value,
							as_array,
						);
						continue;
					}
				}

				reverse_map.insert(property.clone().cast(), value);
			}

			if !reverse_map.is_empty() {
				// Initialize alias by IRI compacting @reverse.
				let alias = compact_iri(
					vocabulary,
					active_context.as_ref(),
					Meta(
						&Term::Keyword(Keyword::Reverse),
						&reverse_properties.key_metadata,
					),
					true,
					false,
					options,
				)
				.map_err(Meta::cast)?;

				// Set the value of the alias entry of result to compacted value.
				result.insert(
					alias.unwrap().cast(),
					Meta(reverse_map.into(), reverse_properties.metadata().clone()),
				);
			}
		}
	}

	// If expanded property is @index and active property has a container mapping in
	// active context that includes @index,
	if let Some(index_entry) = index {
		let mut index_container = false;
		if let Some(active_property) = active_property {
			if let Some(active_property_definition) = active_context.get(*active_property) {
				if active_property_definition
					.container
					.contains(ContainerKind::Index)
				{
					// then the compacted result will be inside of an @index container,
					// drop the @index entry by continuing to the next expanded property.
					index_container = true;
				}
			}
		}

		if !index_container {
			// Initialize alias by IRI compacting expanded property.
			let alias = compact_iri(
				vocabulary,
				active_context.as_ref(),
				Meta(&Term::Keyword(Keyword::Index), &index_entry.key_metadata),
				true,
				false,
				options,
			)
			.map_err(Meta::cast)?;

			// Add an entry alias to result whose value is set to expanded value and continue with the next expanded property.
			result.insert(alias.unwrap().cast(), index_entry.value.clone().cast());
		}
	}

	if let Some(graph_entry) = node.graph_entry() {
		compact_property(
			vocabulary,
			&mut result,
			Meta(
				Term::Keyword(Keyword::Graph),
				graph_entry.key_metadata.clone(),
			),
			graph_entry.value.iter().map(Stripped::as_ref),
			active_context.as_ref(),
			loader,
			false,
			options,
		)
		.await?
	}

	for (expanded_property, expanded_value) in expanded_entries {
		compact_property(
			vocabulary,
			&mut result,
			expanded_property.cloned().cast(),
			expanded_value.iter().map(Stripped::as_ref),
			active_context.as_ref(),
			loader,
			false,
			options,
		)
		.await?
	}

	if let Some(included_entry) = node.included_entry() {
		compact_property(
			vocabulary,
			&mut result,
			Meta(
				Term::Keyword(Keyword::Included),
				included_entry.key_metadata.clone(),
			),
			included_entry.value.iter().map(Stripped::as_ref),
			active_context.as_ref(),
			loader,
			false,
			options,
		)
		.await?
	}

	Ok(Meta(result.into(), meta.clone()))
}

/// Compact the given list of types into the given `result` compacted object.
fn compact_types<I, B, M, C, N, E>(
	vocabulary: &mut N,
	result: &mut json_syntax::Object<M>,
	type_entry: Option<&TypeEntry<I, B, M>>,
	active_context: &Context<I, B, C, M>,
	type_scoped_context: &Context<I, B, C, M>,
	options: Options,
) -> Result<(), MetaError<M, E>>
where
	N: VocabularyMut<I, B>,
	I: Clone + Hash + Eq,
	B: Clone + Hash + Eq,
	M: Clone,
{
	// If expanded property is @type:
	if let Some(type_entry) = type_entry {
		let types = &type_entry.value;
		if !types.is_empty() {
			// If expanded value is a string,
			// then initialize compacted value by IRI compacting expanded value using
			// type-scoped context for active context.
			let compacted_value = if types.len() == 1 {
				optional_string(
					compact_iri(
						vocabulary,
						type_scoped_context,
						types[0].clone().map(Reference::into_term).borrow(),
						true,
						false,
						options,
					)
					.map_err(Meta::cast)?,
					types[0].metadata(),
				)
			} else {
				// Otherwise, expanded value must be a @type array:
				// Initialize compacted value to an empty array.
				let mut compacted_value = Vec::with_capacity(types.len());

				// For each item expanded type in expanded value:
				for ty in types.iter() {
					let ty = ty.clone().map(Reference::into_term);

					// Set term by IRI compacting expanded type using type-scoped context for active context.
					let compacted_ty = compact_iri(
						vocabulary,
						type_scoped_context,
						ty.borrow(),
						true,
						false,
						options,
					)
					.map_err(Meta::cast)?;

					// Append term, to compacted value.
					compacted_value.push(optional_string(compacted_ty, ty.metadata()))
				}

				Meta(
					json_syntax::Value::Array(compacted_value.into_iter().collect()),
					types.metadata().clone(),
				)
			};

			// Initialize alias by IRI compacting expanded property.
			let alias = compact_iri(
				vocabulary,
				active_context,
				Meta(&Term::Keyword(Keyword::Type), &type_entry.key_metadata),
				true,
				false,
				options,
			)
			.map_err(Meta::cast)?
			.unwrap();

			// Initialize as array to true if processing mode is json-ld-1.1 and the
			// container mapping for alias in the active context includes @set,
			// otherwise to the negation of compactArrays.
			let container_mapping = match active_context.get(alias.as_str()) {
				Some(def) => def.container,
				None => Container::None,
			};
			let as_array = (options.processing_mode == ProcessingMode::JsonLd1_1
				&& container_mapping.contains(ContainerKind::Set))
				|| !options.compact_arrays;

			// Use add value to add compacted value to the alias entry in result using as array.
			add_value(
				result,
				alias.borrow().map(String::as_str),
				compacted_value,
				as_array,
			)
		}
	}

	Ok(())
}
