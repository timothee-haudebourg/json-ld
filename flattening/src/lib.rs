//! Flattening algorithm and related types.
use crate::{
	compaction, expansion, id, ExpandedDocument, FlattenedDocument, Id, Indexed, Node, Object,
	ProcessingMode,
};
use generic_json::{JsonClone, JsonHash};
use std::collections::HashSet;

mod vocabulary;
mod node_map;

pub use vocabulary::Namespace;
pub use node_map::*;

/// Flattening options.
#[derive(Clone, Copy, Default)]
pub struct Options {
	/// Sets the processing mode.
	pub processing_mode: ProcessingMode,

	/// Term expansion policy.
	///
	/// Default is `Policy::Standard`.
	pub policy: expansion::Policy,

	/// Determines if IRIs are compacted relative to the provided base IRI or document location when compacting.
	pub compact_to_relative: bool,

	/// If set to `true`, arrays with just one element are replaced with that element during compaction.
	/// If set to `false`, all arrays will remain arrays even if they have just one element.
	pub compact_arrays: bool,

	/// If set to true, input document entries are processed lexicographically.
	/// If false, order is not considered in processing.
	pub ordered: bool,
}

impl From<Options> for expansion::Options {
	fn from(o: Options) -> Self {
		Self {
			processing_mode: o.processing_mode,
			policy: o.policy,
			ordered: false,
		}
	}
}

impl From<Options> for compaction::Options {
	fn from(o: Options) -> Self {
		Self {
			processing_mode: o.processing_mode,
			compact_to_relative: o.compact_to_relative,
			compact_arrays: o.compact_arrays,
			ordered: false,
		}
	}
}