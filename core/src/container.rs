use json_ld_syntax::{ContainerType, Nullable};
use locspan::Meta;
use locspan_derive::StrippedPartialEq;

pub struct InvalidContainer;

#[derive(Clone, Copy, PartialEq, StrippedPartialEq, Eq, Hash, Debug)]
pub enum Container {
	// Empty container
	None,

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
		Container::None
	}

	pub fn from_syntax_ref<M>(
		r: Nullable<json_ld_syntax::ContainerRef<M>>,
	) -> Result<Self, Meta<InvalidContainer, M>>
	where
		M: Clone,
	{
		match r {
			Nullable::Null => Ok(Self::None),
			Nullable::Some(json_ld_syntax::ContainerRef::One(c)) => Ok(c.into()),
			Nullable::Some(json_ld_syntax::ContainerRef::Many(m)) => {
				let mut container = Container::new();

				for Meta(t, t_meta) in m {
					if !container.add(*t) {
						return Err(Meta(InvalidContainer, t_meta.clone()));
					}
				}

				Ok(container)
			}
		}
	}

	pub fn from<'a, I: IntoIterator<Item = &'a ContainerType>>(
		iter: I,
	) -> Result<Container, ContainerType> {
		let mut container = Container::new();
		for item in iter {
			if !container.add(*item) {
				return Err(*item);
			}
		}

		Ok(container)
	}

	pub fn as_slice(&self) -> &[ContainerType] {
		use Container::*;
		match self {
			None => &[],
			Graph => &[ContainerType::Graph],
			Id => &[ContainerType::Id],
			Index => &[ContainerType::Index],
			Language => &[ContainerType::Language],
			List => &[ContainerType::List],
			Set => &[ContainerType::Set],
			Type => &[ContainerType::Type],
			GraphSet => &[ContainerType::Graph, ContainerType::Set],
			GraphId => &[ContainerType::Graph, ContainerType::Id],
			GraphIndex => &[ContainerType::Graph, ContainerType::Index],
			IdSet => &[ContainerType::Id, ContainerType::Set],
			IndexSet => &[ContainerType::Index, ContainerType::Set],
			LanguageSet => &[ContainerType::Language, ContainerType::Set],
			SetType => &[ContainerType::Type, ContainerType::Set],
			GraphIdSet => &[ContainerType::Graph, ContainerType::Id, ContainerType::Set],
			GraphIndexSet => &[
				ContainerType::Graph,
				ContainerType::Index,
				ContainerType::Set,
			],
		}
	}

	pub fn iter(&self) -> impl Iterator<Item = &ContainerType> {
		self.as_slice().iter()
	}

	pub fn len(&self) -> usize {
		self.as_slice().len()
	}

	pub fn is_empty(&self) -> bool {
		matches!(self, Container::None)
	}

	pub fn contains(&self, c: ContainerType) -> bool {
		self.as_slice().contains(&c)
	}

	pub fn with(&self, c: ContainerType) -> Option<Container> {
		let new_container = match (self, c) {
			(Container::None, c) => c.into(),
			(Container::Graph, ContainerType::Graph) => *self,
			(Container::Graph, ContainerType::Set) => Container::GraphSet,
			(Container::Graph, ContainerType::Id) => Container::GraphId,
			(Container::Graph, ContainerType::Index) => Container::GraphIndex,
			(Container::Id, ContainerType::Id) => *self,
			(Container::Id, ContainerType::Graph) => Container::GraphId,
			(Container::Id, ContainerType::Set) => Container::IdSet,
			(Container::Index, ContainerType::Index) => *self,
			(Container::Index, ContainerType::Graph) => Container::GraphIndex,
			(Container::Index, ContainerType::Set) => Container::IndexSet,
			(Container::Language, ContainerType::Language) => *self,
			(Container::Language, ContainerType::Set) => Container::LanguageSet,
			(Container::List, ContainerType::List) => *self,
			(Container::Set, ContainerType::Set) => *self,
			(Container::Set, ContainerType::Graph) => Container::GraphSet,
			(Container::Set, ContainerType::Id) => Container::IdSet,
			(Container::Set, ContainerType::Index) => Container::IndexSet,
			(Container::Set, ContainerType::Language) => Container::LanguageSet,
			(Container::Set, ContainerType::Type) => Container::SetType,
			(Container::Type, ContainerType::Type) => *self,
			(Container::Type, ContainerType::Set) => Container::SetType,
			(Container::GraphSet, ContainerType::Graph) => *self,
			(Container::GraphSet, ContainerType::Set) => *self,
			(Container::GraphSet, ContainerType::Id) => Container::GraphIdSet,
			(Container::GraphSet, ContainerType::Index) => Container::GraphIdSet,
			(Container::GraphId, ContainerType::Graph) => *self,
			(Container::GraphId, ContainerType::Id) => *self,
			(Container::GraphId, ContainerType::Set) => Container::GraphIdSet,
			(Container::GraphIndex, ContainerType::Graph) => *self,
			(Container::GraphIndex, ContainerType::Index) => *self,
			(Container::GraphIndex, ContainerType::Set) => Container::GraphIndexSet,
			(Container::IdSet, ContainerType::Id) => *self,
			(Container::IdSet, ContainerType::Set) => *self,
			(Container::IdSet, ContainerType::Graph) => Container::GraphIdSet,
			(Container::IndexSet, ContainerType::Index) => *self,
			(Container::IndexSet, ContainerType::Set) => *self,
			(Container::IndexSet, ContainerType::Graph) => Container::GraphIndexSet,
			(Container::LanguageSet, ContainerType::Language) => *self,
			(Container::LanguageSet, ContainerType::Set) => *self,
			(Container::SetType, ContainerType::Set) => *self,
			(Container::SetType, ContainerType::Type) => *self,
			(Container::GraphIdSet, ContainerType::Graph) => *self,
			(Container::GraphIdSet, ContainerType::Id) => *self,
			(Container::GraphIdSet, ContainerType::Set) => *self,
			(Container::GraphIndexSet, ContainerType::Graph) => *self,
			(Container::GraphIndexSet, ContainerType::Index) => *self,
			(Container::GraphIndexSet, ContainerType::Set) => *self,
			_ => return None,
		};

		Some(new_container)
	}

	pub fn add(&mut self, c: ContainerType) -> bool {
		match self.with(c) {
			Some(container) => {
				*self = container;
				true
			}
			None => false,
		}
	}
}

impl From<ContainerType> for Container {
	fn from(c: ContainerType) -> Self {
		match c {
			ContainerType::Graph => Self::Graph,
			ContainerType::Id => Self::Id,
			ContainerType::Index => Self::Index,
			ContainerType::Language => Self::Language,
			ContainerType::List => Self::List,
			ContainerType::Set => Self::Set,
			ContainerType::Type => Self::Type,
		}
	}
}
