use std::collections::HashSet;
use json_ld_core::{
	Id,
	Context,
	Indexed,
	Object,
	ExpandedDocument
};
use json_ld_context_processing::{Process, ContextLoader};
use json_ld_syntax::Value;
use iref::IriBuf;
use locspan::{Meta, Stripped};
use crate::{
	Options,
	Loader,
	Warning,
	Error,
	ActiveProperty
};
use super::expand_element;

/// Expand the given JSON-LD document.
///
/// Note that you probably do not want to use this function directly,
/// but instead use the [`Document::expand`](crate::Document::expand) method on
/// a `Value` instance.
pub(crate) async fn expand<'a, T: Id, C: Process<T>, L: Loader + ContextLoader>(
	active_context: &'a Context<T, C>,
	document: &'a Meta<Value<C, C::Metadata>, C::Metadata>,
	base_url: Option<IriBuf>,
	loader: &'a mut L,
	options: Options,
	warnings: impl 'a + Send + FnMut(Meta<Warning, C::Metadata>),
) -> Result<ExpandedDocument<T, C::Metadata>, Meta<Error, C::Metadata>>
where
	T: Send + Sync,
	C: Send + Sync,
	L: Send + Sync,
	<L as Loader>::Output: Into<Value<C, C::Metadata>>,
	<L as ContextLoader>::Output: Into<C>
{
	let base_url = base_url.as_ref().map(|url| url.as_iri());
	let expanded = expand_element(
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
		Ok(ExpandedDocument::new(expanded.into_iter().filter(filter_top_level_item).map(Stripped).collect()))
	}
}

pub(crate) fn filter_top_level_item<T: Id, M>(item: &Indexed<Object<T, M>>) -> bool {
	// Remove dangling values.
	!matches!(item.inner(), Object::Value(_))
}