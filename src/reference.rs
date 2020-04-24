use std::fmt;
use std::convert::TryFrom;
use iref::Iri;
use json::JsonValue;
use crate::{
	Id,
	BlankId,
	Lenient,
	syntax::{
		Term,
		TermLike,
	},
	util
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Reference<T: Id> {
	Id(T),
	Blank(BlankId)
}

impl<T: Id> Reference<T> {
	pub fn as_str(&self) -> &str {
		match self {
			Reference::Id(id) => id.as_iri().into_str(),
			Reference::Blank(id) => id.as_str()
		}
	}

	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Reference::Id(k) => Some(k.as_iri()),
			Reference::Blank(_) => None
		}
	}
}

impl<T: Id> TermLike for Reference<T> {
	fn as_iri(&self) -> Option<Iri> {
		self.as_iri()
	}

	fn as_str(&self) -> &str {
		self.as_str()
	}
}

impl<T: Id + PartialEq> PartialEq<T> for Reference<T> {
	fn eq(&self, other: &T) -> bool {
		match self {
			Reference::Id(id) => id == other,
			_ => false
		}
	}
}

impl<T: Id + PartialEq> PartialEq<T> for Lenient<Reference<T>> {
	fn eq(&self, other: &T) -> bool {
		match self {
			Lenient::Ok(Reference::Id(id)) => id == other,
			_ => false
		}
	}
}

impl<T: Id> From<T> for Reference<T> {
	fn from(id: T) -> Reference<T> {
		Reference::Id(id)
	}
}

impl<T: Id> TryFrom<Term<T>> for Reference<T> {
	type Error = Term<T>;

	fn try_from(term: Term<T>) -> Result<Reference<T>, Term<T>> {
		match term {
			Term::Ref(prop) => Ok(prop),
			term => Err(term)
		}
	}
}

impl<T: Id> From<BlankId> for Reference<T> {
	fn from(blank: BlankId) -> Reference<T> {
		Reference::Blank(blank)
	}
}

impl<T: Id> util::AsJson for Reference<T> {
	fn as_json(&self) -> JsonValue {
		match self {
			Reference::Id(id) => id.as_json(),
			Reference::Blank(b) => b.as_json()
		}
	}
}

impl<T: Id> fmt::Display for Reference<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Reference::Id(id) => id.fmt(f),
			Reference::Blank(b) => b.fmt(f)
		}
	}
}
