use crate::Keyword;
use locspan::Meta;
use locspan_derive::*;

#[derive(
	Clone,
	Copy,
	PartialEq,
	StrippedPartialEq,
	Eq,
	StrippedEq,
	PartialOrd,
	StrippedPartialOrd,
	Ord,
	StrippedOrd,
	Hash,
	Debug,
)]
pub enum ContainerType {
	Graph,
	Id,
	Index,
	Language,
	List,
	Set,
	Type,
}

impl ContainerType {
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
			_ => Err(str),
		}
	}
}

impl TryFrom<Keyword> for ContainerType {
	type Error = Keyword;

	fn try_from(k: Keyword) -> Result<ContainerType, Keyword> {
		use ContainerType::*;
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

impl From<ContainerType> for Keyword {
	fn from(c: ContainerType) -> Keyword {
		use ContainerType::*;
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

impl<M> From<ContainerType> for Container<M> {
	fn from(c: ContainerType) -> Self {
		Container::One(c)
	}
}

#[derive(
	Clone,
	PartialEq,
	StrippedPartialEq,
	Eq,
	StrippedEq,
	PartialOrd,
	StrippedPartialOrd,
	Ord,
	StrippedOrd,
	Hash,
	Debug,
)]
#[stripped_ignore(M)]
pub enum Container<M> {
	One(ContainerType),
	Many(Vec<Meta<ContainerType, M>>),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ContainerRef<'a, M> {
	One(ContainerType),
	Many(&'a [Meta<ContainerType, M>]),
}

impl<'a, M> ContainerRef<'a, M> {
	pub fn is_array(&self) -> bool {
		matches!(self, Self::Many(_))
	}
}

impl<'a, M> From<&'a Container<M>> for ContainerRef<'a, M> {
	fn from(c: &'a Container<M>) -> Self {
		match c {
			Container::One(c) => Self::One(*c),
			Container::Many(m) => Self::Many(m),
		}
	}
}
