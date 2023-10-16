use crate::context::Entry;
use locspan_derive::StrippedPartialEq;
use std::{hash::Hash, str::FromStr};

#[derive(Clone, Copy, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(deserialize = "M: Default")))]
#[locspan(ignore(M))]
pub struct Type<M> {
	#[cfg_attr(feature = "serde", serde(rename = "@container"))]
	pub container: Entry<TypeContainer, M>,

	#[cfg_attr(feature = "serde", serde(rename = "@protected"))]
	pub protected: Option<Entry<bool, M>>,
}

impl<M> Type<M> {
	pub fn iter(&self) -> ContextTypeEntries<M> {
		ContextTypeEntries {
			container: Some(&self.container),
			protected: self.protected.as_ref(),
		}
	}
}

pub struct ContextTypeEntries<'a, M> {
	container: Option<&'a Entry<TypeContainer, M>>,
	protected: Option<&'a Entry<bool, M>>,
}

impl<'a, M> Iterator for ContextTypeEntries<'a, M> {
	type Item = ContextTypeEntry<'a, M>;

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
			Some(c) => Some(ContextTypeEntry::Container(c)),
			None => self.protected.take().map(ContextTypeEntry::Protected),
		}
	}
}

impl<'a, M> ExactSizeIterator for ContextTypeEntries<'a, M> {}

pub enum ContextTypeEntry<'a, M> {
	Container(&'a Entry<TypeContainer, M>),
	Protected(&'a Entry<bool, M>),
}

impl<'a, M> ContextTypeEntry<'a, M> {
	pub fn key(&self) -> ContextTypeKey {
		match self {
			Self::Container(_) => ContextTypeKey::Container,
			Self::Protected(_) => ContextTypeKey::Protected,
		}
	}
}

pub enum ContextTypeKey {
	Container,
	Protected,
}

impl ContextTypeKey {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Container => "@container",
			Self::Protected => "@protected",
		}
	}
}

#[derive(Debug, thiserror::Error)]
#[error("invalid JSON-LD `@type` container `{0}`")]
pub struct InvalidTypeContainer<T = String>(pub T);

#[derive(Clone, Copy, StrippedPartialEq, PartialOrd, Ord, Debug)]
pub enum TypeContainer {
	Set,
}

impl TypeContainer {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Set => "@set",
		}
	}

	pub fn into_str(self) -> &'static str {
		self.as_str()
	}
}

impl PartialEq for TypeContainer {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl Eq for TypeContainer {}

impl Hash for TypeContainer {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.into_str().hash(state)
	}
}

impl FromStr for TypeContainer {
	type Err = InvalidTypeContainer;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"@set" => Ok(Self::Set),
			other => Err(InvalidTypeContainer(other.to_owned())),
		}
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for TypeContainer {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.as_str().serialize(serializer)
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for TypeContainer {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor;

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = TypeContainer;

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
