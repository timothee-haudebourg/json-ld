use std::{hash::Hash, str::FromStr};

use json_syntax::{JsonNumber, JsonNumberBuf};

#[derive(Debug, thiserror::Error)]
#[error("unknown JSON-LD version `{0}`")]
pub struct UnknownJsonLdVersion(pub String);

/// Version number.
///
/// The only allowed value is a number with the value `1.1`.
#[derive(Clone, Copy, PartialOrd, Ord, Debug)]
pub enum JsonLdVersion {
	V1_1,
}

impl JsonLdVersion {
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

	pub fn into_json_number(self) -> &'static JsonNumber {
		unsafe { JsonNumber::new_unchecked(self.into_bytes()) }
	}

	pub fn into_json_number_buf(self) -> JsonNumberBuf {
		unsafe { JsonNumberBuf::new_unchecked(self.into_bytes().into()) }
	}
}

impl PartialEq for JsonLdVersion {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl Eq for JsonLdVersion {}

impl Hash for JsonLdVersion {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.into_str().hash(state)
	}
}

impl From<JsonLdVersion> for &JsonNumber {
	fn from(v: JsonLdVersion) -> Self {
		v.into_json_number()
	}
}

impl From<JsonLdVersion> for JsonNumberBuf {
	fn from(v: JsonLdVersion) -> Self {
		v.into_json_number_buf()
	}
}

impl TryFrom<JsonNumberBuf> for JsonLdVersion {
	type Error = UnknownJsonLdVersion;

	fn try_from(value: JsonNumberBuf) -> Result<Self, Self::Error> {
		if value.trimmed().as_str() == "1.1" {
			Ok(Self::V1_1)
		} else {
			Err(UnknownJsonLdVersion(value.to_string()))
		}
	}
}

impl FromStr for JsonLdVersion {
	type Err = UnknownJsonLdVersion;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s == "1.1" {
			Ok(JsonLdVersion::V1_1)
		} else {
			Err(UnknownJsonLdVersion(s.to_owned()))
		}
	}
}

impl TryFrom<f32> for JsonLdVersion {
	type Error = UnknownJsonLdVersion;

	fn try_from(value: f32) -> Result<Self, Self::Error> {
		if value == 1.1 {
			Ok(JsonLdVersion::V1_1)
		} else {
			Err(UnknownJsonLdVersion(value.to_string()))
		}
	}
}

impl TryFrom<f64> for JsonLdVersion {
	type Error = UnknownJsonLdVersion;

	fn try_from(value: f64) -> Result<Self, Self::Error> {
		if value == 1.1 {
			Ok(JsonLdVersion::V1_1)
		} else {
			Err(UnknownJsonLdVersion(value.to_string()))
		}
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for JsonLdVersion {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.into_json_number().serialize(serializer)
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for JsonLdVersion {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		JsonNumberBuf::deserialize(deserializer)?
			.try_into()
			.map_err(serde::de::Error::custom)
	}
}
