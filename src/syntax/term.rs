use std::fmt;
use iref::Iri;
use json::JsonValue;
use crate::{
	Id,
	Reference,
	BlankId,
	util::AsJson
};
use super::Keyword;

pub trait TermLike {
	fn as_iri(&self) -> Option<Iri>;

	fn as_str(&self) -> &str;
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Term<T: Id> {
	Null,
	Ref(Reference<T>),
	Keyword(Keyword)
}

impl<T: Id> Term<T> {
	pub fn into_id(self) -> Result<T, Self> {
		match self {
			Term::Ref(Reference::Id(id)) => Ok(id),
			term => Err(term)
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Term::Ref(p) => p.as_str(),
			Term::Keyword(k) => k.into_str(),
			Term::Null => ""
		}
	}

	pub fn is_keyword(&self) -> bool {
		match self {
			Term::Keyword(_) => true,
			_ => false
		}
	}

	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Term::Ref(p) => p.as_iri(),
			Term::Keyword(k) => k.as_iri(),
			_ => None
		}
	}
}

impl<T: Id> TermLike for Term<T> {
	fn as_iri(&self) -> Option<Iri> {
		self.as_iri()
	}

	fn as_str(&self) -> &str {
		self.as_str()
	}
}

impl<T: Id> From<T> for Term<T> {
	fn from(id: T) -> Term<T> {
		Term::Ref(Reference::Id(id))
	}
}

impl<T: Id> From<BlankId> for Term<T> {
	fn from(blank: BlankId) -> Term<T> {
		Term::Ref(Reference::Blank(blank))
	}
}

impl<T: Id> From<Reference<T>> for Term<T> {
	fn from(prop: Reference<T>) -> Term<T> {
		Term::Ref(prop)
	}
}

impl<T: Id> fmt::Display for Term<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Term::Ref(p) => p.fmt(f),
			Term::Keyword(kw) => kw.into_str().fmt(f),
			Term::Null => write!(f, "null")
		}
	}
}

impl<T: Id> AsJson for Term<T> {
	fn as_json(&self) -> JsonValue {
		match self {
			Term::Ref(p) => p.as_str().into(),
			Term::Keyword(kw) => kw.into_str().into(),
			Term::Null => JsonValue::Null
		}
	}
}
