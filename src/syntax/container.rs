use std::str::FromStr;

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

// impl From<ContainerItem> for Container {
// 	fn from(c: ContainerItem) -> Self {
// 		Container::One(c)
// 	}
// }

// #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
// #[cfg_attr(
// 	feature = "serde",
// 	derive(serde::Serialize, serde::Deserialize),
// 	serde(untagged)
// )]
// pub enum Container {
// 	One(ContainerItem),
// 	Many(Vec<ContainerItem>),
// }

// impl Container {
// 	pub fn is_array(&self) -> bool {
// 		matches!(self, Self::Many(_))
// 	}

// 	pub fn sub_fragments(&self) -> SubValues {
// 		match self {
// 			Self::One(_) => SubValues::None,
// 			Self::Many(m) => SubValues::Many(m.iter()),
// 		}
// 	}
// }

// pub enum SubValues<'a> {
// 	None,
// 	Many(std::slice::Iter<'a, ContainerItem>),
// }

// impl<'a> Iterator for SubValues<'a> {
// 	type Item = &'a ContainerItem;

// 	fn size_hint(&self) -> (usize, Option<usize>) {
// 		match self {
// 			Self::None => (0, Some(0)),
// 			Self::Many(m) => m.size_hint(),
// 		}
// 	}

// 	fn next(&mut self) -> Option<Self::Item> {
// 		match self {
// 			Self::None => None,
// 			Self::Many(m) => m.next(),
// 		}
// 	}
// }

// impl<'a> ExactSizeIterator for SubValues<'a> {}

// impl<'a> DoubleEndedIterator for SubValues<'a> {
// 	fn next_back(&mut self) -> Option<Self::Item> {
// 		match self {
// 			Self::None => None,
// 			Self::Many(m) => m.next_back(),
// 		}
// 	}
// }

pub struct InvalidContainer;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Container {
	// Empty container
	Null,

	Graph,
	Id,
	Index,
	Language,
	List,
	Set,
	Type,

	GraphSet,
	GraphId,
	GraphIndex,
	IdSet,
	IndexSet,
	LanguageSet,
	SetType,

	GraphIdSet,
	GraphIndexSet,
}

impl Default for Container {
	fn default() -> Self {
		Self::new()
	}
}

impl Container {
	pub fn new() -> Container {
		Container::Null
	}

	pub fn from<'a, I: IntoIterator<Item = &'a ContainerItem>>(
		iter: I,
	) -> Result<Container, ContainerItem> {
		let mut container = Container::new();
		for item in iter {
			if !container.add(*item) {
				return Err(*item);
			}
		}

		Ok(container)
	}

	pub fn as_slice(&self) -> &[ContainerItem] {
		use Container::*;
		match self {
			Null => &[],
			Graph => &[ContainerItem::Graph],
			Id => &[ContainerItem::Id],
			Index => &[ContainerItem::Index],
			Language => &[ContainerItem::Language],
			List => &[ContainerItem::List],
			Set => &[ContainerItem::Set],
			Type => &[ContainerItem::Type],
			GraphSet => &[ContainerItem::Graph, ContainerItem::Set],
			GraphId => &[ContainerItem::Graph, ContainerItem::Id],
			GraphIndex => &[ContainerItem::Graph, ContainerItem::Index],
			IdSet => &[ContainerItem::Id, ContainerItem::Set],
			IndexSet => &[ContainerItem::Index, ContainerItem::Set],
			LanguageSet => &[ContainerItem::Language, ContainerItem::Set],
			SetType => &[ContainerItem::Type, ContainerItem::Set],
			GraphIdSet => &[ContainerItem::Graph, ContainerItem::Id, ContainerItem::Set],
			GraphIndexSet => &[
				ContainerItem::Graph,
				ContainerItem::Index,
				ContainerItem::Set,
			],
		}
	}

	pub fn iter(&self) -> impl Iterator<Item = &ContainerItem> {
		self.as_slice().iter()
	}

	pub fn len(&self) -> usize {
		self.as_slice().len()
	}

	pub fn is_empty(&self) -> bool {
		matches!(self, Container::Null)
	}

	pub fn contains(&self, c: ContainerItem) -> bool {
		self.as_slice().contains(&c)
	}

	pub fn with(&self, c: ContainerItem) -> Option<Container> {
		let new_container = match (self, c) {
			(Container::Null, c) => c.into(),
			(Container::Graph, ContainerItem::Graph) => *self,
			(Container::Graph, ContainerItem::Set) => Container::GraphSet,
			(Container::Graph, ContainerItem::Id) => Container::GraphId,
			(Container::Graph, ContainerItem::Index) => Container::GraphIndex,
			(Container::Id, ContainerItem::Id) => *self,
			(Container::Id, ContainerItem::Graph) => Container::GraphId,
			(Container::Id, ContainerItem::Set) => Container::IdSet,
			(Container::Index, ContainerItem::Index) => *self,
			(Container::Index, ContainerItem::Graph) => Container::GraphIndex,
			(Container::Index, ContainerItem::Set) => Container::IndexSet,
			(Container::Language, ContainerItem::Language) => *self,
			(Container::Language, ContainerItem::Set) => Container::LanguageSet,
			(Container::List, ContainerItem::List) => *self,
			(Container::Set, ContainerItem::Set) => *self,
			(Container::Set, ContainerItem::Graph) => Container::GraphSet,
			(Container::Set, ContainerItem::Id) => Container::IdSet,
			(Container::Set, ContainerItem::Index) => Container::IndexSet,
			(Container::Set, ContainerItem::Language) => Container::LanguageSet,
			(Container::Set, ContainerItem::Type) => Container::SetType,
			(Container::Type, ContainerItem::Type) => *self,
			(Container::Type, ContainerItem::Set) => Container::SetType,
			(Container::GraphSet, ContainerItem::Graph) => *self,
			(Container::GraphSet, ContainerItem::Set) => *self,
			(Container::GraphSet, ContainerItem::Id) => Container::GraphIdSet,
			(Container::GraphSet, ContainerItem::Index) => Container::GraphIdSet,
			(Container::GraphId, ContainerItem::Graph) => *self,
			(Container::GraphId, ContainerItem::Id) => *self,
			(Container::GraphId, ContainerItem::Set) => Container::GraphIdSet,
			(Container::GraphIndex, ContainerItem::Graph) => *self,
			(Container::GraphIndex, ContainerItem::Index) => *self,
			(Container::GraphIndex, ContainerItem::Set) => Container::GraphIndexSet,
			(Container::IdSet, ContainerItem::Id) => *self,
			(Container::IdSet, ContainerItem::Set) => *self,
			(Container::IdSet, ContainerItem::Graph) => Container::GraphIdSet,
			(Container::IndexSet, ContainerItem::Index) => *self,
			(Container::IndexSet, ContainerItem::Set) => *self,
			(Container::IndexSet, ContainerItem::Graph) => Container::GraphIndexSet,
			(Container::LanguageSet, ContainerItem::Language) => *self,
			(Container::LanguageSet, ContainerItem::Set) => *self,
			(Container::SetType, ContainerItem::Set) => *self,
			(Container::SetType, ContainerItem::Type) => *self,
			(Container::GraphIdSet, ContainerItem::Graph) => *self,
			(Container::GraphIdSet, ContainerItem::Id) => *self,
			(Container::GraphIdSet, ContainerItem::Set) => *self,
			(Container::GraphIndexSet, ContainerItem::Graph) => *self,
			(Container::GraphIndexSet, ContainerItem::Index) => *self,
			(Container::GraphIndexSet, ContainerItem::Set) => *self,
			_ => return None,
		};

		Some(new_container)
	}

	pub fn add(&mut self, c: ContainerItem) -> bool {
		match self.with(c) {
			Some(container) => {
				*self = container;
				true
			}
			None => false,
		}
	}
}

impl From<ContainerItem> for Container {
	fn from(c: ContainerItem) -> Self {
		match c {
			ContainerItem::Graph => Self::Graph,
			ContainerItem::Id => Self::Id,
			ContainerItem::Index => Self::Index,
			ContainerItem::Language => Self::Language,
			ContainerItem::List => Self::List,
			ContainerItem::Set => Self::Set,
			ContainerItem::Type => Self::Type,
		}
	}
}

impl From<ContextTypeContainer> for Container {
	fn from(c: ContextTypeContainer) -> Self {
		match c {
			ContextTypeContainer::Set => Container::Set,
		}
	}
}

impl serde::Serialize for Container {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		match self.as_slice() {
			[] => serializer.serialize_unit(),
			[item] => item.serialize(serializer),
			array => array.serialize(serializer),
		}
	}
}

impl<'de> serde::Deserialize<'de> for Container {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor;

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = Container;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(formatter, "a valid JSON-LD `@container` value")
			}

			fn visit_unit<E>(self) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Ok(Container::Null)
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				v.parse::<ContainerItem>()
					.map(Into::into)
					.map_err(E::custom)
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: serde::de::SeqAccess<'de>,
			{
				let mut result = Container::Null;

				while let Some(item) = seq.next_element::<ContainerItem>()? {
					if !result.add(item) {
						return Err(serde::de::Error::custom(
							"invalid JSON-LD `@container` value",
						));
					}
				}

				Ok(result)
			}
		}

		deserializer.deserialize_any(Visitor)
	}
}
