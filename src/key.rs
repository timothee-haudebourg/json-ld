use std::fmt;
use iref::Iri;
use json::JsonValue;
use crate::{Id, Keyword, Property, BlankId, AsJson};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Key<T: Id> {
	Null,
	Prop(Property<T>),
	Keyword(Keyword),
	Unknown(String)
}

impl<T: Id> Key<T> {
	pub fn as_str(&self) -> &str {
		match self {
			Key::Prop(p) => p.as_str(),
			Key::Keyword(k) => k.into_str(),
			Key::Unknown(u) => u.as_str(),
			Key::Null => ""
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
			_ => None
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
			Key::Keyword(kw) => kw.into_str().fmt(f),
			Key::Unknown(u) => u.fmt(f),
			Key::Null => write!(f, "null")
		}
	}
}

impl<T: Id> AsJson for Key<T> {
	fn as_json(&self) -> JsonValue {
		match self {
			Key::Prop(p) => p.as_str().into(),
			Key::Keyword(kw) => kw.into_str().into(),
			Key::Unknown(u) => u.as_str().into(),
			Key::Null => JsonValue::Null
		}
	}
}
