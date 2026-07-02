//! Flattening algorithm and related types.
use rdf_syntax::Generator;

use crate::{
	JsonLdExpandedDocument, JsonLdFlattenedDocument, flattened::JsonLdUnorderedFlattenedDocument,
};

mod node_map;

pub use node_map::*;

use crate::algorithms::JsonLdBacktraceBuilder;

impl JsonLdExpandedDocument {
	pub fn flatten(
		&self,
		generator: impl Generator,
		ordered: bool,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonLdFlattenedDocument, NodeMapExtendError> {
		Ok(self
			.generate_node_map_with(generator, location)?
			.flatten(ordered))
	}

	pub fn flatten_unordered(
		&self,
		generator: impl Generator,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonLdUnorderedFlattenedDocument, NodeMapExtendError> {
		Ok(self
			.generate_node_map_with(generator, location)?
			.flatten_unordered())
	}
}
