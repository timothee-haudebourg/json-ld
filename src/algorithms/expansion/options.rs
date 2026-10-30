use crate::{JsonLdProcessingMode, algorithms::context_processing::JsonLdContextProcessingOptions};

/// Expansion options.
#[derive(Clone, Copy, Default)]
pub struct JsonLdExpansionOptions {
	/// Sets the processing mode.
	pub processing_mode: JsonLdProcessingMode,

	/// Term expansion policy.
	///
	/// Default is `JsonLdExpansionPolicy::Standard`.
	pub policy: JsonLdExpansionPolicy,

	/// If set to true, input document entries are processed lexicographically.
	/// If false, order is not considered in processing.
	pub ordered: bool,
}

impl JsonLdExpansionOptions {
	pub fn unordered(self) -> Self {
		Self {
			ordered: false,
			..self
		}
	}
}

impl From<JsonLdExpansionOptions> for JsonLdContextProcessingOptions {
	fn from(options: JsonLdExpansionOptions) -> JsonLdContextProcessingOptions {
		JsonLdContextProcessingOptions {
			processing_mode: options.processing_mode,
			..Default::default()
		}
	}
}

/// Key expansion policy.
///
/// The default behavior of the expansion algorithm
/// is to drop keys that are not defined in the context unless:
///  - there is a vocabulary mapping (`@vocab`) defined in the context; or
///  - the term contains a `:` character.
///
/// In other words, a key that cannot be expanded into an
/// IRI or a blank node identifier is dropped unless it contains a `:` character.
///
/// Sometimes, it is preferable to keep undefined keys in the
/// expanded document, or to forbid them completely by raising an error.
/// You can define your preferred policy using one of this type variant
/// with the [`JsonLdExpansionOptions::policy`] field.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum JsonLdExpansionPolicy {
	/// Relaxed policy.
	///
	/// Undefined keys are always kept in the expanded document
	/// using the [`JsonLdLenient::Invalid`](crate::JsonLdLenient::Invalid) variant.
	Relaxed,

	/// Standard policy.
	///
	/// Every key that cannot be expanded into an
	/// IRI or a blank node identifier is dropped unless it contains a `:` character.
	#[default]
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

impl JsonLdExpansionPolicy {
	/// Returns `true` is the policy is `Strict` or `Strictest`.
	pub fn is_strict(&self) -> bool {
		matches!(self, Self::Strict | Self::Strictest)
	}
}
