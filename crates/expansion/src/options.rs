use json_ld_core::ProcessingMode;

pub use json_ld_context_processing::algorithm::Action;

/// Expansion options.
#[derive(Clone, Copy, Default)]
pub struct Options {
	/// Sets the processing mode.
	pub processing_mode: ProcessingMode,

	/// Term expansion policy.
	///
	/// Default is `Policy::Standard`.
	pub policy: Policy,

	/// If set to true, input document entries are processed lexicographically.
	/// If false, order is not considered in processing.
	pub ordered: bool,
}

impl Options {
	pub fn unordered(self) -> Self {
		Self {
			ordered: false,
			..self
		}
	}
}

impl From<Options> for json_ld_context_processing::Options {
	fn from(options: Options) -> json_ld_context_processing::Options {
		json_ld_context_processing::Options {
			processing_mode: options.processing_mode,
			..Default::default()
		}
	}
}

/// Key expansion policy.
///
/// The default behavior of the expansion algorithm
/// is to drop keys that are not defined in the context unless:
///   - there is a vocabulary mapping (`@vocab`) defined in the context; or
///   - the term contains a `:` character.
/// In other words, a key that cannot be expanded into an
/// IRI or a blank node identifier is dropped unless it contains a `:` character.
///
/// Sometimes, it is preferable to keep undefined keys in the
/// expanded document, or to forbid them completely by raising an error.
/// You can define your preferred policy using one of this type variant
/// with the [`Options::policy`] field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
	/// How to expand invalid terms.
	pub invalid: Action,

	/// How to expand valid terms that need a vocabulary mapping
	/// (`@vocab` keyword).
	pub vocab: Action,

	/// How to expand valid terms when there is no vocabulary mapping.
	pub allow_undefined: bool,
}

impl Default for Policy {
	fn default() -> Self {
		Self {
			invalid: Action::Keep,
			vocab: Action::Keep,
			allow_undefined: true,
		}
	}
}
