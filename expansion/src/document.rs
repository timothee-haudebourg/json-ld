use super::expand_element;
use crate::{ActiveProperty, Error, Loader, Options, WarningHandler};
use json_ld_context_processing::{ContextLoader, NamespaceMut, Process};
use json_ld_core::{Context, ExpandedDocument, Indexed, Object};
use json_ld_syntax::Value;
use locspan::Meta;
use std::hash::Hash;

/// Expand the given JSON-LD document.
///
/// Note that you probably do not want to use this function directly,
/// but instead use the [`Document::expand`](crate::Document::expand) method on
/// a `Value` instance.
pub(crate) async fn expand<'a, T, B, N, C: Process<T, B>, L: Loader<T> + ContextLoader<T>, W>(
	namespace: &'a mut N,
	document: &'a Meta<Value<C, C::Metadata>, C::Metadata>,
	active_context: Context<T, B, C>,
	base_url: Option<&'a T>,
	loader: &'a mut L,
	options: Options,
	warnings: W,
) -> Result<ExpandedDocument<T, B, C::Metadata>, Meta<Error<L::ContextError>, C::Metadata>>
where
	N: Send + Sync + NamespaceMut<T, B>,
	T: Clone + Eq + Hash + Send + Sync,
	B: Clone + Eq + Hash + Send + Sync,
	C: Send + Sync,
	L: Send + Sync,
	<L as Loader<T>>::Output: Into<Value<C, C::Metadata>>,
	<L as ContextLoader<T>>::Output: Into<C>,
	W: 'a + Send + WarningHandler<B, N, C::Metadata>,
{
	let (expanded, _) = expand_element(
		namespace,
		&active_context,
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
		let Meta(obj, meta) = expanded.into_iter().next().unwrap();
		match obj.into_unnamed_graph() {
			Ok(Meta(graph, _)) => Ok(ExpandedDocument::from(graph)),
			Err(obj) => {
				let obj = Meta(obj, meta);
				let mut result = ExpandedDocument::new();
				if filter_top_level_item(&obj) {
					result.insert(obj);
				}
				Ok(result)
			}
		}
	} else {
		Ok(expanded.into_iter().filter(filter_top_level_item).collect())
	}
}

pub(crate) fn filter_top_level_item<T, B, M>(
	Meta(item, _): &Meta<Indexed<Object<T, B, M>>, M>,
) -> bool {
	// Remove dangling values.
	!matches!(item.inner(), Object::Value(_))
}
