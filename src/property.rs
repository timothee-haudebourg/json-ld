use std::fmt;
use std::convert::TryFrom;
use iref::Iri;
use json::JsonValue;
use crate::{
	Id,
	BlankId,
	Term,
	TermLike,
	util
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Property<T: Id> {
	Id(T),
	Blank(BlankId)
}

impl<T: Id> Property<T> {
	pub fn as_str(&self) -> &str {
		match self {
			Property::Id(id) => id.as_iri().into_str(),
			Property::Blank(id) => id.as_str()
		}
	}

	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Property::Id(k) => Some(k.as_iri()),
			Property::Blank(_) => None
		}
	}
}

impl<T: Id> TermLike for Property<T> {
	fn as_iri(&self) -> Option<Iri> {
		self.as_iri()
	}

	fn as_str(&self) -> &str {
		self.as_str()
	}
}

impl<T: Id> From<T> for Property<T> {
	fn from(id: T) -> Property<T> {
		Property::Id(id)
	}
}

impl<T: Id> TryFrom<Term<T>> for Property<T> {
	type Error = Term<T>;

	fn try_from(term: Term<T>) -> Result<Property<T>, Term<T>> {
		match term {
			Term::Prop(prop) => Ok(prop),
			term => Err(term)
		}
	}
}

impl<T: Id> From<BlankId> for Property<T> {
	fn from(blank: BlankId) -> Property<T> {
		Property::Blank(blank)
	}
}

impl<T: Id> util::AsJson for Property<T> {
	fn as_json(&self) -> JsonValue {
		match self {
			Property::Id(id) => id.as_json(),
			Property::Blank(b) => b.as_json()
		}
	}
}

impl<T: Id> fmt::Display for Property<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Property::Id(id) => id.fmt(f),
			Property::Blank(b) => b.fmt(f)
		}
	}
}
