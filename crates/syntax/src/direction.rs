use std::{fmt, str::FromStr};

#[derive(Debug, thiserror::Error)]
#[error("invalid JSON-LD text direction `{0}`")]
pub struct InvalidDirection<T>(pub T);

impl<'a, T: ?Sized + ToOwned> InvalidDirection<&'a T> {
	pub fn into_owned(self) -> InvalidDirection<T::Owned> {
		InvalidDirection(self.0.to_owned())
	}
}

/// Internationalized string direction.
///
/// Specifies the direction used to read a string.
/// This can be either left-to-right (`"ltr"`) or right-to-left (`"rtl"`).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Direction {
	/// Left-to-right direction.
	Ltr,

	/// Right-to-left direction.
	Rtl,
}

impl Direction {
	pub fn as_str(&self) -> &'static str {
		match self {
			Direction::Ltr => "ltr",
			Direction::Rtl => "rtl",
		}
	}

	pub fn into_str(self) -> &'static str {
		self.as_str()
	}
}

impl FromStr for Direction {
	type Err = InvalidDirection<String>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::try_from(s).map_err(InvalidDirection::into_owned)
	}
}

impl<'a> TryFrom<&'a str> for Direction {
	type Error = InvalidDirection<&'a str>;

	/// Convert the strings `"rtl"` and `"ltr"` into a `Direction`.
	#[inline(always)]
	fn try_from(name: &'a str) -> Result<Direction, InvalidDirection<&'a str>> {
		match name {
			"ltr" => Ok(Direction::Ltr),
			"rtl" => Ok(Direction::Rtl),
			_ => Err(InvalidDirection(name)),
		}
	}
}

impl fmt::Display for Direction {
	#[inline(always)]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Direction::Ltr => write!(f, "ltr"),
			Direction::Rtl => write!(f, "rtl"),
		}
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for Direction {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.as_str().serialize(serializer)
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Direction {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor;

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = Direction;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("JSON-LD text direction")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				v.parse().map_err(|e| E::custom(e))
			}
		}

		deserializer.deserialize_str(Visitor)
	}
}
