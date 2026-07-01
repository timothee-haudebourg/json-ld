use json_syntax::JsonArray;

use crate::{
	Object,
	algorithms::{
		AsyncProcessingEnvironment, JsonLdLocatedError,
		JsonLdLocationStack,
	},
	context::TermDefinitionRef,
	object::ListObject,
	syntax::ContainerItem,
};

use super::{Expanded, Expander};

impl<'a> Expander<'a> {
	#[allow(clippy::too_many_arguments)]
	pub async fn expand_array(
		&self,
		env: &impl AsyncProcessingEnvironment,
		active_property_definition: Option<TermDefinitionRef<'_>>,
		element: &JsonArray,
		from_map: bool,
		location: JsonLdLocationStack<'_>,
	) -> Result<Expanded, JsonLdLocatedError> {
		// Initialize an empty array, result.
		let mut is_list = false;
		let mut result = Vec::new();

		// If the container mapping of `active_property` includes `@list`, and
		// `expanded_item` is an array, set `expanded_item` to a new map containing
		// the entry `@list` where the value is the original `expanded_item`.
		if let Some(definition) = active_property_definition {
			is_list = definition.container().contains(ContainerItem::List);
		}

		// For each item in element:
		for (i, item) in element.iter().enumerate() {
			// Initialize `expanded_item` to the result of using this algorithm
			// recursively, passing `active_context`, `active_property`, `item` as element,
			// `base_url`, the `frame_expansion`, `ordered`, and `from_map` flags.
			let item_loc = location.array_index(i);
			let e = Box::pin(self.expand_element(env, item, from_map, item_loc)).await?;

			result.extend(e);
		}

		if is_list {
			return Ok(Expanded::Object(
				Object::List(ListObject::new(result)).into(),
			));
		}

		// Return result.
		Ok(Expanded::Array(result))
	}
}
