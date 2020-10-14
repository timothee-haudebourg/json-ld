use std::fmt;
use iref::{
	Iri,
	AsIri
};
use json::JsonValue;
use crate::{
	Id,
	Lenient,
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
pub enum Term<T: AsIri> {
	Null,
	Ref(Reference<T>),
	Keyword(Keyword)
}

impl<T: AsIri> Term<T> {
	pub fn is_null(&self) -> bool {
		match self {
			Term::Null => true,
			_ => false
		}
	}

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
			_ => None
		}
	}
}

impl<T: AsIri> TermLike for Term<T> {
	fn as_iri(&self) -> Option<Iri> {
		self.as_iri()
	}

	fn as_str(&self) -> &str {
		self.as_str()
	}
}

impl<'a, T: AsIri> From<&'a Term<T>> for Term<&'a T> {
	fn from(t: &'a Term<T>) -> Term<&'a T> {
		match t {
			Term::Null => Term::Null,
			Term::Ref(r) => Term::Ref(r.into()),
			Term::Keyword(k) => Term::Keyword(*k)
		}
	}
}

impl<'a, T: Id> From<&'a Lenient<Term<T>>> for Lenient<Term<&'a T>> {
	fn from(t: &'a Lenient<Term<T>>) -> Lenient<Term<&'a T>> {
		match t {
			Lenient::Ok(t) => Lenient::Ok(t.into()),
			Lenient::Unknown(u) => Lenient::Unknown(u.clone())
		}
	}
}

impl<T: AsIri> From<T> for Term<T> {
	fn from(id: T) -> Term<T> {
		Term::Ref(Reference::Id(id))
	}
}

impl<T: AsIri> From<BlankId> for Term<T> {
	fn from(blank: BlankId) -> Term<T> {
		Term::Ref(Reference::Blank(blank))
	}
}

impl<T: AsIri> From<Reference<T>> for Term<T> {
	fn from(prop: Reference<T>) -> Term<T> {
		Term::Ref(prop)
	}
}

impl<T: AsIri> From<Reference<T>> for Lenient<Term<T>> {
	fn from(prop: Reference<T>) -> Lenient<Term<T>> {
		Lenient::Ok(Term::Ref(prop))
	}
}

impl<'a, T: AsIri> From<&'a Reference<T>> for Lenient<Term<&'a T>> {
	fn from(r: &'a Reference<T>) -> Lenient<Term<&'a T>> {
		Lenient::Ok(Term::Ref(r.into()))
	}
}

impl<'a, T: AsIri> From<&'a Lenient<Reference<T>>> for Lenient<Term<&'a T>> {
	fn from(r: &'a Lenient<Reference<T>>) -> Lenient<Term<&'a T>> {
		match r {
			Lenient::Ok(r) => Lenient::Ok(Term::Ref(r.into())),
			Lenient::Unknown(u) => Lenient::Unknown(u.clone())
		}
	}
}

impl<T: AsIri + fmt::Display> fmt::Display for Term<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Term::Ref(p) => p.fmt(f),
			Term::Keyword(kw) => kw.into_str().fmt(f),
			Term::Null => write!(f, "null")
		}
	}
}

impl<T: AsIri> AsJson for Term<T> {
	fn as_json(&self) -> JsonValue {
		match self {
			Term::Ref(p) => p.as_str().into(),
			Term::Keyword(kw) => kw.into_str().into(),
			Term::Null => JsonValue::Null
		}
	}
}
