use std::collections::HashSet;
use json_ld_core::{
	Id,
	Context,
	Indexed,
	Object
};
use json_syntax::Value;
use iref::IriBuf;
use locspan::Loc;
use crate::{
	Options,
	Loader,
	Warning,
	Error
};
use super::expand_element;

/// Expand the given JSON-LD document.
///
/// Note that you probably do not want to use this function directly,
/// but instead use the [`Document::expand`](crate::Document::expand) method on
/// a `Value` instance.
pub async fn expand<'a, T: Id, S, P, C, L: Loader>(
	active_context: &'a Context<T, C>,
	document: &'a Value<S, P>,
	base_url: Option<IriBuf>,
	loader: &'a mut L,
	options: Options,
	warnings: &mut Vec<Loc<Warning, S, P>>,
) -> Result<HashSet<Indexed<Object<T>>>, Loc<Error, S, P>>
where
	T: Send + Sync,
	C: Send + Sync,
	L: Send + Sync,
	L::Output: Into<Value<S, P>>,
{
	let base_url = base_url.as_ref().map(|url| url.as_iri());
	let expanded = expand_element(
		active_context,
		None,
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
			Ok(graph) => Ok(graph),
			Err(obj) => {
				let mut set = HashSet::new();
				if filter_top_level_item(&obj) {
					set.insert(obj);
				}
				Ok(set)
			}
		}
	} else {
		Ok(expanded.into_iter().filter(filter_top_level_item).collect())
	}
}

fn filter_top_level_item<T: Id>(item: &Indexed<Object<T>>) -> bool {
	// Remove dangling values.
	!matches!(item.inner(), Object::Value(_))
}