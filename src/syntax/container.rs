use std::convert::TryFrom;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ContainerType {
	Graph,
	Id,
	Index,
	Language,
	List,
	Set,
	Type
}

impl<'a> TryFrom<&'a str> for ContainerType {
	type Error = &'a str;

	fn try_from(str: &'a str) -> Result<ContainerType, &'a str> {
		use ContainerType::*;
		match str {
			"@graph" => Ok(Graph),
			"@id" => Ok(Id),
			"@index" => Ok(Index),
			"@language" => Ok(Language),
			"@list" => Ok(List),
			"@set" => Ok(Set),
			"@type" => Ok(Type),
			_ => Err(str)
		}
	}
}

impl From<ContainerType> for Container {
	fn from(c: ContainerType) -> Container {
		use ContainerType::*;
		match c {
			Graph => Container::Graph,
			Id => Container::Id,
			Index => Container::Index,
			Language => Container::Language,
			List => Container::List,
			Set => Container::Set,
			Type => Container::Type
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
	TypeSet,

	GraphIdSet,
	GraphIndexSet
}

impl Container {
	pub fn new() -> Container {
		Container::None
	}

	pub fn from<'a, I: IntoIterator<Item = &'a ContainerType>>(iter: I) -> Result<Container, ContainerType> {
		let mut container = Container::new();
		for item in iter {
			if !container.add(*item) {
				return Err(*item)
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
			TypeSet => &[ContainerType::Type, ContainerType::Set],
			GraphIdSet => &[ContainerType::Graph, ContainerType::Id, ContainerType::Set],
			GraphIndexSet => &[ContainerType::Graph, ContainerType::Index, ContainerType::Set]
		}
	}

	pub fn iter(&self) -> impl Iterator<Item = &ContainerType> {
		self.as_slice().iter()
	}

	pub fn len(&self) -> usize {
		self.as_slice().len()
	}

	pub fn is_empty(&self) -> bool {
		match self {
			Container::None => true,
			_ => false
		}
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
			(Container::Language, ContainerType::Language) => *self,
			(Container::Language, ContainerType::Set) => Container::LanguageSet,
			(Container::List, ContainerType::List) => *self,
			(Container::Set, ContainerType::Set) => *self,
			(Container::Set, ContainerType::Graph) => Container::GraphSet,
			(Container::Set, ContainerType::Id) => Container::IdSet,
			(Container::Set, ContainerType::Index) => Container::IndexSet,
			(Container::Set, ContainerType::Language) => Container::LanguageSet,
			(Container::Set, ContainerType::Type) => Container::TypeSet,
			(Container::Type, ContainerType::Type) => *self,
			(Container::Type, ContainerType::Set) => Container::TypeSet,
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
			(Container::GraphIdSet, ContainerType::Graph) => *self,
			(Container::GraphIdSet, ContainerType::Id) => *self,
			(Container::GraphIdSet, ContainerType::Set) => *self,
			(Container::GraphIndexSet, ContainerType::Graph) => *self,
			(Container::GraphIndexSet, ContainerType::Index) => *self,
			(Container::GraphIndexSet, ContainerType::Set) => *self,
			_ => return None
		};

		Some(new_container)
	}

	pub fn add(&mut self, c: ContainerType) -> bool {
		match self.with(c) {
			Some(container) => {
				*self = container;
				true
			},
			None => false
		}
	}
}
