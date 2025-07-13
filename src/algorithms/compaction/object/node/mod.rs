use mown::Mown;

use crate::{
	algorithms::{
		compaction::{object::value::add_value, Compactor},
		context_processing::ContextProcessingOptions,
		ProcessingEnvironment, ProcessingEnvironmentRefMut,
	},
	syntax::{Container, ContainerItem, Keyword},
	Error, Id, Node, ProcessingMode, Term, Type,
};

mod property;

fn optional_string(s: Option<String>) -> json_syntax::Value {
	s.map(Into::into)
		.unwrap_or_else(|| json_syntax::Value::Null)
}

impl Compactor<'_> {
	/// Compact the given indexed node.
	#[allow(clippy::too_many_arguments)]
	pub async fn compact_indexed_node_with(
		&self,
		env: &mut impl ProcessingEnvironment,
		node: &Node,
		index: Option<&str>,
		// type_scoped_context: &ProcessedContext,
		// active_property: Option<&str>,
	) -> Result<json_syntax::Value, Error> {
		// If active context has a previous context, the active context is not propagated.
		// If element does not contain an @value entry, and element does not consist of
		// a single @id entry, set active context to previous context from active context,
		// as the scope of a term-scoped context does not apply when processing new node objects.
		let mut active_context = Mown::Borrowed(self.active_context);
		if !(node.is_empty() && node.id.is_some()) {
			// does not consist of a single @id entry
			if let Some(previous_context) = self.active_context.previous_context() {
				active_context = Mown::Borrowed(previous_context);
			}
		}

		// If the term definition for active property in active context has a local context:
		// FIXME https://github.com/w3c/json-ld-api/issues/502
		//       Seems that the term definition should be looked up in `type_scoped_context`.
		if let Some(active_property) = self.active_property {
			if let Some(active_property_definition) = self.type_scoped_context.get(active_property)
			{
				if let Some(local_context) = active_property_definition.context() {
					active_context = Mown::Owned(
						local_context
							.process_with(
								ProcessingEnvironmentRefMut(env),
								active_property_definition.base_url(),
								active_context.as_ref(),
								ContextProcessingOptions::from(self.options).with_override(),
							)
							.await?,
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
				let compacted_ty = self
					.with_active_context(self.type_scoped_context)
					.compact_iri(&ty.clone().into_term(), true, false)?;
				compacted_types.push(compacted_ty)
			}

			compacted_types.sort_by(|a, b| a.as_ref().unwrap().cmp(b.as_ref().unwrap()));

			for term in &compacted_types {
				if let Some(term_definition) = self
					.type_scoped_context
					.get(term.as_ref().unwrap().as_str())
				{
					if let Some(local_context) = term_definition.context() {
						let processing_options =
							ContextProcessingOptions::from(self.options).without_propagation();
						active_context = Mown::Owned(
							local_context
								.process_with(
									ProcessingEnvironmentRefMut(env),
									term_definition.base_url(),
									active_context.as_ref(),
									processing_options,
								)
								.await?,
						)
					}
				}
			}
		}

		// For each key expanded property and value expanded value in element, ordered
		// lexicographically by expanded property if ordered is true:
		let mut expanded_entries: Vec<_> = node.properties().iter().collect();
		if self.options.ordered {
			expanded_entries.sort_by(|(a, _), (b, _)| a.as_str().cmp(b.as_str()))
		}

		// If expanded property is @id:
		if let Some(id_entry) = &node.id {
			let id = id_entry.clone().into_term();

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
				let type_mapping = match self.active_property {
					Some(prop) => match active_context.get(prop) {
						Some(def) => def.typ(),
						None => None,
					},
					None => None,
				};

				if type_mapping == Some(&Type::Id) {
					let compacted_value = self
						.with_active_context(&active_context)
						.compact_iri(&id, false, false)?;
					return Ok(optional_string(compacted_value));
				}

				// Otherwise, if the type mapping of active property is set to @vocab,
				// set result to the result of IRI compacting the value associated with the @id entry.
				if type_mapping == Some(&Type::Vocab) {
					let compacted_value = self
						.with_active_context(&active_context)
						.compact_iri(&id, true, false)?;
					return Ok(optional_string(compacted_value));
				}
			}

			// If expanded value is a string, then initialize compacted value by IRI
			// compacting expanded value with vocab set to false.
			let compacted_value = self
				.with_active_context(&active_context)
				.compact_iri(&id, false, false)?;

			// Initialize alias by IRI compacting expanded property.
			let alias = self.with_active_context(&active_context).compact_iri(
				&Term::Keyword(Keyword::Id),
				true,
				false,
			)?;

			// Add an entry alias to result whose value is set to compacted value and continue
			// to the next expanded property.
			if let Some(key) = alias {
				result.insert(key, optional_string(compacted_value));
			}
		}

		self.with_active_context(&active_context)
			.compact_types(&mut result, node.types.as_deref())?;

		// If expanded property is @reverse:
		if let Some(reverse_properties) = node.reverse_properties_entry() {
			if !reverse_properties.is_empty() {
				// Initialize compacted value to the result of using this algorithm recursively,
				// passing active context, @reverse for active property,
				// expanded value for element, and the compactArrays and ordered flags.
				let active_property = "@reverse";
				if let Some(active_property_definition) = active_context.get(active_property) {
					if let Some(local_context) = active_property_definition.context() {
						active_context = Mown::Owned(
							local_context
								.process_with(
									ProcessingEnvironmentRefMut(env),
									active_property_definition.base_url(),
									active_context.as_ref(),
									ContextProcessingOptions::from(self.options).with_override(),
								)
								.await?,
						)
					}
				}

				let mut reverse_result = json_syntax::Object::default();
				for (expanded_property, expanded_value) in reverse_properties.iter() {
					self.with_active_context(&active_context)
						.compact_property(
							env,
							&mut reverse_result,
							expanded_property.clone().into(),
							expanded_value.iter(),
							true,
						)
						.await?;
				}

				// For each property and value in compacted value:
				let mut reverse_map = json_syntax::Object::default();
				for (property, mapped_value) in reverse_result.iter_mut() {
					let mut value = json_syntax::Value::Null;
					std::mem::swap(&mut value, &mut *mapped_value);

					// If the term definition for property in the active context indicates that
					// property is a reverse property
					if let Some(term_definition) = active_context.get(property.as_str()) {
						if term_definition.reverse_property() {
							// Initialize as array to true if the container mapping for property in
							// the active context includes @set, otherwise the negation of compactArrays.
							let as_array = term_definition.container().contains(ContainerItem::Set)
								|| !self.options.compact_arrays;

							// Use add value to add value to the property entry in result using as array.
							add_value(&mut result, property, value, as_array);
							continue;
						}
					}

					reverse_map.insert(property.clone(), value);
				}

				if !reverse_map.is_empty() {
					// Initialize alias by IRI compacting @reverse.
					let alias = self.with_active_context(&active_context).compact_iri(
						&Term::Keyword(Keyword::Reverse),
						true,
						false,
					)?;

					// Set the value of the alias entry of result to compacted value.
					result.insert(alias.unwrap(), reverse_map.into());
				}
			}
		}

		// If expanded property is @index and active property has a container mapping in
		// active context that includes @index,
		if let Some(index_entry) = index {
			let mut index_container = false;
			if let Some(active_property) = self.active_property {
				if let Some(active_property_definition) = active_context.get(active_property) {
					if active_property_definition
						.container()
						.contains(ContainerItem::Index)
					{
						// then the compacted result will be inside of an @index container,
						// drop the @index entry by continuing to the next expanded property.
						index_container = true;
					}
				}
			}

			if !index_container {
				// Initialize alias by IRI compacting expanded property.
				let alias = self.with_active_context(&active_context).compact_iri(
					&Term::Keyword(Keyword::Index),
					true,
					false,
				)?;

				// Add an entry alias to result whose value is set to expanded value and continue with the next expanded property.
				result.insert(alias.unwrap(), index_entry.into());
			}
		}

		if let Some(graph_entry) = node.graph_entry() {
			self.with_active_context(&active_context)
				.compact_property(
					env,
					&mut result,
					Term::Keyword(Keyword::Graph),
					graph_entry.iter(),
					false,
				)
				.await?
		}

		for (expanded_property, expanded_value) in expanded_entries {
			self.with_active_context(&active_context)
				.compact_property(
					env,
					&mut result,
					expanded_property.clone().into(),
					expanded_value.iter(),
					false,
				)
				.await?
		}

		if let Some(included_entry) = node.included_entry() {
			self.with_active_context(&active_context)
				.compact_property(
					env,
					&mut result,
					Term::Keyword(Keyword::Included),
					included_entry.iter(),
					false,
				)
				.await?
		}

		Ok(result.into())
	}

	/// Compact the given list of types into the given `result` compacted object.
	fn compact_types(
		self,
		result: &mut json_syntax::Object,
		types: Option<&[Id]>,
	) -> Result<(), Error> {
		// If expanded property is @type:
		if let Some(types) = types {
			if !types.is_empty() {
				// If expanded value is a string,
				// then initialize compacted value by IRI compacting expanded value using
				// type-scoped context for active context.
				let compacted_value = if types.len() == 1 {
					optional_string(
						self.with_active_context(self.type_scoped_context)
							.compact_iri(&types[0].clone().into_term(), true, false)?,
					)
				} else {
					// Otherwise, expanded value must be a @type array:
					// Initialize compacted value to an empty array.
					let mut compacted_value = Vec::with_capacity(types.len());

					// For each item expanded type in expanded value:
					for ty in types.iter() {
						let ty = ty.clone().into_term();

						// Set term by IRI compacting expanded type using type-scoped context for active context.
						let compacted_ty = self
							.with_active_context(&self.type_scoped_context)
							.compact_iri(&ty, true, false)?;

						// Append term, to compacted value.
						compacted_value.push(optional_string(compacted_ty))
					}

					json_syntax::Value::Array(compacted_value.into_iter().collect())
				};

				// Initialize alias by IRI compacting expanded property.
				let alias = self
					.compact_iri(&Term::Keyword(Keyword::Type), true, false)?
					.unwrap();

				// Initialize as array to true if processing mode is json-ld-1.1 and the
				// container mapping for alias in the active context includes @set,
				// otherwise to the negation of compactArrays.
				let container_mapping = match self.active_context.get(alias.as_str()) {
					Some(def) => def.container(),
					None => Container::Null,
				};
				let as_array = (self.options.processing_mode == ProcessingMode::JsonLd1_1
					&& container_mapping.contains(ContainerItem::Set))
					|| !self.options.compact_arrays;

				// Use add value to add compacted value to the alias entry in result using as array.
				add_value(result, &alias, compacted_value, as_array)
			}
		}

		Ok(())
	}
}
