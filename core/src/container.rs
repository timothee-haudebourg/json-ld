pub use json_ld_syntax::ContainerKind;
use json_ld_syntax::{context::definition::TypeContainer, Nullable};

pub struct InvalidContainer;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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

	pub fn from_syntax(r: Nullable<&json_ld_syntax::Container>) -> Result<Self, InvalidContainer> {
		match r {
			Nullable::Null => Ok(Self::None),
			Nullable::Some(json_ld_syntax::Container::One(c)) => Ok((*c).into()),
			Nullable::Some(json_ld_syntax::Container::Many(m)) => {
				let mut container = Container::new();

				for t in m {
					if !container.add(*t) {
						return Err(InvalidContainer);
					}
				}

				Ok(container)
			}
		}
	}

	pub fn from<'a, I: IntoIterator<Item = &'a ContainerKind>>(
		iter: I,
	) -> Result<Container, ContainerKind> {
		let mut container = Container::new();
		for item in iter {
			if !container.add(*item) {
				return Err(*item);
			}
		}

		Ok(container)
	}

	pub fn as_slice(&self) -> &[ContainerKind] {
		use Container::*;
		match self {
			None => &[],
			Graph => &[ContainerKind::Graph],
			Id => &[ContainerKind::Id],
			Index => &[ContainerKind::Index],
			Language => &[ContainerKind::Language],
			List => &[ContainerKind::List],
			Set => &[ContainerKind::Set],
			Type => &[ContainerKind::Type],
			GraphSet => &[ContainerKind::Graph, ContainerKind::Set],
			GraphId => &[ContainerKind::Graph, ContainerKind::Id],
			GraphIndex => &[ContainerKind::Graph, ContainerKind::Index],
			IdSet => &[ContainerKind::Id, ContainerKind::Set],
			IndexSet => &[ContainerKind::Index, ContainerKind::Set],
			LanguageSet => &[ContainerKind::Language, ContainerKind::Set],
			SetType => &[ContainerKind::Type, ContainerKind::Set],
			GraphIdSet => &[ContainerKind::Graph, ContainerKind::Id, ContainerKind::Set],
			GraphIndexSet => &[
				ContainerKind::Graph,
				ContainerKind::Index,
				ContainerKind::Set,
			],
		}
	}

	pub fn iter(&self) -> impl Iterator<Item = &ContainerKind> {
		self.as_slice().iter()
	}

	pub fn len(&self) -> usize {
		self.as_slice().len()
	}

	pub fn is_empty(&self) -> bool {
		matches!(self, Container::None)
	}

	pub fn contains(&self, c: ContainerKind) -> bool {
		self.as_slice().contains(&c)
	}

	pub fn with(&self, c: ContainerKind) -> Option<Container> {
		let new_container = match (self, c) {
			(Container::None, c) => c.into(),
			(Container::Graph, ContainerKind::Graph) => *self,
			(Container::Graph, ContainerKind::Set) => Container::GraphSet,
			(Container::Graph, ContainerKind::Id) => Container::GraphId,
			(Container::Graph, ContainerKind::Index) => Container::GraphIndex,
			(Container::Id, ContainerKind::Id) => *self,
			(Container::Id, ContainerKind::Graph) => Container::GraphId,
			(Container::Id, ContainerKind::Set) => Container::IdSet,
			(Container::Index, ContainerKind::Index) => *self,
			(Container::Index, ContainerKind::Graph) => Container::GraphIndex,
			(Container::Index, ContainerKind::Set) => Container::IndexSet,
			(Container::Language, ContainerKind::Language) => *self,
			(Container::Language, ContainerKind::Set) => Container::LanguageSet,
			(Container::List, ContainerKind::List) => *self,
			(Container::Set, ContainerKind::Set) => *self,
			(Container::Set, ContainerKind::Graph) => Container::GraphSet,
			(Container::Set, ContainerKind::Id) => Container::IdSet,
			(Container::Set, ContainerKind::Index) => Container::IndexSet,
			(Container::Set, ContainerKind::Language) => Container::LanguageSet,
			(Container::Set, ContainerKind::Type) => Container::SetType,
			(Container::Type, ContainerKind::Type) => *self,
			(Container::Type, ContainerKind::Set) => Container::SetType,
			(Container::GraphSet, ContainerKind::Graph) => *self,
			(Container::GraphSet, ContainerKind::Set) => *self,
			(Container::GraphSet, ContainerKind::Id) => Container::GraphIdSet,
			(Container::GraphSet, ContainerKind::Index) => Container::GraphIdSet,
			(Container::GraphId, ContainerKind::Graph) => *self,
			(Container::GraphId, ContainerKind::Id) => *self,
			(Container::GraphId, ContainerKind::Set) => Container::GraphIdSet,
			(Container::GraphIndex, ContainerKind::Graph) => *self,
			(Container::GraphIndex, ContainerKind::Index) => *self,
			(Container::GraphIndex, ContainerKind::Set) => Container::GraphIndexSet,
			(Container::IdSet, ContainerKind::Id) => *self,
			(Container::IdSet, ContainerKind::Set) => *self,
			(Container::IdSet, ContainerKind::Graph) => Container::GraphIdSet,
			(Container::IndexSet, ContainerKind::Index) => *self,
			(Container::IndexSet, ContainerKind::Set) => *self,
			(Container::IndexSet, ContainerKind::Graph) => Container::GraphIndexSet,
			(Container::LanguageSet, ContainerKind::Language) => *self,
			(Container::LanguageSet, ContainerKind::Set) => *self,
			(Container::SetType, ContainerKind::Set) => *self,
			(Container::SetType, ContainerKind::Type) => *self,
			(Container::GraphIdSet, ContainerKind::Graph) => *self,
			(Container::GraphIdSet, ContainerKind::Id) => *self,
			(Container::GraphIdSet, ContainerKind::Set) => *self,
			(Container::GraphIndexSet, ContainerKind::Graph) => *self,
			(Container::GraphIndexSet, ContainerKind::Index) => *self,
			(Container::GraphIndexSet, ContainerKind::Set) => *self,
			_ => return None,
		};

		Some(new_container)
	}

	pub fn add(&mut self, c: ContainerKind) -> bool {
		match self.with(c) {
			Some(container) => {
				*self = container;
				true
			}
			None => false,
		}
	}

	pub fn into_syntax(self) -> Option<json_ld_syntax::Container> {
		let slice = self.as_slice();

		match slice.len() {
			0 => None,
			1 => Some(json_ld_syntax::Container::One(slice[0])),
			_ => Some(json_ld_syntax::Container::Many(slice.to_vec())),
		}
	}
}

impl From<ContainerKind> for Container {
	fn from(c: ContainerKind) -> Self {
		match c {
			ContainerKind::Graph => Self::Graph,
			ContainerKind::Id => Self::Id,
			ContainerKind::Index => Self::Index,
			ContainerKind::Language => Self::Language,
			ContainerKind::List => Self::List,
			ContainerKind::Set => Self::Set,
			ContainerKind::Type => Self::Type,
		}
	}
}

impl From<TypeContainer> for Container {
	fn from(c: TypeContainer) -> Self {
		match c {
			TypeContainer::Set => Container::Set,
		}
	}
}
