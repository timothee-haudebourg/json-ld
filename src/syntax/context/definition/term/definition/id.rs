use crate::syntax::context::definition::JsonLdKeyOrKeywordRef;
use crate::syntax::{JsonLdCompactIri, JsonLdExpandableRef, JsonLdKeyword};
use rdf_syntax::BlankId;
use rdf_syntax::Iri;
use std::fmt;
use std::hash::Hash;

#[derive(Clone, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum JsonLdTermId {
	Keyword(JsonLdKeyword),
	Term(String),
}

impl JsonLdTermId {
	pub fn as_iri(&self) -> Option<&Iri> {
		match self {
			Self::Term(t) => Iri::new(t).ok(),
			Self::Keyword(_) => None,
		}
	}

	pub fn as_blank_id(&self) -> Option<&BlankId> {
		match self {
			Self::Term(t) => BlankId::new(t).ok(),
			Self::Keyword(_) => None,
		}
	}

	pub fn as_compact_iri(&self) -> Option<&JsonLdCompactIri> {
		match self {
			Self::Term(t) => JsonLdCompactIri::new(t).ok(),
			Self::Keyword(_) => None,
		}
	}

	pub fn as_keyword(&self) -> Option<JsonLdKeyword> {
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
			Self::Keyword(k) => k.to_string(),
		}
	}

	pub fn is_keyword(&self) -> bool {
		matches!(self, Self::Keyword(_))
	}

	pub fn is_keyword_like(&self) -> bool {
		crate::syntax::is_keyword_like(self.as_str())
	}

	pub fn as_id_ref(&self) -> JsonLdIdRef<'_> {
		match self {
			Self::Term(t) => JsonLdIdRef::Term(t),
			Self::Keyword(k) => JsonLdIdRef::Keyword(*k),
		}
	}
}

impl PartialEq for JsonLdTermId {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Term(a), Self::Term(b)) => a == b,
			(Self::Keyword(a), Self::Keyword(b)) => a == b,
			_ => false,
		}
	}
}

impl Eq for JsonLdTermId {}

impl Hash for JsonLdTermId {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

impl From<String> for JsonLdTermId {
	fn from(s: String) -> Self {
		match JsonLdKeyword::try_from(s.as_str()) {
			Ok(k) => Self::Keyword(k),
			Err(_) => Self::Term(s),
		}
	}
}

impl<'a> From<&'a JsonLdTermId> for JsonLdExpandableRef<'a> {
	fn from(i: &'a JsonLdTermId) -> Self {
		match i {
			JsonLdTermId::Term(t) => Self::String(t),
			JsonLdTermId::Keyword(k) => Self::Keyword(*k),
		}
	}
}

impl fmt::Display for JsonLdTermId {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Term(t) => t.fmt(f),
			Self::Keyword(k) => k.fmt(f),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JsonLdIdRef<'a> {
	Term(&'a str),
	Keyword(JsonLdKeyword),
}

impl<'a> JsonLdIdRef<'a> {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Term(t) => t,
			Self::Keyword(k) => k.into_str(),
		}
	}

	pub fn is_keyword(&self) -> bool {
		matches!(self, Self::Keyword(_))
	}

	pub fn is_keyword_like(&self) -> bool {
		crate::syntax::is_keyword_like(self.as_str())
	}
}

impl<'a> From<&'a str> for JsonLdIdRef<'a> {
	fn from(s: &'a str) -> Self {
		match JsonLdKeyword::try_from(s) {
			Ok(k) => Self::Keyword(k),
			Err(_) => Self::Term(s),
		}
	}
}

impl<'a> From<JsonLdIdRef<'a>> for JsonLdExpandableRef<'a> {
	fn from(i: JsonLdIdRef<'a>) -> Self {
		match i {
			JsonLdIdRef::Term(t) => Self::String(t),
			JsonLdIdRef::Keyword(k) => Self::Keyword(k),
		}
	}
}

impl<'a> fmt::Display for JsonLdIdRef<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> From<JsonLdIdRef<'a>> for JsonLdKeyOrKeywordRef<'a> {
	fn from(i: JsonLdIdRef<'a>) -> Self {
		match i {
			JsonLdIdRef::Term(t) => Self::Key(t.into()),
			JsonLdIdRef::Keyword(k) => Self::Keyword(k),
		}
	}
}
