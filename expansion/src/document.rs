use json_ld_core::{
	Id,
	Context
};

/// Expand the given JSON-LD document.
///
/// Note that you probably do not want to use this function directly,
/// but instead use the [`Document::expand`](crate::Document::expand) method, implemented for
/// every JSON type implementing the [`generic_json::Json`] trait.
pub async fn expand<'a, T: Id, C, L: Loader>(
	active_context: &'a C,
	document: &'a J,
	base_url: Option<IriBuf>,
	loader: &'a mut L,
	options: Options,
	warnings: &mut Vec<Loc<Warning, J::MetaData>>,
) -> Result<HashSet<Indexed<Object<J, T>>>, Loc<Error, J::MetaData>>
where
	T: Send + Sync,
	C: Send + Sync,
	C::LocalContext: From<L::Output> + From<J>,
	L: Send + Sync,
	L::Output: Into<J>,
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
