use iref::Iri;
use json::JsonValue;
use crate::{ContainerType, Id, Value, Object};
use crate::context::{MutableActiveContext, ContextLoader, TermDefinition};

use super::{ExpansionError, expand_element};

pub async fn expand_array<T: Id, C: MutableActiveContext<T>, L: ContextLoader<C::LocalContext>>(active_context: &C, active_property: Option<&str>, active_property_definition: Option<&TermDefinition<T, C>>, element: &[JsonValue], base_url: Option<Iri<'_>>, loader: &mut L, ordered: bool, from_map: bool) -> Result<Vec<Object<T>>, ExpansionError> where C::LocalContext: From<JsonValue> {
	// Initialize an empty array, result.
	let mut result = Vec::new();

	// For each item in element:
	for item in element {
		// Initialize `expanded_item` to the result of using this algorithm
		// recursively, passing `active_context`, `active_property`, `item` as element,
		// `base_url`, the `frame_expansion`, `ordered`, and `from_map` flags.
		let expanded_items: Vec<Object<T>> = expand_element(active_context, active_property, item, base_url, loader, ordered, from_map).await?;

		// If the container mapping of `active_property` includes `@list`, and
		// `expanded_item` is an array, set `expanded_item` to a new map containing
		// the entry `@list` where the value is the original `expanded_item`.
		let is_list = if let Some(definition) = active_property_definition {
			definition.container.contains(ContainerType::List) && expanded_items.len() > 1
		} else {
			false
		};

		if is_list {
			result.push(Value::List(expanded_items).into());
		} else {
			// If `expanded_item` is an array, append each of its items to result.
			for item in expanded_items {
				result.push(item)
			}
		}
	}

	// Return result.
	return Ok(result)
}
