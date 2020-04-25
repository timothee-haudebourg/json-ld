use std::convert::TryFrom;
use std::fmt;
use json::JsonValue;
use crate::util::AsJson;

/// Internationalized string direction.
///
/// Specifies the direction used to read a string.
/// This can be either left-to-right (`"ltr"`) or right-to-left (`"rtl"`).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Direction {
	/// Left-to-right direction.
	Ltr,

	/// Right-to-left direction.
	Rtl
}

impl<'a> TryFrom<&'a str> for Direction {
	type Error = &'a str;

	/// Convert the strings `"rtl"` and `"ltr"` into a `Direction`.
	fn try_from(name: &'a str) -> Result<Direction, &'a str> {
		match name {
			"ltr" => Ok(Direction::Ltr),
			"rtl" => Ok(Direction::Rtl),
			_ => Err(name)
		}
	}
}

impl fmt::Display for Direction {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Direction::Ltr => write!(f, "ltr"),
			Direction::Rtl => write!(f, "rtl")
		}
	}
}

impl AsJson for Direction {
	/// Convert the direction into a JSON string.
	/// Either `"rtl"` or `"ltr"`.
	fn as_json(&self) -> JsonValue {
		match self {
			Direction::Ltr => "ltr".into(),
			Direction::Rtl => "rtl".into()
		}
	}
}
