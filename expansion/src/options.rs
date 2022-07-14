use json_ld_context_processing::ProcessingOptions;
use json_ld_core::ProcessingMode;

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

impl From<Options> for ProcessingOptions {
	fn from(options: Options) -> ProcessingOptions {
		ProcessingOptions {
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Policy {
	/// Relaxed policy.
	///
	/// Undefined keys are always kept in the expanded document
	/// using the [`Reference::Invalid`](crate::Reference::Invalid) variant.
	Relaxed,

	/// Standard policy.
	///
	/// Every key that cannot be expanded into an
	/// IRI or a blank node identifier is dropped unless it contains a `:` character.
	Standard,

	/// Strict policy.
	///
	/// Every key that cannot be expanded into an IRI or a blank node identifier
	/// will raise an error unless the term contains a `:` character.
	Strict,

	/// Strictest policy.
	///
	/// Every key that cannot be expanded into an IRI or a blank node identifier
	/// will raise an error.
	Strictest,
}

impl Policy {
	/// Returns `true` is the policy is `Strict` or `Strictest`.
	pub fn is_strict(&self) -> bool {
		matches!(self, Self::Strict | Self::Strictest)
	}
}

impl Default for Policy {
	fn default() -> Self {
		Self::Standard
	}
}
