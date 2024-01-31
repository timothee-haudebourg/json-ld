use crate::Keyword;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ContainerKind {
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

impl From<ContainerKind> for Container {
	fn from(c: ContainerKind) -> Self {
		Container::One(c)
	}
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(untagged)
)]
pub enum Container {
	One(ContainerKind),
	Many(Vec<ContainerKind>),
}

impl Container {
	pub fn is_array(&self) -> bool {
		matches!(self, Self::Many(_))
	}

	pub fn sub_fragments(&self) -> SubValues {
		match self {
			Self::One(_) => SubValues::None,
			Self::Many(m) => SubValues::Many(m.iter()),
		}
	}
}

pub enum SubValues<'a> {
	None,
	Many(std::slice::Iter<'a, ContainerKind>),
}

impl<'a> Iterator for SubValues<'a> {
	type Item = &'a ContainerKind;

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

impl<'a> ExactSizeIterator for SubValues<'a> {}

impl<'a> DoubleEndedIterator for SubValues<'a> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Many(m) => m.next_back(),
		}
	}
}
