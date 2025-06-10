use std::{hash::Hash, str::FromStr};

#[derive(Debug, thiserror::Error)]
#[error("unknown JSON-LD version `{0}`")]
pub struct UnknownVersion(pub String);

/// Version number.
///
/// The only allowed value is a number with the value `1.1`.
#[derive(Clone, Copy, PartialOrd, Ord, Debug)]
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

impl From<Version> for &json_syntax::Number {
	fn from(v: Version) -> Self {
		v.into_json_number()
	}
}

impl From<Version> for json_syntax::NumberBuf {
	fn from(v: Version) -> Self {
		v.into_json_number_buf()
	}
}

impl TryFrom<json_syntax::NumberBuf> for Version {
	type Error = UnknownVersion;

	fn try_from(value: json_syntax::NumberBuf) -> Result<Self, Self::Error> {
		if value.trimmed().as_str() == "1.1" {
			Ok(Self::V1_1)
		} else {
			Err(UnknownVersion(value.to_string()))
		}
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

impl TryFrom<f32> for Version {
	type Error = UnknownVersion;

	fn try_from(value: f32) -> Result<Self, Self::Error> {
		if value == 1.1 {
			Ok(Version::V1_1)
		} else {
			Err(UnknownVersion(value.to_string()))
		}
	}
}

impl TryFrom<f64> for Version {
	type Error = UnknownVersion;

	fn try_from(value: f64) -> Result<Self, Self::Error> {
		if value == 1.1 {
			Ok(Version::V1_1)
		} else {
			Err(UnknownVersion(value.to_string()))
		}
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for Version {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.into_json_number().serialize(serializer)
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Version {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		json_syntax::NumberBuf::deserialize(deserializer)?
			.try_into()
			.map_err(serde::de::Error::custom)
	}
}
