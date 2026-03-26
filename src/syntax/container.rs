use std::str::FromStr;

use crate::context::Container;
use crate::syntax::context::ContextTypeContainer;

use super::Keyword;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ContainerItem {
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

impl ContainerItem {
	pub fn into_keyword(self) -> Keyword {
		self.into()
	}

	pub fn keyword(&self) -> Keyword {
		self.into_keyword()
	}

	pub fn as_str(&self) -> &'static str {
		self.into_keyword().into_str()
	}
}

impl std::fmt::Display for ContainerItem {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.as_str())
	}
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Invalid `@container` item: `{0}`")]
pub struct InvalidContainerItem<T = String>(pub T);

impl<T: ?Sized + ToOwned> InvalidContainerItem<&T> {
	pub fn into_owned(self) -> InvalidContainerItem<T::Owned> {
		InvalidContainerItem(self.0.to_owned())
	}
}

impl FromStr for ContainerItem {
	type Err = InvalidContainerItem;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		s.try_into().map_err(InvalidContainerItem::into_owned)
	}
}

impl<'a> TryFrom<&'a str> for ContainerItem {
	type Error = InvalidContainerItem<&'a str>;

	fn try_from(str: &'a str) -> Result<ContainerItem, InvalidContainerItem<&'a str>> {
		use ContainerItem::*;
		match str {
			"@graph" => Ok(Graph),
			"@id" => Ok(Id),
			"@index" => Ok(Index),
			"@language" => Ok(Language),
			"@list" => Ok(List),
			"@set" => Ok(Set),
			"@type" => Ok(Type),
			_ => Err(InvalidContainerItem(str)),
		}
	}
}

impl TryFrom<Keyword> for ContainerItem {
	type Error = Keyword;

	fn try_from(k: Keyword) -> Result<ContainerItem, Keyword> {
		use ContainerItem::*;
		match k {
			Keyword::Graph => Ok(Graph),
			Keyword::Id => Ok(Id),
			Keyword::Index => Ok(Index),
			Keyword::Language => Ok(Language),
			Keyword::List => Ok(List),
			Keyword::Set => Ok(Set),
			Keyword::Type => Ok(Type),
			k => Err(k),
		}
	}
}

impl From<ContainerItem> for Keyword {
	fn from(c: ContainerItem) -> Keyword {
		use ContainerItem::*;
		match c {
			Graph => Keyword::Graph,
			Id => Keyword::Id,
			Index => Keyword::Index,
			Language => Keyword::Language,
			List => Keyword::List,
			Set => Keyword::Set,
			Type => Keyword::Type,
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
pub enum ContainerValue {
	/// Null container value.
	Null,

	/// A single container keyword, e.g. `"@set"`.
	Item(ContainerItem),

	/// An array of container keywords, e.g. `["@set"]` or `["@graph", "@id"]`.
	Array(Vec<ContainerItem>),
}

impl ContainerValue {
	/// Whether this value was specified as an array in the source JSON.
	pub fn is_array(&self) -> bool {
		matches!(self, Self::Array(_))
	}

	/// Process this syntax value into a [`Container`].
	pub fn to_container(&self) -> Result<Container, UnexpectedContainerItem> {
		match self {
			Self::Null => Ok(Container::Null),
			Self::Item(item) => Ok((*item).into()),
			Self::Array(items) => Container::from(items.iter()).map_err(UnexpectedContainerItem),
		}
	}
}

/// Error returned when a container array contains an unexpected item
/// combination.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("unexpected `@container` item: `{0}`")]
pub struct UnexpectedContainerItem(pub ContainerItem);

impl From<ContextTypeContainer> for ContainerValue {
	fn from(c: ContextTypeContainer) -> Self {
		match c {
			ContextTypeContainer::Set => ContainerValue::Item(ContainerItem::Set),
		}
	}
}
