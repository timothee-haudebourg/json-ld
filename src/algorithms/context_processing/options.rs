use crate::JsonLdProcessingMode;

/// Options of the Context Processing Algorithm.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct JsonLdContextProcessingOptions {
	/// The processing mode
	pub processing_mode: JsonLdProcessingMode,

	/// Override protected definitions.
	pub override_protected: bool,

	/// Propagate the processed context.
	pub propagate: bool,
}

impl JsonLdContextProcessingOptions {
	/// Return the same set of options, but with `override_protected` set to `true`.
	#[must_use]
	pub fn with_override(&self) -> JsonLdContextProcessingOptions {
		let mut opt = *self;
		opt.override_protected = true;
		opt
	}

	/// Return the same set of options, but with `override_protected` set to `false`.
	#[must_use]
	pub fn with_no_override(&self) -> JsonLdContextProcessingOptions {
		let mut opt = *self;
		opt.override_protected = false;
		opt
	}

	/// Return the same set of options, but with `propagate` set to `false`.
	#[must_use]
	pub fn without_propagation(&self) -> JsonLdContextProcessingOptions {
		let mut opt = *self;
		opt.propagate = false;
		opt
	}
}

impl Default for JsonLdContextProcessingOptions {
	fn default() -> JsonLdContextProcessingOptions {
		JsonLdContextProcessingOptions {
			processing_mode: JsonLdProcessingMode::default(),
			override_protected: false,
			propagate: true,
		}
	}
}
