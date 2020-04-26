use std::fmt;
use std::convert::TryFrom;

/// Processing modes
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ProcessingMode {
	/// JSON-LD 1.0.
	JsonLd1_0,

	/// JSON-LD 1.1.
	JsonLd1_1
}

impl ProcessingMode {
	pub fn as_str(&self) -> &str {
		match self {
			ProcessingMode::JsonLd1_0 => "json-ld-1.0",
			ProcessingMode::JsonLd1_1 => "json-ld-1.1"
		}
	}
}

impl Default for ProcessingMode {
	fn default() -> ProcessingMode {
		ProcessingMode::JsonLd1_1
	}
}

impl<'a> TryFrom<&'a str> for ProcessingMode {
	type Error = ();

	fn try_from(name: &'a str) -> Result<ProcessingMode, ()> {
		match name {
			"json-ld-1.0" => Ok(ProcessingMode::JsonLd1_0),
			"json-ld-1.1" => Ok(ProcessingMode::JsonLd1_1),
			_ => Err(())
		}
	}
}

impl fmt::Display for ProcessingMode {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
