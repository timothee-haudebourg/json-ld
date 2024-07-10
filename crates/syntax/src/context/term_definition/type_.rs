use crate::{CompactIri, ExpandableRef, Keyword};
use iref::Iri;
use std::hash::Hash;

#[derive(Clone, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Type {
	Keyword(TypeKeyword),
	Term(String),
}

impl Type {
	pub fn as_iri(&self) -> Option<&Iri> {
		match self {
			Self::Term(t) => Iri::new(t).ok(),
			Self::Keyword(_) => None,
		}
	}

	pub fn as_compact_iri(&self) -> Option<&CompactIri> {
		match self {
			Self::Term(t) => CompactIri::new(t).ok(),
			Self::Keyword(_) => None,
		}
	}

	pub fn as_keyword(&self) -> Option<TypeKeyword> {
		match self {
			Self::Keyword(k) => Some(*k),
			Self::Term(_) => None,
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Self::Term(t) => t.as_str(),
			Self::Keyword(k) => k.into_str(),
		}
	}

	pub fn into_string(self) -> String {
		match self {
			Self::Term(t) => t,
			Self::Keyword(k) => k.as_str().to_string(),
		}
	}
}

impl PartialEq for Type {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Term(a), Self::Term(b)) => a == b,
			(Self::Keyword(a), Self::Keyword(b)) => a == b,
			_ => false,
		}
	}
}

impl Eq for Type {}

impl Hash for Type {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

impl From<String> for Type {
	fn from(s: String) -> Self {
		match TypeKeyword::try_from(s.as_str()) {
			Ok(k) => Self::Keyword(k),
			Err(_) => Self::Term(s),
		}
	}
}

/// Subset of keyword acceptable for as value for the `@type` entry
/// of an expanded term definition.
#[derive(Clone, Copy, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TypeKeyword {
	#[cfg_attr(feature = "serde", serde(rename = "@id"))]
	Id,

	#[cfg_attr(feature = "serde", serde(rename = "@json"))]
	Json,

	#[cfg_attr(feature = "serde", serde(rename = "@none"))]
	None,

	#[cfg_attr(feature = "serde", serde(rename = "@vocab"))]
	Vocab,
}

impl PartialEq for TypeKeyword {
	fn eq(&self, other: &Self) -> bool {
		matches!(
			(self, other),
			(Self::Id, Self::Id)
				| (Self::Json, Self::Json)
				| (Self::None, Self::None)
				| (Self::Vocab, Self::Vocab)
		)
	}
}

impl Eq for TypeKeyword {}

impl Hash for TypeKeyword {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.into_str().hash(state)
	}
}

impl TypeKeyword {
	pub fn keyword(&self) -> Keyword {
		self.into_keyword()
	}

	pub fn into_keyword(self) -> Keyword {
		self.into()
	}

	pub fn as_str(&self) -> &'static str {
		self.into_keyword().into_str()
	}

	pub fn into_str(self) -> &'static str {
		self.into_keyword().into_str()
	}
}

pub struct NotATypeKeyword(pub Keyword);

pub enum InvalidTypeKeyword<T> {
	NotAKeyword(T),
	NotATypeKeyword(Keyword),
}

impl<T> From<NotATypeKeyword> for InvalidTypeKeyword<T> {
	fn from(NotATypeKeyword(k): NotATypeKeyword) -> Self {
		Self::NotATypeKeyword(k)
	}
}

impl<T> From<crate::NotAKeyword<T>> for InvalidTypeKeyword<T> {
	fn from(crate::NotAKeyword(t): crate::NotAKeyword<T>) -> Self {
		Self::NotAKeyword(t)
	}
}

impl From<TypeKeyword> for Keyword {
	fn from(k: TypeKeyword) -> Self {
		match k {
			TypeKeyword::Id => Self::Id,
			TypeKeyword::Json => Self::Json,
			TypeKeyword::None => Self::None,
			TypeKeyword::Vocab => Self::Vocab,
		}
	}
}

impl TryFrom<Keyword> for TypeKeyword {
	type Error = NotATypeKeyword;

	fn try_from(k: Keyword) -> Result<Self, Self::Error> {
		match k {
			Keyword::Id => Ok(Self::Id),
			Keyword::Json => Ok(Self::Json),
			Keyword::None => Ok(Self::None),
			Keyword::Vocab => Ok(Self::Vocab),
			_ => Err(NotATypeKeyword(k)),
		}
	}
}

impl<'a> TryFrom<&'a str> for TypeKeyword {
	type Error = InvalidTypeKeyword<&'a str>;

	fn try_from(s: &'a str) -> Result<Self, Self::Error> {
		Ok(Self::try_from(Keyword::try_from(s)?)?)
	}
}

// impl<'a> From<TypeRef<'a>> for context::definition::KeyOrKeywordRef<'a> {
// 	fn from(d: TypeRef<'a>) -> Self {
// 		match d {
// 			TypeRef::Term(t) => Self::Key(t.into()),
// 			TypeRef::Keyword(k) => Self::Keyword(k.into()),
// 		}
// 	}
// }

impl<'a> From<&'a Type> for ExpandableRef<'a> {
	fn from(d: &'a Type) -> Self {
		match d {
			Type::Term(t) => Self::String(t),
			Type::Keyword(k) => Self::Keyword((*k).into()),
		}
	}
}
