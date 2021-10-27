use super::{expand_element, Expanded, JsonExpand, Options};
use crate::{
	context::{Loader, TermDefinition},
	object::*,
	syntax::ContainerType,
	ContextMut, Error, Id,
};
use cc_traits::Iter;
use iref::Iri;

pub async fn expand_array<
	J: JsonExpand,
	T: Id + Send + Sync,
	C: ContextMut<T> + Send + Sync,
	L: Loader + Send + Sync,
>(
	active_context: &C,
	active_property: Option<&str>,
	active_property_definition: Option<&TermDefinition<T, C>>,
	element: &J::Array,
	base_url: Option<Iri<'_>>,
	loader: &mut L,
	options: Options,
	from_map: bool,
) -> Result<Expanded<J, T>, Error>
where
	C::LocalContext: From<L::Output> + From<J>,
	L::Output: Into<J>,
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
	for item in element.iter() {
		// Initialize `expanded_item` to the result of using this algorithm
		// recursively, passing `active_context`, `active_property`, `item` as element,
		// `base_url`, the `frame_expansion`, `ordered`, and `from_map` flags.
		result.extend(
			expand_element(
				active_context,
				active_property,
				&*item,
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
