use crate::{expand_element, ActiveProperty, Error, Expanded, Loader, Options, WarningHandler};
use json_ld_core::{context::TermDefinitionRef, object, Context, Environment, Object};
use json_ld_syntax::ContainerKind;
use json_syntax::Array;
use rdf_types::VocabularyMut;
use std::hash::Hash;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn expand_array<N, L, W>(
	env: Environment<'_, N, L, W>,
	active_context: &Context<N::Iri, N::BlankId>,
	active_property: ActiveProperty<'_>,
	active_property_definition: Option<TermDefinitionRef<'_, N::Iri, N::BlankId>>,
	element: &Array,
	base_url: Option<&N::Iri>,
	options: Options,
	from_map: bool,
) -> Result<Expanded<N::Iri, N::BlankId>, Error>
where
	N: VocabularyMut,
	N::Iri: Clone + Eq + Hash,
	N::BlankId: Clone + Eq + Hash,
	L: Loader,
	W: WarningHandler<N>,
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
		let e = Box::pin(expand_element(
			Environment {
				vocabulary: env.vocabulary,
				loader: env.loader,
				warnings: env.warnings,
			},
			active_context,
			active_property,
			item,
			base_url,
			options,
			from_map,
		))
		.await?;

		result.extend(e);
	}

	if is_list {
		return Ok(Expanded::Object(
			Object::List(object::List::new(result)).into(),
		));
	}

	// Return result.
	Ok(Expanded::Array(result))
}
