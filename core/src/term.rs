use crate::{Id, ValidId};
use contextual::{AsRefWithContext, DisplayWithContext, WithContext};
use json_ld_syntax::Keyword;
use rdf_types::vocabulary::Vocabulary;
use std::fmt;

// pub trait TermLike {
// 	fn as_iri(&self) -> Option<Iri>;

// 	fn as_str(&self) -> &str;
// }

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Term<T, B> {
	Null,
	Ref(Id<T, B>),
	Keyword(Keyword),
}

impl<I, B> Term<I, B> {
	pub fn is_null(&self) -> bool {
		matches!(self, Term::Null)
	}

	pub fn into_id(self) -> Result<I, Self> {
		match self {
			Term::Ref(Id::Valid(ValidId::Iri(id))) => Ok(id),
			term => Err(term),
		}
	}

	pub fn is_keyword(&self) -> bool {
		matches!(self, Term::Keyword(_))
	}

	pub fn as_iri(&self) -> Option<&I> {
		match self {
			Term::Ref(p) => p.as_iri(),
			_ => None,
		}
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> DisplayWithContext<N> for Term<T, B> {
	fn fmt_with(&self, vocabulary: &N, f: &mut fmt::Formatter) -> fmt::Result {
		use std::fmt::Display;
		match self {
			Self::Null => write!(f, "null"),
			Self::Ref(id) => id.with(vocabulary).fmt(f),
			Self::Keyword(k) => k.fmt(f),
		}
	}
}

impl<T: AsRef<str>, B: AsRef<str>> Term<T, B> {
	pub fn as_str(&self) -> &str {
		match self {
			Term::Ref(p) => p.as_str(),
			Term::Keyword(k) => k.into_str(),
			Term::Null => "",
		}
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> AsRefWithContext<str, N> for Term<T, B> {
	fn as_ref_with<'a>(&'a self, vocabulary: &'a N) -> &'a str {
		match self {
			Term::Ref(p) => p.with(vocabulary).as_str(),
			Term::Keyword(k) => k.into_str(),
			Term::Null => "",
		}
	}
}

// impl<T> TermLike for Term<T, B> {
// 	fn as_iri(&self) -> Option<Iri> {
// 		self.as_iri()
// 	}

// 	fn as_str(&self) -> &str {
// 		self.as_str()
// 	}
// }

impl<'a, T, B> From<&'a Term<T, B>> for Term<&'a T, &'a B> {
	fn from(t: &'a Term<T, B>) -> Term<&'a T, &'a B> {
		match t {
			Term::Null => Term::Null,
			Term::Ref(r) => Term::Ref(r.into()),
			Term::Keyword(k) => Term::Keyword(*k),
		}
	}
}

impl<T, B> From<T> for Term<T, B> {
	fn from(id: T) -> Term<T, B> {
		Term::Ref(Id::Valid(ValidId::Iri(id)))
	}
}

impl<T, B> From<Id<T, B>> for Term<T, B> {
	fn from(prop: Id<T, B>) -> Term<T, B> {
		Term::Ref(prop)
	}
}

impl<T: fmt::Display, B: fmt::Display> fmt::Display for Term<T, B> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Term::Ref(p) => p.fmt(f),
			Term::Keyword(kw) => kw.fmt(f),
			Term::Null => write!(f, "null"),
		}
	}
}

impl<T: fmt::Debug, B: fmt::Debug> fmt::Debug for Term<T, B> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Term::Ref(p) => write!(f, "Term::Ref({:?})", p),
			Term::Keyword(kw) => write!(f, "Term::Keyword({})", kw),
			Term::Null => write!(f, "Term::Null"),
		}
	}
}
