use json_syntax::Array;

use crate::{
	algorithms::{Error, ProcessingEnvironment},
	context::TermDefinitionRef,
	object::ListObject,
	syntax::ContainerItem,
	Object,
};

use super::{Expanded, Expander};

impl<'a> Expander<'a> {
	#[allow(clippy::too_many_arguments)]
	pub async fn expand_array(
		&self,
		env: &mut impl ProcessingEnvironment,
		active_property_definition: Option<TermDefinitionRef<'_>>,
		element: &Array,
		from_map: bool,
	) -> Result<Expanded, Error> {
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
		for item in element.iter() {
			// Initialize `expanded_item` to the result of using this algorithm
			// recursively, passing `active_context`, `active_property`, `item` as element,
			// `base_url`, the `frame_expansion`, `ordered`, and `from_map` flags.
			let e = Box::pin(self.expand_element(
				env,
				// Environment {
				// 	vocabulary: env.vocabulary,
				// 	loader: env.loader,
				// 	warnings: env.warnings,
				// },
				// active_context,
				// active_property,
				item, // base_url,
				// options,
				from_map,
			))
			.await?;

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
