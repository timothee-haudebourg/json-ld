use std::fmt;
use iref::Iri;
use crate::{Id, BlankId};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Property<T: Id> {
	Id(T),
	Blank(BlankId)
}

impl<T: Id> Property<T> {
	pub fn as_str(&self) -> &str {
		match self {
			Property::Id(id) => id.iri().into_str(),
			Property::Blank(id) => id.as_str()
		}
	}

	pub fn iri(&self) -> Option<Iri> {
		match self {
			Property::Id(k) => Some(k.iri()),
			Property::Blank(_) => None
		}
	}

	fn iri_eq(&self, other: &Property<T>) -> bool {
		if self == other {
			true
		} else {
			if let Some(self_iri) = self.iri() {
				if let Some(other_iri) = other.iri() {
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
