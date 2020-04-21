use std::fmt;
use iref::Iri;
use crate::{Id, BlankId};

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

	pub fn iri_eq(&self, other: &Property<T>) -> bool {
		if self == other {
			true
		} else {
			if let Some(self_iri) = self.as_iri() {
				if let Some(other_iri) = other.as_iri() {
					return self_iri == other_iri
				}
			}

			false
		}
	}
}

impl<T: Id> From<T> for Property<T> {
	fn from(id: T) -> Property<T> {
		Property::Id(id)
	}
}

impl<T: Id> From<BlankId> for Property<T> {
	fn from(blank: BlankId) -> Property<T> {
		Property::Blank(blank)
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
