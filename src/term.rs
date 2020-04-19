use std::fmt;
use iref::Iri;
use json::JsonValue;
use crate::{Id, Keyword, Property, BlankId, AsJson};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Term<T: Id> {
	Null,
	Prop(Property<T>),
	Keyword(Keyword),
	Unknown(String)
}

impl<T: Id> Term<T> {
	pub fn as_str(&self) -> &str {
		match self {
			Term::Prop(p) => p.as_str(),
			Term::Keyword(k) => k.into_str(),
			Term::Unknown(u) => u.as_str(),
			Term::Null => ""
		}
	}

	pub fn is_keyword(&self) -> bool {
		match self {
			Term::Keyword(_) => true,
			_ => false
		}
	}

	pub fn iri(&self) -> Option<Iri> {
		match self {
			Term::Prop(p) => p.iri(),
			Term::Keyword(k) => k.iri(),
			_ => None
		}
	}

	pub fn iri_eq(&self, other: &Term<T>) -> bool {
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

impl<T: Id> From<T> for Term<T> {
	fn from(id: T) -> Term<T> {
		Term::Prop(Property::Id(id))
	}
}

impl<T: Id> From<BlankId> for Term<T> {
	fn from(blank: BlankId) -> Term<T> {
		Term::Prop(Property::Blank(blank))
	}
}

impl<T: Id> From<Property<T>> for Term<T> {
	fn from(prop: Property<T>) -> Term<T> {
		Term::Prop(prop)
	}
}

impl<T: Id> fmt::Display for Term<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Term::Prop(p) => p.fmt(f),
			Term::Keyword(kw) => kw.into_str().fmt(f),
			Term::Unknown(u) => u.fmt(f),
			Term::Null => write!(f, "null")
		}
	}
}

impl<T: Id> AsJson for Term<T> {
	fn as_json(&self) -> JsonValue {
		match self {
			Term::Prop(p) => p.as_str().into(),
			Term::Keyword(kw) => kw.into_str().into(),
			Term::Unknown(u) => u.as_str().into(),
			Term::Null => JsonValue::Null
		}
	}
}
