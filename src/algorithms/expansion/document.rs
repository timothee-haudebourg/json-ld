use json_syntax::Value;

use crate::{
	algorithms::{Error, ProcessingEnvironment},
	ExpandedDocument,
};

use super::{filter_top_level_item, Expander};

impl<'a> Expander<'a> {
	pub async fn expand_document(
		&self,
		env: &mut impl ProcessingEnvironment,
		document: &Value,
	) -> Result<ExpandedDocument, Error> {
		let expanded = self.expand_element(env, document, false).await?;

		if expanded.len() == 1 {
			let obj = expanded.into_iter().next().unwrap();
			match obj.into_unnamed_graph() {
				Ok(graph) => Ok(ExpandedDocument::from(graph)),
				Err(obj) => {
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
}
