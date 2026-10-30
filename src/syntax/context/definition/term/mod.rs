use crate::syntax::{JsonLdCompactIri, JsonLdKeyword};
use rdf_syntax::BlankId;
use rdf_syntax::Iri;
use std::borrow::Borrow;
use std::fmt;
use std::hash::Hash;

mod definition;

pub use definition::*;

/// Context term.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct JsonLdContextTerm(String);

impl JsonLdContextTerm {
	pub fn as_iri(&self) -> Option<&Iri> {
		Iri::new(&self.0).ok()
	}

	pub fn as_compact_iri(&self) -> Option<&JsonLdCompactIri> {
		JsonLdCompactIri::new(&self.0).ok()
	}

	pub fn as_blank_id(&self) -> Option<&BlankId> {
		BlankId::new(&self.0).ok()
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn into_string(self) -> String {
		self.0
	}

	pub fn is_keyword_like(&self) -> bool {
		crate::syntax::is_keyword_like(self.as_str())
	}
}

impl From<json_syntax::object::Key> for JsonLdContextTerm {
	fn from(k: json_syntax::object::Key) -> Self {
		Self::from(k.into_string())
	}
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for JsonLdContextTerm {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

impl From<String> for JsonLdContextTerm {
	fn from(k: String) -> Self {
		Self(k)
	}
}

impl<'a> From<&'a str> for JsonLdContextTerm {
	fn from(value: &'a str) -> Self {
		Self(value.to_owned())
	}
}

impl fmt::Display for JsonLdContextTerm {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl Borrow<str> for JsonLdContextTerm {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KeyRef<'a>(&'a str);

impl<'a> KeyRef<'a> {
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn is_keyword_like(&self) -> bool {
		crate::syntax::is_keyword_like(self.as_str())
	}

	pub fn as_str(&self) -> &'a str {
		self.0
	}

	pub fn to_owned(self) -> JsonLdContextTerm {
		JsonLdContextTerm(self.0.to_owned())
	}
}

impl<'a> From<&'a str> for KeyRef<'a> {
	fn from(s: &'a str) -> Self {
		Self(s)
	}
}

impl<'a> From<&'a JsonLdContextTerm> for KeyRef<'a> {
	fn from(k: &'a JsonLdContextTerm) -> Self {
		Self(&k.0)
	}
}

impl<'a> fmt::Display for KeyRef<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum JsonLdKeyOrKeyword {
	Keyword(JsonLdKeyword),
	Key(JsonLdContextTerm),
}

impl JsonLdKeyOrKeyword {
	pub fn is_empty(&self) -> bool {
		match self {
			Self::Keyword(_) => false,
			Self::Key(k) => k.is_empty(),
		}
	}

	pub fn into_keyword(self) -> Option<JsonLdKeyword> {
		match self {
			Self::Keyword(k) => Some(k),
			Self::Key(_) => None,
		}
	}

	pub fn into_key(self) -> Option<JsonLdContextTerm> {
		match self {
			Self::Keyword(_) => None,
			Self::Key(k) => Some(k),
		}
	}

	pub fn as_keyword(&self) -> Option<JsonLdKeyword> {
		match self {
			Self::Keyword(k) => Some(*k),
			Self::Key(_) => None,
		}
	}

	pub fn as_key(&self) -> Option<&JsonLdContextTerm> {
		match self {
			Self::Keyword(_) => None,
			Self::Key(k) => Some(k),
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Self::Keyword(k) => k.into_str(),
			Self::Key(k) => k.as_str(),
		}
	}
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for JsonLdKeyOrKeyword {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

impl fmt::Display for JsonLdKeyOrKeyword {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Key(k) => k.fmt(f),
			Self::Keyword(k) => k.fmt(f),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum JsonLdKeyOrKeywordRef<'a> {
	Keyword(JsonLdKeyword),
	Key(KeyRef<'a>),
}

impl<'a> JsonLdKeyOrKeywordRef<'a> {
	pub fn to_owned(self) -> JsonLdKeyOrKeyword {
		match self {
			Self::Keyword(k) => JsonLdKeyOrKeyword::Keyword(k),
			Self::Key(k) => JsonLdKeyOrKeyword::Key(k.to_owned()),
		}
	}

	pub fn as_str(&self) -> &'a str {
		match self {
			Self::Keyword(k) => k.into_str(),
			Self::Key(k) => k.as_str(),
		}
	}
}

impl<'a> From<&'a str> for JsonLdKeyOrKeywordRef<'a> {
	fn from(s: &'a str) -> Self {
		match JsonLdKeyword::try_from(s) {
			Ok(k) => Self::Keyword(k),
			Err(_) => Self::Key(s.into()),
		}
	}
}

impl<'a> From<&'a JsonLdKeyOrKeyword> for JsonLdKeyOrKeywordRef<'a> {
	fn from(k: &'a JsonLdKeyOrKeyword) -> Self {
		match k {
			JsonLdKeyOrKeyword::Keyword(k) => Self::Keyword(*k),
			JsonLdKeyOrKeyword::Key(k) => Self::Key(k.into()),
		}
	}
}

impl<'a> From<KeyRef<'a>> for JsonLdKeyOrKeywordRef<'a> {
	fn from(k: KeyRef<'a>) -> Self {
		Self::Key(k)
	}
}

impl<'a> From<&'a JsonLdContextTerm> for JsonLdKeyOrKeywordRef<'a> {
	fn from(k: &'a JsonLdContextTerm) -> Self {
		Self::Key(k.into())
	}
}

pub enum KeyOrType {
	Key(JsonLdContextTerm),
	Type,
}

impl KeyOrType {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Key(k) => k.as_str(),
			Self::Type => "@type",
		}
	}
}

impl fmt::Display for KeyOrType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}
