use crate::{expand_element, ActiveProperty, Error, Expanded, Loader, Options, WarningHandler};
use json_ld_context_processing::{ContextLoader, ProcessMeta};
use json_ld_core::{context::TermDefinitionRef, object, Context, Object};
use json_ld_syntax::ContainerKind;
use json_syntax::{Array, Value};
use locspan::Meta;
use rdf_types::VocabularyMut;
use std::hash::Hash;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn expand_array<
	T,
	B,
	M,
	N,
	L: Loader<T, M> + ContextLoader<T, M>,
	W: Send + WarningHandler<B, N, M>,
>(
	vocabulary: &mut N,
	active_context: &Context<T, B, M>,
	active_property: ActiveProperty<'_, M>,
	active_property_definition: Option<TermDefinitionRef<'_, T, B, M>>,
	Meta(element, meta): Meta<&Array<M>, &M>,
	base_url: Option<&T>,
	loader: &mut L,
	options: Options,
	from_map: bool,
	mut warnings: W,
) -> Result<(Expanded<T, B, M>, W), Meta<Error<M, L::ContextError>, M>>
where
	N: Send + Sync + VocabularyMut<Iri = T, BlankId = B>,
	T: Clone + Eq + Hash + Sync + Send,
	B: Clone + Eq + Hash + Sync + Send,
	M: Clone + Sync + Send,
	L: Sync + Send,
	L::Output: Into<Value<M>>,
	L::ContextError: Send,
{
	// Initialize an empty array, result.
	let mut is_list = false;
	let mut result = Vec::new();

	// If the container mapping of `active_property` includes `@list`, and
	// `expanded_item` is an array, set `expanded_item` to a new map containing
	// the entry `@list` where the value is the original `expanded_item`.
	if let Some(definition) = active_property_definition {
		is_list = definition.container().contains(ContainerKind::List);
	}

	// For each item in element:
	for item in element.iter() {
		// Initialize `expanded_item` to the result of using this algorithm
		// recursively, passing `active_context`, `active_property`, `item` as element,
		// `base_url`, the `frame_expansion`, `ordered`, and `from_map` flags.
		let (e, w) = expand_element(
			vocabulary,
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
				Object::List(object::List::new_with(
					meta.clone(),
					Meta(result, meta.clone()),
				))
				.into(),
				meta.clone(),
			)),
			warnings,
		));
	}

	// Return result.
	Ok((Expanded::Array(result), warnings))
}
