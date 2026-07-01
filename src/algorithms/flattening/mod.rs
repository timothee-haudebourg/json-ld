//! Flattening algorithm and related types.
use rdf_syntax::Generator;

use crate::{ExpandedDocument, FlattenedDocument, flattened::UnorderedFlattenedDocument};

mod node_map;

pub use node_map::*;

use crate::algorithms::{JsonLdLocated, JsonLdLocationStack};

impl ExpandedDocument {
	pub fn flatten(
		self,
		generator: impl Generator,
		ordered: bool,
		location: JsonLdLocationStack<'_>,
	) -> Result<FlattenedDocument, NodeMapExtendError> {
		Ok(self
			.generate_node_map_with(generator, location)?
			.flatten(ordered))
	}

	pub fn flatten_unordered(
		self,
		generator: impl Generator,
		location: JsonLdLocationStack<'_>,
	) -> Result<UnorderedFlattenedDocument, NodeMapExtendError> {
		Ok(self
			.generate_node_map_with(generator, location)?
			.flatten_unordered())
	}
}
