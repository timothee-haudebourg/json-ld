use super::Keyword;
use crate::{
	util::{AsJson, JsonFrom},
	BlankId, Reference,
};
use generic_json::JsonClone;
use iref::{AsIri, Iri};
use std::fmt;

pub trait TermLike {
	fn as_iri(&self) -> Option<Iri>;

	fn as_str(&self) -> &str;
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Term<T: AsIri> {
	Null,
	Ref(Reference<T>),
	Keyword(Keyword),
}

impl<T: AsIri> Term<T> {
	pub fn is_null(&self) -> bool {
		matches!(self, Term::Null)
	}

	pub fn into_id(self) -> Result<T, Self> {
		match self {
			Term::Ref(Reference::Id(id)) => Ok(id),
			term => Err(term),
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Term::Ref(p) => p.as_str(),
			Term::Keyword(k) => k.into_str(),
			Term::Null => "",
		}
	}

	pub fn is_keyword(&self) -> bool {
		matches!(self, Term::Keyword(_))
	}

	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Term::Ref(p) => p.as_iri(),
			_ => None,
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
			Term::Keyword(k) => Term::Keyword(*k),
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

impl<T: AsIri + fmt::Display> fmt::Display for Term<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Term::Ref(p) => p.fmt(f),
			Term::Keyword(kw) => kw.into_str().fmt(f),
			Term::Null => write!(f, "null"),
		}
	}
}

impl<T: AsIri> fmt::Debug for Term<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Term::Ref(p) => write!(f, "Term::Ref({:?})", p),
			Term::Keyword(kw) => write!(f, "Term::Keyword({})", kw),
			Term::Null => write!(f, "Term::Null"),
		}
	}
}

impl<J: JsonClone, K: JsonFrom<J>, T: AsIri> AsJson<J, K> for Term<T> {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		match self {
			Term::Ref(p) => p.as_str().as_json_with(meta),
			Term::Keyword(kw) => kw.into_str().as_json_with(meta),
			Term::Null => K::null(meta(None)),
		}
	}
}

// pub trait ToLenientTerm<T: Id> {
// 	type Target: Borrow<Lenient<Term<T>>>;

// 	fn to_lenient_term(&self) -> Self::Target;
// }

// impl<'a, T: Id> ToLenientTerm<T> for &'a Lenient<Term<T>> {
// 	type Target = &'a Lenient<Term<T>>;

// 	#[inline]
// 	fn to_lenient_term(&self) -> &'a Lenient<Term<T>> {
// 		self
// 	}
// }

// impl<'a, T: Id> ToLenientTerm<T> for &'a T {
// 	type Target = Lenient<Term<T>>;

// 	#[inline]
// 	fn to_lenient_term(&self) -> Lenient<Term<T>> {
// 		Lenient::Ok(Term::Ref(Reference::Id((*self).clone())))
// 	}
// }

// impl<T: Id> ToLenientTerm<T> for Keyword {
// 	type Target = Lenient<Term<T>>;

// 	#[inline]
// 	fn to_lenient_term(&self) -> Lenient<Term<T>> {
// 		Lenient::Ok(Term::Keyword(*self))
// 	}
// }

// impl<'a, T: Id> ToLenientTerm<T> for &'a Reference<T> {
// 	type Target = Lenient<Term<T>>;

// 	#[inline]
// 	fn to_lenient_term(&self) -> Lenient<Term<T>> {
// 		Lenient::Ok(Term::Ref((*self).clone()))
// 	}
// }

// impl<'a, T: Id> ToLenientTerm<T> for &'a Lenient<Reference<T>> {
// 	type Target = Lenient<Term<T>>;

// 	#[inline]
// 	fn to_lenient_term(&self) -> Lenient<Term<T>> {
// 		match self {
// 			Lenient::Ok(r) => Lenient::Ok(Term::Ref((*r).clone())),
// 			Lenient::Unknown(u) => Lenient::Unknown(u.clone())
// 		}
// 	}
// }
