use super::expand_element;
use crate::{ActiveProperty, Error, Loader, Options, WarningHandler};
use json_ld_context_processing::{ContextLoader, ProcessMeta};
use json_ld_core::{Context, ExpandedDocument, IndexedObject, Object};
use json_syntax::Value;
use locspan::Meta;
use rdf_types::VocabularyMut;
use std::hash::Hash;

/// Expand the given JSON-LD document.
///
/// Note that you probably do not want to use this function directly,
/// but instead use the [`Document::expand`](crate::Document::expand) method on
/// a `Value` instance.
pub(crate) async fn expand<'a, T, B, M, C, N, L: Loader<T, M> + ContextLoader<T, M>, W>(
	vocabulary: &'a mut N,
	document: &'a Meta<Value<M>, M>,
	active_context: Context<T, B, C, M>,
	base_url: Option<&'a T>,
	loader: &'a mut L,
	options: Options,
	warnings: W,
) -> Result<Meta<ExpandedDocument<T, B, M>, M>, Meta<Error<M, L::ContextError>, M>>
where
	N: Send + Sync + VocabularyMut<Iri=T, BlankId=B>,
	T: Clone + Eq + Hash + Send + Sync,
	B: Clone + Eq + Hash + Send + Sync,
	M: Clone + Send + Sync,
	C: ProcessMeta<T, B, M> + From<json_ld_syntax::context::Value<M>>,
	L: Send + Sync,
	L::Output: Into<Value<M>>,
	L::Context: Into<C>,
	L::ContextError: Send,
	W: 'a + Send + WarningHandler<B, N, M>,
{
	let (expanded, _) = expand_element(
		vocabulary,
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
			Ok(Meta(graph, meta)) => Ok(Meta(ExpandedDocument::from(graph), meta)),
			Err(obj) => {
				let obj = Meta(obj, meta.clone());
				let mut result = ExpandedDocument::new();
				if filter_top_level_item(&obj) {
					result.insert(obj);
				}
				Ok(Meta(result, meta))
			}
		}
	} else {
		Ok(Meta(
			expanded.into_iter().filter(filter_top_level_item).collect(),
			document.metadata().clone(),
		))
	}
}

pub(crate) fn filter_top_level_item<T, B, M>(Meta(item, _): &IndexedObject<T, B, M>) -> bool {
	// Remove dangling values.
	!matches!(item.inner(), Object::Value(_))
}
