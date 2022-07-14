use super::expand_element;
use crate::{ActiveProperty, Error, Loader, Options, Warning};
use iref::IriBuf;
use json_ld_context_processing::{ContextLoader, Process};
use json_ld_core::{Context, ExpandedDocument, Id, Indexed, Object};
use json_ld_syntax::Value;
use locspan::{Meta, Stripped};
use std::collections::HashSet;

/// Expand the given JSON-LD document.
///
/// Note that you probably do not want to use this function directly,
/// but instead use the [`Document::expand`](crate::Document::expand) method on
/// a `Value` instance.
pub(crate) async fn expand<'a, T: Id, C: Process<T>, L: Loader + ContextLoader, W>(
	active_context: &'a Context<T, C>,
	document: &'a Meta<Value<C, C::Metadata>, C::Metadata>,
	base_url: Option<IriBuf>,
	loader: &'a mut L,
	options: Options,
	warnings: W,
) -> Result<ExpandedDocument<T, C::Metadata>, Meta<Error<L>, C::Metadata>>
where
	T: Send + Sync,
	C: Send + Sync,
	L: Send + Sync,
	<L as Loader>::Output: Into<Value<C, C::Metadata>>,
	<L as ContextLoader>::Output: Into<C>,
	W: 'a + Send + FnMut(Meta<Warning, C::Metadata>),
{
	let base_url = base_url.as_ref().map(|url| url.as_iri());
	let (expanded, _) = expand_element(
		active_context,
		ActiveProperty::None,
		document,
		base_url,
		loader,
		options,
		false,
		warnings,
	)
	.await?;
	if expanded.len() == 1 {
		match expanded.into_iter().next().unwrap().into_unnamed_graph() {
			Ok(graph) => Ok(ExpandedDocument::new(graph)),
			Err(obj) => {
				let mut set = HashSet::new();
				if filter_top_level_item(&obj) {
					set.insert(Stripped(obj));
				}
				Ok(ExpandedDocument::new(set))
			}
		}
	} else {
		Ok(ExpandedDocument::new(
			expanded
				.into_iter()
				.filter(filter_top_level_item)
				.map(Stripped)
				.collect(),
		))
	}
}

pub(crate) fn filter_top_level_item<T: Id, M>(item: &Indexed<Object<T, M>>) -> bool {
	// Remove dangling values.
	!matches!(item.inner(), Object::Value(_))
}
