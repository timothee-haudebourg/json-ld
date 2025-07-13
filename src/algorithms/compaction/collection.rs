use indexmap::IndexSet;

use crate::{
	algorithms::{compaction::Compactor, ProcessingEnvironment},
	syntax::ContainerItem,
	Error,
};

use super::CompactFragment;

impl Compactor<'_> {
	pub async fn compact_collection_with<'a, O, T>(
		&self,
		env: &mut impl ProcessingEnvironment,
		items: O,
	) -> Result<json_syntax::Value, Error>
	where
		T: 'a + CompactFragment,
		O: 'a + Iterator<Item = &'a T>,
	{
		let mut result = Vec::new();

		for item in items {
			let compacted_item = Box::pin(item.compact_fragment(env, self)).await?;

			if !compacted_item.is_null() {
				result.push(compacted_item)
			}
		}

		let mut list_or_set = false;
		if let Some(active_property) = self.active_property {
			if let Some(active_property_definition) = self.active_context.get(active_property) {
				list_or_set = active_property_definition
					.container()
					.contains(ContainerItem::List)
					|| active_property_definition
						.container()
						.contains(ContainerItem::Set);
			}
		}

		if result.is_empty()
			|| result.len() > 1
			|| !self.options.compact_arrays
			|| self.active_property == Some("@graph")
			|| self.active_property == Some("@set")
			|| list_or_set
		{
			return Ok(json_syntax::Value::Array(result.into_iter().collect()));
		}

		Ok(result.into_iter().next().unwrap())
	}
}

impl<T: CompactFragment> CompactFragment for Vec<T> {
	async fn compact_fragment(
		&self,
		env: &mut impl ProcessingEnvironment,
		compactor: &Compactor<'_>,
	) -> Result<json_syntax::Value, Error> {
		compactor.compact_collection_with(env, self.iter()).await
	}
}

impl<T: CompactFragment> CompactFragment for [T] {
	async fn compact_fragment(
		&self,
		env: &mut impl ProcessingEnvironment,
		compactor: &Compactor<'_>,
	) -> Result<json_syntax::Value, Error> {
		compactor.compact_collection_with(env, self.iter()).await
	}
}

impl<T: CompactFragment> CompactFragment for IndexSet<T> {
	async fn compact_fragment(
		&self,
		env: &mut impl ProcessingEnvironment,
		compactor: &Compactor<'_>,
	) -> Result<json_syntax::Value, Error> {
		compactor.compact_collection_with(env, self.iter()).await
	}
}
