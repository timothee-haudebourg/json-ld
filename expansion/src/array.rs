use iref::Iri;
use locspan::Meta;
use json_ld_core::{Id, Context, context::TermDefinition, Object};
use json_ld_context_processing::{Process, Loader as ContextLoader};
use json_ld_syntax::{ContainerType, Value, Array};
use crate::{Loader, Options, Warning, Error, Expanded, expand_element};

pub async fn expand_array<
	T: Id,
	C: Process<T>,
	L: Loader + ContextLoader
>(
	active_context: &Context<T, C>,
	active_property: Option<Meta<&str, C::Metadata>>,
	active_property_definition: Option<&TermDefinition<T, C>>,
	element: &Array<C, C::Metadata>,
	base_url: Option<Iri<'_>>,
	loader: &mut L,
	options: Options,
	from_map: bool,
	mut warnings: impl Send + FnMut(Meta<Warning, C::Metadata>),
) -> Result<Expanded<T>, Meta<Error, C::Metadata>>
where
	T: Sync + Send,
	C: Sync + Send,
	L: Sync + Send,
	<L as Loader>::Output: Into<Value<C, C::Metadata>>,
	<L as ContextLoader>::Output: Into<C>
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
				active_property.clone(),
				&*item,
				base_url,
				loader,
				options,
				from_map,
				&mut warnings,
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
