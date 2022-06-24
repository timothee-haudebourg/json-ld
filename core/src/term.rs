use json_ld_syntax::Keyword;
use crate::Reference;
use iref::{AsIri, Iri};
use std::fmt;

pub trait TermLike {
	fn as_iri(&self) -> Option<Iri>;

	fn as_str(&self) -> &str;
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Term<T> {
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

// impl<K: JsonBuild, T: AsIri> AsAnyJson<K> for Term<T> {
// 	fn as_json_with(&self, meta: K::MetaData) -> K {
// 		match self {
// 			Term::Ref(p) => p.as_str().as_json_with(meta),
// 			Term::Keyword(kw) => kw.into_str().as_json_with(meta),
// 			Term::Null => K::null(meta),
// 		}
// 	}
// }
