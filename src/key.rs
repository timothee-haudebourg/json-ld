use std::fmt;
use iref::Iri;
use crate::{Id, Keyword, Property, BlankId};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Key<T: Id> {
	Prop(Property<T>),
	Keyword(Keyword)
}

impl<T: Id> Key<T> {
	pub fn as_str(&self) -> &str {
		match self {
			Key::Prop(p) => p.as_str(),
			Key::Keyword(k) => k.into_str()
		}
	}

	pub fn is_keyword(&self) -> bool {
		match self {
			Key::Keyword(_) => true,
			_ => false
		}
	}

	pub fn iri(&self) -> Option<Iri> {
		match self {
			Key::Prop(p) => p.iri(),
			Key::Keyword(k) => k.iri(),
		}
	}

	pub fn iri_eq(&self, other: &Key<T>) -> bool {
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

impl<T: Id> From<T> for Key<T> {
	fn from(id: T) -> Key<T> {
		Key::Prop(Property::Id(id))
	}
}

impl<T: Id> From<BlankId> for Key<T> {
	fn from(blank: BlankId) -> Key<T> {
		Key::Prop(Property::Blank(blank))
	}
}

impl<T: Id> From<Property<T>> for Key<T> {
	fn from(prop: Property<T>) -> Key<T> {
		Key::Prop(prop)
	}
}

impl<T: Id> fmt::Display for Key<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Key::Prop(p) => p.fmt(f),
			Key::Keyword(kw) => kw.into_str().fmt(f)
		}
	}
}
