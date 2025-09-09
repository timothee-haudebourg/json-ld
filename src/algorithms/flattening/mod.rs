//! Flattening algorithm and related types.
use rdf_types::Generator;

use crate::{flattened::UnorderedFlattenedDocument, ExpandedDocument, FlattenedDocument};

mod node_map;

pub use node_map::*;

impl ExpandedDocument {
	pub fn flatten(
		self,
		generator: impl Generator,
		ordered: bool,
	) -> Result<FlattenedDocument, ConflictingIndexes> {
		Ok(self.generate_node_map_with(generator)?.flatten(ordered))
	}

	pub fn flatten_unordered(
		self,
		generator: impl Generator,
	) -> Result<UnorderedFlattenedDocument, ConflictingIndexes> {
		Ok(self.generate_node_map_with(generator)?.flatten_unordered())
	}
}
