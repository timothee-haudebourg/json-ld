use std::{hash::Hash, str::FromStr};

/// Context type.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JsonLdContextType {
	#[cfg_attr(feature = "serde", serde(rename = "@container"))]
	pub container: JsonLdContextTypeContainer,

	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@protected",
			default,
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub protected: Option<bool>,
}

impl JsonLdContextType {
	pub fn iter(&self) -> JsonLdContextTypeEntries {
		JsonLdContextTypeEntries {
			container: Some(self.container),
			protected: self.protected,
		}
	}
}

pub struct JsonLdContextTypeEntries {
	container: Option<JsonLdContextTypeContainer>,
	protected: Option<bool>,
}

impl Iterator for JsonLdContextTypeEntries {
	type Item = JsonLdContextTypeEntry;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let mut len = 0;

		if self.container.is_some() {
			len += 1;
		}

		if self.protected.is_some() {
			len += 1;
		}

		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		match self.container.take() {
			Some(c) => Some(JsonLdContextTypeEntry::Container(c)),
			None => self.protected.take().map(JsonLdContextTypeEntry::Protected),
		}
	}
}

impl ExactSizeIterator for JsonLdContextTypeEntries {}

pub enum JsonLdContextTypeEntry {
	Container(JsonLdContextTypeContainer),
	Protected(bool),
}

impl JsonLdContextTypeEntry {
	pub fn key(&self) -> JsonLdContextTypeKey {
		match self {
			Self::Container(_) => JsonLdContextTypeKey::Container,
			Self::Protected(_) => JsonLdContextTypeKey::Protected,
		}
	}
}

pub enum JsonLdContextTypeKey {
	Container,
	Protected,
}

impl JsonLdContextTypeKey {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Container => "@container",
			Self::Protected => "@protected",
		}
	}
}

#[derive(Debug, thiserror::Error)]
#[error("invalid JSON-LD `@type` container `{0}`")]
pub struct InvalidJsonLdTypeContainer<T = String>(pub T);

#[derive(Clone, Copy, PartialOrd, Ord, Debug)]
pub enum JsonLdContextTypeContainer {
	Set,
}

impl JsonLdContextTypeContainer {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Set => "@set",
		}
	}

	pub fn into_str(self) -> &'static str {
		self.as_str()
	}
}

impl PartialEq for JsonLdContextTypeContainer {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl Eq for JsonLdContextTypeContainer {}

impl Hash for JsonLdContextTypeContainer {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.into_str().hash(state)
	}
}

impl FromStr for JsonLdContextTypeContainer {
	type Err = InvalidJsonLdTypeContainer;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"@set" => Ok(Self::Set),
			other => Err(InvalidJsonLdTypeContainer(other.to_owned())),
		}
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for JsonLdContextTypeContainer {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.as_str().serialize(serializer)
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for JsonLdContextTypeContainer {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor;

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = JsonLdContextTypeContainer;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("JSON-LD type container")
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
