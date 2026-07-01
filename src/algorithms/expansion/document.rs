use json_syntax::JsonValue;

use crate::{
	ExpandedDocument,
	algorithms::{AsyncProcessingEnvironment, JsonLdLocatedError, JsonLdLocationStack},
};

use super::{Expander, filter_top_level_item};

impl<'a> Expander<'a> {
	pub async fn expand_document(
		&self,
		env: &impl AsyncProcessingEnvironment,
		document: &JsonValue,
		location: JsonLdLocationStack<'_>,
	) -> Result<ExpandedDocument, JsonLdLocatedError> {
		let expanded = self.expand_element(env, document, false, location).await?;

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
