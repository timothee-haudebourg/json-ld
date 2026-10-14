use json_syntax::{JsonValue, object::Key};

use crate::{
	JsonLdExpandedDocument,
	algorithms::{
		AsyncProcessingEnvironment, JsonLdBacktraceBuilder, JsonLdError, JsonLdLocatedError,
	},
};

use super::{Expander, filter_top_level_item};

impl<'a> Expander<'a> {
	pub async fn expand_document(
		&self,
		env: &impl AsyncProcessingEnvironment,
		document: &JsonValue,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonLdExpandedDocument, JsonLdLocatedError> {
		// JSON-LD requires object keys to be unique throughout a document
		// Check this once, up front, for the whole document: every function
		// downstream of this point assumes it never encounters a duplicate key
		// and does not re-check.
		if let Some((path, key)) = document.find_duplicate_key_recursive() {
			return Err(JsonLdError::DuplicateKey(Key::from(key)).at(location.push_path(&path)));
		}

		let expanded = self.expand_element(env, document, false, location).await?;

		if expanded.len() == 1 {
			let obj = expanded.into_iter().next().unwrap();
			match obj.into_unnamed_graph() {
				Ok(graph) => Ok(JsonLdExpandedDocument::from(graph)),
				Err(obj) => {
					let mut result = JsonLdExpandedDocument::new();
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
