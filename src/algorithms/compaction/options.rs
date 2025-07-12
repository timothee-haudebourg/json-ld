use crate::{
	algorithms::{context_processing::ContextProcessingOptions, expansion::ExpansionOptions},
	ProcessingMode,
};

/// Compaction options.
#[derive(Clone, Copy)]
pub struct CompactionOptions {
	/// JSON-LD processing mode.
	pub processing_mode: ProcessingMode,

	/// Determines if IRIs are compacted relative to the provided base IRI or document location when compacting.
	pub compact_to_relative: bool,

	/// If set to `true`, arrays with just one element are replaced with that element during compaction.
	/// If set to `false`, all arrays will remain arrays even if they have just one element.
	pub compact_arrays: bool,

	/// If set to `true`, properties are processed by lexical order.
	/// If `false`, order is not considered in processing.
	pub ordered: bool,
}

impl CompactionOptions {
	pub fn unordered(self) -> Self {
		Self {
			ordered: false,
			..self
		}
	}
}

impl From<CompactionOptions> for ContextProcessingOptions {
	fn from(options: CompactionOptions) -> Self {
		Self {
			processing_mode: options.processing_mode,
			..Self::default()
		}
	}
}

impl From<ExpansionOptions> for CompactionOptions {
	fn from(options: ExpansionOptions) -> Self {
		Self {
			processing_mode: options.processing_mode,
			ordered: options.ordered,
			..Self::default()
		}
	}
}

impl Default for CompactionOptions {
	fn default() -> Self {
		Self {
			processing_mode: ProcessingMode::default(),
			compact_to_relative: true,
			compact_arrays: true,
			ordered: false,
		}
	}
}
