use crate::syntax::JsonLdContainerItem;
use crate::syntax::context::JsonLdContextTypeContainer;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum JsonLdContainer {
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

impl Default for JsonLdContainer {
	fn default() -> Self {
		Self::new()
	}
}

impl JsonLdContainer {
	pub fn new() -> JsonLdContainer {
		JsonLdContainer::Null
	}

	pub fn from<'a, I: IntoIterator<Item = &'a JsonLdContainerItem>>(
		iter: I,
	) -> Result<JsonLdContainer, JsonLdContainerItem> {
		let mut container = JsonLdContainer::new();
		for item in iter {
			if !container.add(*item) {
				return Err(*item);
			}
		}

		Ok(container)
	}

	pub fn as_slice(&self) -> &[JsonLdContainerItem] {
		use JsonLdContainer::*;
		match self {
			Null => &[],
			Graph => &[JsonLdContainerItem::Graph],
			Id => &[JsonLdContainerItem::Id],
			Index => &[JsonLdContainerItem::Index],
			Language => &[JsonLdContainerItem::Language],
			List => &[JsonLdContainerItem::List],
			Set => &[JsonLdContainerItem::Set],
			Type => &[JsonLdContainerItem::Type],
			GraphSet => &[JsonLdContainerItem::Graph, JsonLdContainerItem::Set],
			GraphId => &[JsonLdContainerItem::Graph, JsonLdContainerItem::Id],
			GraphIndex => &[JsonLdContainerItem::Graph, JsonLdContainerItem::Index],
			IdSet => &[JsonLdContainerItem::Id, JsonLdContainerItem::Set],
			IndexSet => &[JsonLdContainerItem::Index, JsonLdContainerItem::Set],
			LanguageSet => &[JsonLdContainerItem::Language, JsonLdContainerItem::Set],
			SetType => &[JsonLdContainerItem::Type, JsonLdContainerItem::Set],
			GraphIdSet => &[
				JsonLdContainerItem::Graph,
				JsonLdContainerItem::Id,
				JsonLdContainerItem::Set,
			],
			GraphIndexSet => &[
				JsonLdContainerItem::Graph,
				JsonLdContainerItem::Index,
				JsonLdContainerItem::Set,
			],
		}
	}

	pub fn iter(&self) -> impl Iterator<Item = &JsonLdContainerItem> {
		self.as_slice().iter()
	}

	pub fn len(&self) -> usize {
		self.as_slice().len()
	}

	pub fn is_empty(&self) -> bool {
		matches!(self, JsonLdContainer::Null)
	}

	pub fn contains(&self, c: JsonLdContainerItem) -> bool {
		self.as_slice().contains(&c)
	}

	pub fn with(&self, c: JsonLdContainerItem) -> Option<JsonLdContainer> {
		let new_container = match (self, c) {
			(JsonLdContainer::Null, c) => c.into(),
			(JsonLdContainer::Graph, JsonLdContainerItem::Graph) => *self,
			(JsonLdContainer::Graph, JsonLdContainerItem::Set) => JsonLdContainer::GraphSet,
			(JsonLdContainer::Graph, JsonLdContainerItem::Id) => JsonLdContainer::GraphId,
			(JsonLdContainer::Graph, JsonLdContainerItem::Index) => JsonLdContainer::GraphIndex,
			(JsonLdContainer::Id, JsonLdContainerItem::Id) => *self,
			(JsonLdContainer::Id, JsonLdContainerItem::Graph) => JsonLdContainer::GraphId,
			(JsonLdContainer::Id, JsonLdContainerItem::Set) => JsonLdContainer::IdSet,
			(JsonLdContainer::Index, JsonLdContainerItem::Index) => *self,
			(JsonLdContainer::Index, JsonLdContainerItem::Graph) => JsonLdContainer::GraphIndex,
			(JsonLdContainer::Index, JsonLdContainerItem::Set) => JsonLdContainer::IndexSet,
			(JsonLdContainer::Language, JsonLdContainerItem::Language) => *self,
			(JsonLdContainer::Language, JsonLdContainerItem::Set) => JsonLdContainer::LanguageSet,
			(JsonLdContainer::List, JsonLdContainerItem::List) => *self,
			(JsonLdContainer::Set, JsonLdContainerItem::Set) => *self,
			(JsonLdContainer::Set, JsonLdContainerItem::Graph) => JsonLdContainer::GraphSet,
			(JsonLdContainer::Set, JsonLdContainerItem::Id) => JsonLdContainer::IdSet,
			(JsonLdContainer::Set, JsonLdContainerItem::Index) => JsonLdContainer::IndexSet,
			(JsonLdContainer::Set, JsonLdContainerItem::Language) => JsonLdContainer::LanguageSet,
			(JsonLdContainer::Set, JsonLdContainerItem::Type) => JsonLdContainer::SetType,
			(JsonLdContainer::Type, JsonLdContainerItem::Type) => *self,
			(JsonLdContainer::Type, JsonLdContainerItem::Set) => JsonLdContainer::SetType,
			(JsonLdContainer::GraphSet, JsonLdContainerItem::Graph) => *self,
			(JsonLdContainer::GraphSet, JsonLdContainerItem::Set) => *self,
			(JsonLdContainer::GraphSet, JsonLdContainerItem::Id) => JsonLdContainer::GraphIdSet,
			(JsonLdContainer::GraphSet, JsonLdContainerItem::Index) => JsonLdContainer::GraphIdSet,
			(JsonLdContainer::GraphId, JsonLdContainerItem::Graph) => *self,
			(JsonLdContainer::GraphId, JsonLdContainerItem::Id) => *self,
			(JsonLdContainer::GraphId, JsonLdContainerItem::Set) => JsonLdContainer::GraphIdSet,
			(JsonLdContainer::GraphIndex, JsonLdContainerItem::Graph) => *self,
			(JsonLdContainer::GraphIndex, JsonLdContainerItem::Index) => *self,
			(JsonLdContainer::GraphIndex, JsonLdContainerItem::Set) => {
				JsonLdContainer::GraphIndexSet
			}
			(JsonLdContainer::IdSet, JsonLdContainerItem::Id) => *self,
			(JsonLdContainer::IdSet, JsonLdContainerItem::Set) => *self,
			(JsonLdContainer::IdSet, JsonLdContainerItem::Graph) => JsonLdContainer::GraphIdSet,
			(JsonLdContainer::IndexSet, JsonLdContainerItem::Index) => *self,
			(JsonLdContainer::IndexSet, JsonLdContainerItem::Set) => *self,
			(JsonLdContainer::IndexSet, JsonLdContainerItem::Graph) => {
				JsonLdContainer::GraphIndexSet
			}
			(JsonLdContainer::LanguageSet, JsonLdContainerItem::Language) => *self,
			(JsonLdContainer::LanguageSet, JsonLdContainerItem::Set) => *self,
			(JsonLdContainer::SetType, JsonLdContainerItem::Set) => *self,
			(JsonLdContainer::SetType, JsonLdContainerItem::Type) => *self,
			(JsonLdContainer::GraphIdSet, JsonLdContainerItem::Graph) => *self,
			(JsonLdContainer::GraphIdSet, JsonLdContainerItem::Id) => *self,
			(JsonLdContainer::GraphIdSet, JsonLdContainerItem::Set) => *self,
			(JsonLdContainer::GraphIndexSet, JsonLdContainerItem::Graph) => *self,
			(JsonLdContainer::GraphIndexSet, JsonLdContainerItem::Index) => *self,
			(JsonLdContainer::GraphIndexSet, JsonLdContainerItem::Set) => *self,
			_ => return None,
		};

		Some(new_container)
	}

	pub fn add(&mut self, c: JsonLdContainerItem) -> bool {
		match self.with(c) {
			Some(container) => {
				*self = container;
				true
			}
			None => false,
		}
	}
}

impl From<JsonLdContainerItem> for JsonLdContainer {
	fn from(c: JsonLdContainerItem) -> Self {
		match c {
			JsonLdContainerItem::Graph => Self::Graph,
			JsonLdContainerItem::Id => Self::Id,
			JsonLdContainerItem::Index => Self::Index,
			JsonLdContainerItem::Language => Self::Language,
			JsonLdContainerItem::List => Self::List,
			JsonLdContainerItem::Set => Self::Set,
			JsonLdContainerItem::Type => Self::Type,
		}
	}
}

impl From<JsonLdContextTypeContainer> for JsonLdContainer {
	fn from(c: JsonLdContextTypeContainer) -> Self {
		match c {
			JsonLdContextTypeContainer::Set => Self::Set,
		}
	}
}
