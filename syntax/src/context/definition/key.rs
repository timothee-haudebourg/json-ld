use crate::{CompactIri, Keyword};
use iref::Iri;
use rdf_types::BlankId;
use std::borrow::Borrow;
use std::fmt;
use std::hash::Hash;

/// Context key.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Key(String);

impl Key {
	pub fn as_iri(&self) -> Option<&Iri> {
		Iri::new(&self.0).ok()
	}

	pub fn as_compact_iri(&self) -> Option<&CompactIri> {
		CompactIri::new(&self.0).ok()
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
		crate::is_keyword_like(self.as_str())
	}
}

impl From<json_syntax::object::Key> for Key {
	fn from(k: json_syntax::object::Key) -> Self {
		Self::from(k.into_string())
	}
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for Key {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

impl From<String> for Key {
	fn from(k: String) -> Self {
		Self(k)
	}
}

impl<'a> From<&'a str> for Key {
	fn from(value: &'a str) -> Self {
		Self(value.to_owned())
	}
}

impl fmt::Display for Key {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl Borrow<str> for Key {
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
		crate::is_keyword_like(self.as_str())
	}

	pub fn as_str(&self) -> &'a str {
		self.0
	}

	pub fn to_owned(self) -> Key {
		Key(self.0.to_owned())
	}
}

impl<'a> From<&'a str> for KeyRef<'a> {
	fn from(s: &'a str) -> Self {
		Self(s)
	}
}

impl<'a> From<&'a Key> for KeyRef<'a> {
	fn from(k: &'a Key) -> Self {
		Self(&k.0)
	}
}

impl<'a> fmt::Display for KeyRef<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyOrKeyword {
	Keyword(Keyword),
	Key(Key),
}

impl KeyOrKeyword {
	pub fn is_empty(&self) -> bool {
		match self {
			Self::Keyword(_) => false,
			Self::Key(k) => k.is_empty(),
		}
	}

	pub fn into_keyword(self) -> Option<Keyword> {
		match self {
			Self::Keyword(k) => Some(k),
			Self::Key(_) => None,
		}
	}

	pub fn into_key(self) -> Option<Key> {
		match self {
			Self::Keyword(_) => None,
			Self::Key(k) => Some(k),
		}
	}

	pub fn as_keyword(&self) -> Option<Keyword> {
		match self {
			Self::Keyword(k) => Some(*k),
			Self::Key(_) => None,
		}
	}

	pub fn as_key(&self) -> Option<&Key> {
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
impl Hash for KeyOrKeyword {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

impl fmt::Display for KeyOrKeyword {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Key(k) => k.fmt(f),
			Self::Keyword(k) => k.fmt(f),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KeyOrKeywordRef<'a> {
	Keyword(Keyword),
	Key(KeyRef<'a>),
}

impl<'a> KeyOrKeywordRef<'a> {
	pub fn to_owned(self) -> KeyOrKeyword {
		match self {
			Self::Keyword(k) => KeyOrKeyword::Keyword(k),
			Self::Key(k) => KeyOrKeyword::Key(k.to_owned()),
		}
	}

	pub fn as_str(&self) -> &'a str {
		match self {
			Self::Keyword(k) => k.into_str(),
			Self::Key(k) => k.as_str(),
		}
	}
}

impl<'a> From<&'a str> for KeyOrKeywordRef<'a> {
	fn from(s: &'a str) -> Self {
		match Keyword::try_from(s) {
			Ok(k) => Self::Keyword(k),
			Err(_) => Self::Key(s.into()),
		}
	}
}

impl<'a> From<&'a KeyOrKeyword> for KeyOrKeywordRef<'a> {
	fn from(k: &'a KeyOrKeyword) -> Self {
		match k {
			KeyOrKeyword::Keyword(k) => Self::Keyword(*k),
			KeyOrKeyword::Key(k) => Self::Key(k.into()),
		}
	}
}

impl<'a> From<KeyRef<'a>> for KeyOrKeywordRef<'a> {
	fn from(k: KeyRef<'a>) -> Self {
		Self::Key(k)
	}
}

impl<'a> From<&'a Key> for KeyOrKeywordRef<'a> {
	fn from(k: &'a Key) -> Self {
		Self::Key(k.into())
	}
}

pub enum KeyOrType {
	Key(Key),
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
