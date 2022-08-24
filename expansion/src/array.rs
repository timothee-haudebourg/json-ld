use crate::{expand_element, ActiveProperty, Error, Expanded, Loader, Options, WarningHandler};
use json_ld_context_processing::{ContextLoader, NamespaceMut, Process};
use json_ld_core::{context::TermDefinition, object, Context, Object};
use json_ld_syntax::{Array, ContainerKind, Value};
use locspan::Meta;
use std::hash::Hash;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn expand_array<
	T,
	B,
	N,
	C: Process<T, B>,
	L: Loader<T> + ContextLoader<T>,
	W: Send + WarningHandler<B, N, C::Metadata>,
>(
	namespace: &mut N,
	active_context: &Context<T, B, C>,
	active_property: ActiveProperty<'_, C::Metadata>,
	active_property_definition: Option<&TermDefinition<T, B, C>>,
	Meta(element, meta): Meta<&Array<C, C::Metadata>, &C::Metadata>,
	base_url: Option<&T>,
	loader: &mut L,
	options: Options,
	from_map: bool,
	mut warnings: W,
) -> Result<(Expanded<T, B, C::Metadata>, W), Meta<Error<L::ContextError>, C::Metadata>>
where
	N: Send + Sync + NamespaceMut<T, B>,
	T: Clone + Eq + Hash + Sync + Send,
	B: Clone + Eq + Hash + Sync + Send,
	C: Sync + Send,
	L: Sync + Send,
	<L as Loader<T>>::Output: Into<Value<C, C::Metadata>>,
	<L as ContextLoader<T>>::Output: Into<C>,
{
	// Initialize an empty array, result.
	let mut is_list = false;
	let mut result = Vec::new();

	// If the container mapping of `active_property` includes `@list`, and
	// `expanded_item` is an array, set `expanded_item` to a new map containing
	// the entry `@list` where the value is the original `expanded_item`.
	if let Some(definition) = active_property_definition {
		is_list = definition.container.contains(ContainerKind::List);
	}

	// For each item in element:
	for item in element.iter() {
		// Initialize `expanded_item` to the result of using this algorithm
		// recursively, passing `active_context`, `active_property`, `item` as element,
		// `base_url`, the `frame_expansion`, `ordered`, and `from_map` flags.
		let (e, w) = expand_element(
			namespace,
			active_context,
			active_property,
			item,
			base_url,
			loader,
			options,
			from_map,
			warnings,
		)
		.await?;
		warnings = w;

		result.extend(e);
	}

	if is_list {
		return Ok((
			Expanded::Object(Meta(
				Object::List(object::List::new(meta.clone(), Meta(result, meta.clone()))).into(),
				meta.clone(),
			)),
			warnings,
		));
	}

	// Return result.
	Ok((Expanded::Array(result), warnings))
}
