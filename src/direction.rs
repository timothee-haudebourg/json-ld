use std::convert::TryFrom;
use std::fmt;
use json::JsonValue;
use crate::AsJson;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
	Ltr,
	Rtl
}

impl<'a> TryFrom<&'a str> for Direction {
	type Error = &'a str;

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
	fn as_json(&self) -> JsonValue {
		match self {
			Direction::Ltr => "ltr".into(),
			Direction::Rtl => "rtl".into()
		}
	}
}
