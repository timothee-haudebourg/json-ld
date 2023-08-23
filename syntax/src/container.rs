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
pub enum ContainerKind {
	Graph,
	Id,
	Index,
	Language,
	List,
	Set,
	Type,
}

impl ContainerKind {
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

impl<'a> TryFrom<&'a str> for ContainerKind {
	type Error = &'a str;

	fn try_from(str: &'a str) -> Result<ContainerKind, &'a str> {
		use ContainerKind::*;
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

impl TryFrom<Keyword> for ContainerKind {
	type Error = Keyword;

	fn try_from(k: Keyword) -> Result<ContainerKind, Keyword> {
		use ContainerKind::*;
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

impl From<ContainerKind> for Keyword {
	fn from(c: ContainerKind) -> Keyword {
		use ContainerKind::*;
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

impl<M> From<ContainerKind> for Container<M> {
	fn from(c: ContainerKind) -> Self {
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
#[locspan(ignore(M))]
pub enum Container<M> {
	One(ContainerKind),
	Many(Vec<Meta<ContainerKind, M>>),
}

impl<M> Container<M> {
	pub fn is_array(&self) -> bool {
		matches!(self, Self::Many(_))
	}

	pub fn sub_fragments(&self) -> SubValues<M> {
		match self {
			Self::One(_) => SubValues::None,
			Self::Many(m) => SubValues::Many(m.iter()),
		}
	}
}

pub enum SubValues<'a, M> {
	None,
	Many(std::slice::Iter<'a, Meta<ContainerKind, M>>),
}

impl<'a, M> Iterator for SubValues<'a, M> {
	type Item = &'a Meta<ContainerKind, M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		match self {
			Self::None => (0, Some(0)),
			Self::Many(m) => m.size_hint(),
		}
	}

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Many(m) => m.next(),
		}
	}
}

impl<'a, M> ExactSizeIterator for SubValues<'a, M> {}

impl<'a, M> DoubleEndedIterator for SubValues<'a, M> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Many(m) => m.next_back(),
		}
	}
}
