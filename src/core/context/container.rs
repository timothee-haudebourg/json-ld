use crate::syntax::context::ContextTypeContainer;
use crate::syntax::ContainerItem;

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
			ContextTypeContainer::Set => Self::Set,
		}
	}
}
