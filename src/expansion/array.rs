use super::{expand_element, Expanded, Options};
use crate::{
	context::{Loader, TermDefinition},
	object::*,
	syntax::ContainerType,
	ContextMut, Error, Id,
};
use iref::Iri;
use json::JsonValue;

pub async fn expand_array<
	T: Send + Sync + Id,
	C: Send + Sync + ContextMut<T>,
	L: Send + Sync + Loader,
>(
	active_context: &C,
	active_property: Option<&str>,
	active_property_definition: Option<&TermDefinition<T, C>>,
	element: &[JsonValue],
	base_url: Option<Iri<'_>>,
	loader: &mut L,
	options: Options,
	from_map: bool,
) -> Result<Expanded<T>, Error>
where
	C::LocalContext: Send + Sync + From<L::Output> + From<JsonValue>,
	L::Output: Into<JsonValue>,
{
	// Initialize an empty array, result.
	let mut is_list = false;
	let mut result = Vec::new();

	// If the container mapping of `active_property` includes `@list`, and
	// `expanded_item` is an array, set `expanded_item` to a new map containing
	// the entry `@list` where the value is the original `expanded_item`.
	if let Some(definition) = active_property_definition {
		is_list = definition.container.contains(ContainerType::List);
	}

	// For each item in element:
	for item in element {
		// Initialize `expanded_item` to the result of using this algorithm
		// recursively, passing `active_context`, `active_property`, `item` as element,
		// `base_url`, the `frame_expansion`, `ordered`, and `from_map` flags.
		result.extend(
			expand_element(
				active_context,
				active_property,
				item,
				base_url,
				loader,
				options,
				from_map,
			)
			.await?,
		);
	}

	if is_list {
		return Ok(Expanded::Object(Object::List(result).into()));
	}

	// Return result.
	Ok(Expanded::Array(result))
}
