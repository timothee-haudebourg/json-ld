use crate::syntax::{JsonLdCompactIri, JsonLdExpandableRef, JsonLdKeyword};
use rdf_syntax::Iri;
use std::hash::Hash;

/// Term definition type.
#[derive(Clone, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum JsonLdTermType {
	Keyword(JsonLdTypeKeyword),
	Term(String),
}

impl JsonLdTermType {
	pub fn as_iri(&self) -> Option<&Iri> {
		match self {
			Self::Term(t) => Iri::new(t).ok(),
			Self::Keyword(_) => None,
		}
	}

	pub fn as_compact_iri(&self) -> Option<&JsonLdCompactIri> {
		match self {
			Self::Term(t) => JsonLdCompactIri::new(t).ok(),
			Self::Keyword(_) => None,
		}
	}

	pub fn as_keyword(&self) -> Option<JsonLdTypeKeyword> {
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

impl PartialEq for JsonLdTermType {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Term(a), Self::Term(b)) => a == b,
			(Self::Keyword(a), Self::Keyword(b)) => a == b,
			_ => false,
		}
	}
}

impl Eq for JsonLdTermType {}

impl Hash for JsonLdTermType {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

impl From<String> for JsonLdTermType {
	fn from(s: String) -> Self {
		match JsonLdTypeKeyword::try_from(s.as_str()) {
			Ok(k) => Self::Keyword(k),
			Err(_) => Self::Term(s),
		}
	}
}

/// Subset of keyword acceptable for as value for the `@type` entry
/// of an expanded term definition.
#[derive(Clone, Copy, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JsonLdTypeKeyword {
	#[cfg_attr(feature = "serde", serde(rename = "@id"))]
	Id,

	#[cfg_attr(feature = "serde", serde(rename = "@json"))]
	Json,

	#[cfg_attr(feature = "serde", serde(rename = "@none"))]
	None,

	#[cfg_attr(feature = "serde", serde(rename = "@vocab"))]
	Vocab,
}

impl PartialEq for JsonLdTypeKeyword {
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

impl Eq for JsonLdTypeKeyword {}

impl Hash for JsonLdTypeKeyword {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.into_str().hash(state)
	}
}

impl JsonLdTypeKeyword {
	pub fn keyword(&self) -> JsonLdKeyword {
		self.into_keyword()
	}

	pub fn into_keyword(self) -> JsonLdKeyword {
		self.into()
	}

	pub fn as_str(&self) -> &'static str {
		self.into_keyword().into_str()
	}

	pub fn into_str(self) -> &'static str {
		self.into_keyword().into_str()
	}
}

pub struct NotAJsonLdTypeKeyword(pub JsonLdKeyword);

pub enum InvalidJsonLdTypeKeyword<T> {
	NotAKeyword(T),
	NotATypeKeyword(JsonLdKeyword),
}

impl<T> From<NotAJsonLdTypeKeyword> for InvalidJsonLdTypeKeyword<T> {
	fn from(NotAJsonLdTypeKeyword(k): NotAJsonLdTypeKeyword) -> Self {
		Self::NotATypeKeyword(k)
	}
}

impl<T> From<crate::syntax::NotAJsonLdKeyword<T>> for InvalidJsonLdTypeKeyword<T> {
	fn from(crate::syntax::NotAJsonLdKeyword(t): crate::syntax::NotAJsonLdKeyword<T>) -> Self {
		Self::NotAKeyword(t)
	}
}

impl From<JsonLdTypeKeyword> for JsonLdKeyword {
	fn from(k: JsonLdTypeKeyword) -> Self {
		match k {
			JsonLdTypeKeyword::Id => Self::Id,
			JsonLdTypeKeyword::Json => Self::Json,
			JsonLdTypeKeyword::None => Self::None,
			JsonLdTypeKeyword::Vocab => Self::Vocab,
		}
	}
}

impl TryFrom<JsonLdKeyword> for JsonLdTypeKeyword {
	type Error = NotAJsonLdTypeKeyword;

	fn try_from(k: JsonLdKeyword) -> Result<Self, Self::Error> {
		match k {
			JsonLdKeyword::Id => Ok(Self::Id),
			JsonLdKeyword::Json => Ok(Self::Json),
			JsonLdKeyword::None => Ok(Self::None),
			JsonLdKeyword::Vocab => Ok(Self::Vocab),
			_ => Err(NotAJsonLdTypeKeyword(k)),
		}
	}
}

impl<'a> TryFrom<&'a str> for JsonLdTypeKeyword {
	type Error = InvalidJsonLdTypeKeyword<&'a str>;

	fn try_from(s: &'a str) -> Result<Self, Self::Error> {
		Ok(Self::try_from(JsonLdKeyword::try_from(s)?)?)
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

impl<'a> From<&'a JsonLdTermType> for JsonLdExpandableRef<'a> {
	fn from(d: &'a JsonLdTermType) -> Self {
		match d {
			JsonLdTermType::Term(t) => Self::String(t),
			JsonLdTermType::Keyword(k) => Self::Keyword((*k).into()),
		}
	}
}
