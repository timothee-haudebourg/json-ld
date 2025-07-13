use mown::Mown;

use crate::{
	algorithms::{
		compaction::CompactIndexedFragment, context_processing::ContextProcessingOptions,
		ProcessingEnvironment, ProcessingEnvironmentRefMut,
	},
	object::{AnyObject, Ref},
	syntax::{ContainerItem, Keyword},
	Error, Term,
};

use super::Compactor;

mod node;
mod value;

impl Compactor<'_> {
	pub async fn compact_any_indexed_object(
		&self,
		env: &mut impl ProcessingEnvironment,
		object: &impl AnyObject,
		index: Option<&str>,
	) -> Result<json_syntax::Value, Error> {
		match object.as_ref() {
			Ref::Value(value) => self.compact_indexed_value_with(env, value, index).await,
			Ref::Node(node) => self.compact_indexed_node_with(env, node, index).await,
			Ref::List(list) => {
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
				if let Some(active_property) = self.active_property {
					if let Some(active_property_definition) =
						self.type_scoped_context.get(active_property)
					{
						if let Some(local_context) = active_property_definition.context() {
							active_context = Mown::Owned(
								local_context
									.process_with(
										ProcessingEnvironmentRefMut(env),
										// vocabulary,
										active_property_definition.base_url(),
										active_context.as_ref(),
										ContextProcessingOptions::from(self.options)
											.with_override(),
									)
									.await?,
							)
						}

						list_container = active_property_definition
							.container()
							.contains(ContainerItem::List);
					}
				}

				if list_container {
					self.with_active_context(&active_context)
						.with_type_scoped_context(&active_context)
						.compact_collection_with(env, list.iter())
						.await
				} else {
					let mut result = json_syntax::Object::default();
					self.with_active_context(&active_context)
						.compact_property(
							env,
							&mut result,
							Term::Keyword(Keyword::List),
							list,
							// active_context.as_ref(),
							// loader,
							false,
							// options,
						)
						.await?;

					// If expanded property is @index and active property has a container mapping in
					// active context that includes @index,
					if let Some(index) = index {
						let mut index_container = false;
						if let Some(active_property) = self.active_property {
							if let Some(active_property_definition) =
								active_context.get(active_property)
							{
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
							let alias = self.with_active_context(&active_context).compact_key(
								&Term::Keyword(Keyword::Index),
								true,
								false,
							)?;

							// Add an entry alias to result whose value is set to expanded value and continue with the next expanded property.
							result.insert(alias.unwrap(), json_syntax::Value::String(index.into()));
						}
					}

					Ok(json_syntax::Value::Object(result))
				}
			}
		}
	}
}

impl<T: AnyObject> CompactIndexedFragment for T {
	async fn compact_indexed_fragment(
		&self,
		env: &mut impl ProcessingEnvironment,
		compactor: &Compactor<'_>,
		index: Option<&str>,
	) -> Result<json_syntax::Value, Error> {
		compactor.compact_any_indexed_object(env, self, index).await
	}
}
