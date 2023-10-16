use locspan_derive::StrippedPartialEq;
use std::{hash::Hash, str::FromStr};

#[derive(Debug, thiserror::Error)]
#[error("unknown JSON-LD version `{0}`")]
pub struct UnknownVersion(pub String);

/// Version number.
///
/// The only allowed value is a number with the value `1.1`.
#[derive(Clone, Copy, StrippedPartialEq, PartialOrd, Ord, Debug)]
pub enum Version {
	V1_1,
}

impl Version {
	pub fn into_bytes(self) -> &'static [u8] {
		match self {
			Self::V1_1 => b"1.1",
		}
	}

	pub fn into_str(self) -> &'static str {
		match self {
			Self::V1_1 => "1.1",
		}
	}

	pub fn into_json_number(self) -> &'static json_syntax::Number {
		unsafe { json_syntax::Number::new_unchecked(self.into_bytes()) }
	}

	pub fn into_json_number_buf(self) -> json_syntax::NumberBuf {
		unsafe { json_syntax::NumberBuf::new_unchecked(self.into_bytes().into()) }
	}
}

impl PartialEq for Version {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl Eq for Version {}

impl Hash for Version {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.into_str().hash(state)
	}
}

impl<'a> From<Version> for &'a json_syntax::Number {
	fn from(v: Version) -> Self {
		v.into_json_number()
	}
}

impl From<Version> for json_syntax::NumberBuf {
	fn from(v: Version) -> Self {
		v.into_json_number_buf()
	}
}

impl FromStr for Version {
	type Err = UnknownVersion;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s == "1.1" {
			Ok(Version::V1_1)
		} else {
			Err(UnknownVersion(s.to_owned()))
		}
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for Version {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.into_str().serialize(serializer)
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Version {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor;

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = Version;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("JSON-LD version")
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
