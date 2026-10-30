use std::str::FromStr;

use crate::context::JsonLdContainer;
use crate::syntax::context::JsonLdContextTypeContainer;

use super::JsonLdKeyword;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JsonLdContainerItem {
	#[cfg_attr(feature = "serde", serde(rename = "@graph"))]
	Graph,

	#[cfg_attr(feature = "serde", serde(rename = "@id"))]
	Id,

	#[cfg_attr(feature = "serde", serde(rename = "@index"))]
	Index,

	#[cfg_attr(feature = "serde", serde(rename = "@language"))]
	Language,

	#[cfg_attr(feature = "serde", serde(rename = "@list"))]
	List,

	#[cfg_attr(feature = "serde", serde(rename = "@set"))]
	Set,

	#[cfg_attr(feature = "serde", serde(rename = "@type"))]
	Type,
}

impl JsonLdContainerItem {
	pub fn into_keyword(self) -> JsonLdKeyword {
		self.into()
	}

	pub fn keyword(&self) -> JsonLdKeyword {
		self.into_keyword()
	}

	pub fn as_str(&self) -> &'static str {
		self.into_keyword().into_str()
	}
}

impl std::fmt::Display for JsonLdContainerItem {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.as_str())
	}
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Invalid `@container` item: `{0}`")]
pub struct InvalidJsonLdContainerItem<T = String>(pub T);

impl<T: ?Sized + ToOwned> InvalidJsonLdContainerItem<&T> {
	pub fn into_owned(self) -> InvalidJsonLdContainerItem<T::Owned> {
		InvalidJsonLdContainerItem(self.0.to_owned())
	}
}

impl FromStr for JsonLdContainerItem {
	type Err = InvalidJsonLdContainerItem;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		s.try_into().map_err(InvalidJsonLdContainerItem::into_owned)
	}
}

impl<'a> TryFrom<&'a str> for JsonLdContainerItem {
	type Error = InvalidJsonLdContainerItem<&'a str>;

	fn try_from(str: &'a str) -> Result<JsonLdContainerItem, InvalidJsonLdContainerItem<&'a str>> {
		use JsonLdContainerItem::*;
		match str {
			"@graph" => Ok(Graph),
			"@id" => Ok(Id),
			"@index" => Ok(Index),
			"@language" => Ok(Language),
			"@list" => Ok(List),
			"@set" => Ok(Set),
			"@type" => Ok(Type),
			_ => Err(InvalidJsonLdContainerItem(str)),
		}
	}
}

impl TryFrom<JsonLdKeyword> for JsonLdContainerItem {
	type Error = JsonLdKeyword;

	fn try_from(k: JsonLdKeyword) -> Result<JsonLdContainerItem, JsonLdKeyword> {
		use JsonLdContainerItem::*;
		match k {
			JsonLdKeyword::Graph => Ok(Graph),
			JsonLdKeyword::Id => Ok(Id),
			JsonLdKeyword::Index => Ok(Index),
			JsonLdKeyword::Language => Ok(Language),
			JsonLdKeyword::List => Ok(List),
			JsonLdKeyword::Set => Ok(Set),
			JsonLdKeyword::Type => Ok(Type),
			k => Err(k),
		}
	}
}

impl From<JsonLdContainerItem> for JsonLdKeyword {
	fn from(c: JsonLdContainerItem) -> JsonLdKeyword {
		use JsonLdContainerItem::*;
		match c {
			Graph => JsonLdKeyword::Graph,
			Id => JsonLdKeyword::Id,
			Index => JsonLdKeyword::Index,
			Language => JsonLdKeyword::Language,
			List => JsonLdKeyword::List,
			Set => JsonLdKeyword::Set,
			Type => JsonLdKeyword::Type,
		}
	}
}

/// Syntax-level representation of a `@container` value.
///
/// Preserves whether the value was null, a single item, or an array,
/// which matters for JSON-LD 1.0 validation.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(untagged)
)]
pub enum JsonLdContainerValue {
	/// Null container value.
	Null,

	/// A single container keyword, e.g. `"@set"`.
	Item(JsonLdContainerItem),

	/// An array of container keywords, e.g. `["@set"]` or `["@graph", "@id"]`.
	Array(Vec<JsonLdContainerItem>),
}

impl JsonLdContainerValue {
	/// Whether this value was specified as an array in the source JSON.
	pub fn is_array(&self) -> bool {
		matches!(self, Self::Array(_))
	}

	/// Process this syntax value into a [`JsonLdContainer`].
	pub fn to_container(&self) -> Result<JsonLdContainer, UnexpectedJsonLdContainerItem> {
		match self {
			Self::Null => Ok(JsonLdContainer::Null),
			Self::Item(item) => Ok((*item).into()),
			Self::Array(items) => {
				JsonLdContainer::from(items.iter()).map_err(UnexpectedJsonLdContainerItem)
			}
		}
	}
}

/// Error returned when a container array contains an unexpected item
/// combination.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("unexpected `@container` item: `{0}`")]
pub struct UnexpectedJsonLdContainerItem(pub JsonLdContainerItem);

impl From<JsonLdContextTypeContainer> for JsonLdContainerValue {
	fn from(c: JsonLdContextTypeContainer) -> Self {
		match c {
			JsonLdContextTypeContainer::Set => JsonLdContainerValue::Item(JsonLdContainerItem::Set),
		}
	}
}
