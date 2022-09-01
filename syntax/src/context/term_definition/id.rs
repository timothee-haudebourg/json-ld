use crate::{context, CompactIri, ExpandableRef, Keyword};
use iref::Iri;
use locspan_derive::StrippedPartialEq;
use rdf_types::BlankId;
use std::fmt;
use std::hash::Hash;

#[derive(Clone, StrippedPartialEq, PartialOrd, Ord, Debug)]
pub enum Id {
	Term(#[stripped] String),
	Keyword(#[stripped] Keyword),
}

impl Id {
	pub fn as_iri(&self) -> Option<Iri> {
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

	pub fn as_compact_iri(&self) -> Option<&CompactIri> {
		match self {
			Self::Term(t) => CompactIri::new(t).ok(),
			Self::Keyword(_) => None,
		}
	}

	pub fn as_keyword(&self) -> Option<Keyword> {
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
}

impl PartialEq for Id {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Term(a), Self::Term(b)) => a == b,
			(Self::Keyword(a), Self::Keyword(b)) => a == b,
			_ => false,
		}
	}
}

impl Eq for Id {}

impl Hash for Id {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

impl From<String> for Id {
	fn from(s: String) -> Self {
		match Keyword::try_from(s.as_str()) {
			Ok(k) => Self::Keyword(k),
			Err(_) => Self::Term(s),
		}
	}
}

#[derive(Clone, Copy)]
pub enum IdRef<'a> {
	Term(&'a str),
	Keyword(Keyword),
}

impl<'a> IdRef<'a> {
	pub fn as_iri(&self) -> Option<Iri<'a>> {
		match self {
			Self::Term(t) => Iri::new(*t).ok(),
			Self::Keyword(_) => None,
		}
	}

	pub fn as_blank_id(&self) -> Option<&'a BlankId> {
		match self {
			Self::Term(t) => BlankId::new(t).ok(),
			Self::Keyword(_) => None,
		}
	}

	pub fn as_compact_iri(&self) -> Option<&'a CompactIri> {
		match self {
			Self::Term(t) => CompactIri::new(t).ok(),
			Self::Keyword(_) => None,
		}
	}

	pub fn as_keyword(&self) -> Option<Keyword> {
		match self {
			Self::Keyword(k) => Some(*k),
			Self::Term(_) => None,
		}
	}

	pub fn as_str(&self) -> &'a str {
		match self {
			Self::Term(t) => t,
			Self::Keyword(k) => k.into_str(),
		}
	}

	pub fn is_keyword(&self) -> bool {
		matches!(self, Self::Keyword(_))
	}

	pub fn is_keyword_like(&self) -> bool {
		crate::is_keyword_like(self.as_str())
	}
}

impl<'a> From<&'a str> for IdRef<'a> {
	fn from(s: &'a str) -> Self {
		match Keyword::try_from(s) {
			Ok(k) => Self::Keyword(k),
			Err(_) => Self::Term(s),
		}
	}
}

impl<'a> From<IdRef<'a>> for context::definition::KeyOrKeywordRef<'a> {
	fn from(i: IdRef<'a>) -> Self {
		match i {
			IdRef::Term(t) => Self::Key(t.into()),
			IdRef::Keyword(k) => Self::Keyword(k),
		}
	}
}

impl<'a> From<IdRef<'a>> for ExpandableRef<'a> {
	fn from(i: IdRef<'a>) -> Self {
		match i {
			IdRef::Term(t) => Self::String(t),
			IdRef::Keyword(k) => Self::Keyword(k),
		}
	}
}

impl<'a> fmt::Display for IdRef<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Term(t) => t.fmt(f),
			Self::Keyword(k) => k.fmt(f),
		}
	}
}

impl<'a> From<&'a Id> for IdRef<'a> {
	fn from(i: &'a Id) -> Self {
		match i {
			Id::Term(t) => Self::Term(t),
			Id::Keyword(k) => Self::Keyword(*k),
		}
	}
}
