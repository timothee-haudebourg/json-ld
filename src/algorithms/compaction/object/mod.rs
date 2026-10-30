use json_syntax::{JsonObject, JsonValue};
use mown::Mown;

use crate::{
	JsonLdBacktraceBuilder, JsonLdLocatedError, JsonLdTerm,
	algorithms::{
		AsyncJsonLdEnvironment, AsyncProcessingEnvironmentRef, compaction::CompactIndexedFragment,
		context_processing::JsonLdContextProcessingOptions,
	},
	object::{AnyJsonLdObject, JsonLdObjectRef},
	syntax::{JsonLdContainerItem, JsonLdKeyword},
};

use super::Compactor;

mod node;
mod value;

impl Compactor<'_> {
	pub async fn compact_any_indexed_object(
		&self,
		env: &impl AsyncJsonLdEnvironment,
		object: &impl AnyJsonLdObject,
		index: Option<&str>,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		match object.as_ref() {
			JsonLdObjectRef::Value(value) => {
				self.compact_indexed_value_with(env, value, index, location)
					.await
			}
			JsonLdObjectRef::Node(node) => {
				self.compact_indexed_node_with(env, node, index, location)
					.await
			}
			JsonLdObjectRef::List(list) => {
				let mut active_context = self.active_context;
				// If active context has a previous context, the active context is not propagated.
				// If element does not contain an @value entry, and element does not consist of
				// a single @id entry, set active context to previous context from active context,
				// as the scope of a term-scoped context does not apply when processing new node objects.
				if let Some(previous_context) = active_context.previous_context() {
					active_context = previous_context
				}

				// If the term definition for active property in active context has a local context:
				// FIXME https://github.com/w3c/json-ld-api/issues/502
				//       Seems that the term definition should be looked up in `type_scoped_context`.
				let mut active_context = Mown::Borrowed(active_context);
				let mut list_container = false;
				if let Some(active_property) = self.active_property
					&& let Some(active_property_definition) =
						self.type_scoped_context.get(active_property)
				{
					if let Some(local_context) = active_property_definition.context() {
						let def_loc = location.push_backtrace(
							active_property_definition.location().unwrap_or_default(),
						);
						let scoped_context_loc = def_loc.object_value(JsonLdKeyword::Context);

						active_context = Mown::Owned(
							local_context
								.process_with(
									AsyncProcessingEnvironmentRef(env),
									active_property_definition.base_url(),
									active_context.as_ref(),
									JsonLdContextProcessingOptions::from(self.options)
										.with_override(),
									scoped_context_loc,
								)
								.await?
								.into_raw(),
						)
					}

					list_container = active_property_definition
						.container()
						.contains(JsonLdContainerItem::List);
				}

				// `location` is this list object's own position; its items live
				// one level deeper, under its `@list` entry.
				let list_loc = location.object_entry("@list");

				if list_container {
					self.with_active_context(&active_context)
						.with_type_scoped_context(&active_context)
						.compact_collection_with(env, list.iter(), list_loc)
						.await
				} else {
					let mut result = JsonObject::default();
					self.with_active_context(&active_context)
						.compact_property(
							env,
							&mut result,
							JsonLdTerm::Keyword(JsonLdKeyword::List),
							list,
							// active_context.as_ref(),
							// loader,
							false,
							list_loc,
							// options,
						)
						.await?;

					// If expanded property is @index and active property has a container mapping in
					// active context that includes @index,
					if let Some(index) = index {
						let mut index_container = false;
						if let Some(active_property) = self.active_property
							&& let Some(active_property_definition) =
								active_context.get(active_property)
							&& active_property_definition
								.container()
								.contains(JsonLdContainerItem::Index)
						{
							// then the compacted result will be inside of an @index container,
							// drop the @index entry by continuing to the next expanded property.
							index_container = true;
						}

						if !index_container {
							// Initialize alias by IRI compacting expanded property.
							let alias = self.with_active_context(&active_context).compact_key(
								&JsonLdTerm::Keyword(JsonLdKeyword::Index),
								true,
								false,
								location,
							)?;

							// Add an entry alias to result whose value is set to expanded value and continue with the next expanded property.
							result.insert(alias.unwrap(), JsonValue::String(index.into()));
						}
					}

					Ok(JsonValue::Object(result))
				}
			}
		}
	}
}

impl<T: AnyJsonLdObject> CompactIndexedFragment for T {
	async fn compact_indexed_fragment(
		&self,
		env: &impl AsyncJsonLdEnvironment,
		compactor: &Compactor<'_>,
		index: Option<&str>,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		compactor
			.compact_any_indexed_object(env, self, index, location)
			.await
	}
}
