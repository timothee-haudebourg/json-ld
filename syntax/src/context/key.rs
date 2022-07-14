use crate::{CompactIri, CompactIriBuf, InvalidCompactIri, Keyword};
use iref::{Iri, IriBuf};
use locspan_derive::StrippedPartialEq;
use rdf_types::{BlankId, BlankIdBuf};
use std::borrow::Borrow;
use std::fmt;

/// Context key.
#[derive(Clone, PartialEq, StrippedPartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Key {
	Iri(#[stripped] IriBuf),
	CompactIri(#[stripped] CompactIriBuf),
	Blank(#[stripped] BlankIdBuf),
	Term(#[stripped] String),
}

impl From<json_syntax::object::Key> for Key {
	fn from(k: json_syntax::object::Key) -> Self {
		Self::from(k.into_string())
	}
}

impl From<String> for Key {
	fn from(k: String) -> Self {
		match BlankIdBuf::new(k) {
			Ok(b) => Self::Blank(b),
			Err(rdf_types::InvalidBlankId(k)) => match CompactIriBuf::new(k) {
				Ok(c) => Self::CompactIri(c),
				Err(InvalidCompactIri(k)) => match IriBuf::from_string(k) {
					Ok(iri) => Self::Iri(iri),
					Err((_, k)) => Self::Term(k),
				},
			},
		}
	}
}

impl Key {
	pub fn len(&self) -> usize {
		match self {
			Self::Iri(i) => i.len(),
			Self::CompactIri(i) => i.len(),
			Self::Blank(b) => b.len(),
			Self::Term(t) => t.len(),
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Self::Iri(i) => i.as_str(),
			Self::CompactIri(i) => i.as_str(),
			Self::Blank(i) => i.as_str(),
			Self::Term(t) => t.as_str(),
		}
	}

	pub fn is_empty(&self) -> bool {
		match self {
			Self::Term(t) => t.is_empty(),
			_ => false,
		}
	}
}

impl fmt::Display for Key {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Iri(i) => i.fmt(f),
			Self::CompactIri(i) => i.fmt(f),
			Self::Blank(i) => i.fmt(f),
			Self::Term(t) => t.fmt(f),
		}
	}
}

impl Borrow<str> for Key {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyRef<'a> {
	Iri(Iri<'a>),
	CompactIri(&'a CompactIri),
	Blank(&'a BlankId),
	Term(&'a str),
}

impl<'a> KeyRef<'a> {
	pub fn is_empty(&self) -> bool {
		match self {
			Self::Term(t) => t.is_empty(),
			_ => false,
		}
	}

	pub fn is_keyword_like(&self) -> bool {
		crate::is_keyword_like(self.as_str())
	}

	pub fn as_str(&self) -> &'a str {
		match self {
			Self::Iri(i) => i.into_str(),
			Self::CompactIri(i) => i.as_str(),
			Self::Blank(i) => i.as_str(),
			Self::Term(s) => s,
		}
	}

	pub fn to_owned(self) -> Key {
		match self {
			Self::Iri(i) => Key::Iri(i.to_owned()),
			Self::CompactIri(i) => Key::CompactIri(i.to_owned()),
			Self::Blank(i) => Key::Blank(i.to_owned()),
			Self::Term(t) => Key::Term(t.to_owned()),
		}
	}
}

impl<'a> From<&'a Key> for KeyRef<'a> {
	fn from(k: &'a Key) -> Self {
		match k {
			Key::Iri(i) => Self::Iri(i.as_iri()),
			Key::CompactIri(i) => Self::CompactIri(i),
			Key::Blank(i) => Self::Blank(i),
			Key::Term(t) => Self::Term(t),
		}
	}
}

impl<'a> fmt::Display for KeyRef<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Iri(i) => i.fmt(f),
			Self::Blank(i) => i.fmt(f),
			Self::CompactIri(i) => i.fmt(f),
			Self::Term(t) => t.fmt(f),
		}
	}
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
}
