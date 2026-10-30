use std::{fmt, str::FromStr};

#[derive(Debug, thiserror::Error)]
#[error("invalid JSON-LD text direction `{0}`")]
pub struct InvalidJsonLdDirection<T>(pub T);

impl<T: ?Sized + ToOwned> InvalidJsonLdDirection<&T> {
	pub fn into_owned(self) -> InvalidJsonLdDirection<T::Owned> {
		InvalidJsonLdDirection(self.0.to_owned())
	}
}

/// Internationalized string direction.
///
/// Specifies the direction used to read a string.
/// This can be either left-to-right (`"ltr"`) or right-to-left (`"rtl"`).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum JsonLdDirection {
	/// Left-to-right direction.
	Ltr,

	/// Right-to-left direction.
	Rtl,
}

impl JsonLdDirection {
	pub fn as_str(&self) -> &'static str {
		match self {
			JsonLdDirection::Ltr => "ltr",
			JsonLdDirection::Rtl => "rtl",
		}
	}

	pub fn into_str(self) -> &'static str {
		self.as_str()
	}
}

impl FromStr for JsonLdDirection {
	type Err = InvalidJsonLdDirection<String>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::try_from(s).map_err(InvalidJsonLdDirection::into_owned)
	}
}

impl<'a> TryFrom<&'a str> for JsonLdDirection {
	type Error = InvalidJsonLdDirection<&'a str>;

	/// Convert the strings `"rtl"` and `"ltr"` into a `Direction`.
	#[inline(always)]
	fn try_from(name: &'a str) -> Result<JsonLdDirection, InvalidJsonLdDirection<&'a str>> {
		match name {
			"ltr" => Ok(JsonLdDirection::Ltr),
			"rtl" => Ok(JsonLdDirection::Rtl),
			_ => Err(InvalidJsonLdDirection(name)),
		}
	}
}

impl fmt::Display for JsonLdDirection {
	#[inline(always)]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			JsonLdDirection::Ltr => write!(f, "ltr"),
			JsonLdDirection::Rtl => write!(f, "rtl"),
		}
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for JsonLdDirection {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.as_str().serialize(serializer)
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for JsonLdDirection {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor;

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = JsonLdDirection;

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
