use std::convert::TryFrom;
use std::fmt;

/// Processing mode.
///
/// This is a property of the context processing and compaction options.
/// New features defined in JSON-LD 1.1 are available unless the processing mode is set to [`ProcessingMode::JsonLd1_0`].
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ProcessingMode {
	/// JSON-LD 1.0.
	JsonLd1_0,

	/// JSON-LD 1.1.
	#[default]
	JsonLd1_1,
}

impl ProcessingMode {
	/// Returns the name of the processing mode.
	#[inline(always)]
	pub fn as_str(&self) -> &str {
		match self {
			ProcessingMode::JsonLd1_0 => "json-ld-1.0",
			ProcessingMode::JsonLd1_1 => "json-ld-1.1",
		}
	}
}

impl<'a> TryFrom<&'a str> for ProcessingMode {
	type Error = ();

	fn try_from(name: &'a str) -> Result<ProcessingMode, ()> {
		match name {
			"json-ld-1.0" => Ok(ProcessingMode::JsonLd1_0),
			"json-ld-1.1" => Ok(ProcessingMode::JsonLd1_1),
			_ => Err(()),
		}
	}
}

impl fmt::Display for ProcessingMode {
	#[inline(always)]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
