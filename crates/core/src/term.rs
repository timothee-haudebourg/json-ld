use crate::{Id, ValidId};
use contextual::{AsRefWithContext, DisplayWithContext, WithContext};
use iref::IriBuf;
use json_ld_syntax::Keyword;
use rdf_types::{vocabulary::Vocabulary, BlankIdBuf};
use std::fmt;

/// Identifier, keyword or `@null`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Term<T = IriBuf, B = BlankIdBuf> {
	/// `@null` value.
	Null,

	/// Node identifier.
	Id(Id<T, B>),

	/// Keyword.
	Keyword(Keyword),
}

impl<I, B> Term<I, B> {
	/// Checks if this term is `@null`.
	pub fn is_null(&self) -> bool {
		matches!(self, Term::Null)
	}

	/// Turns this term into an IRI if possible.
	///
	/// If it is not an IRI, returns the term itself.
	pub fn into_iri(self) -> Result<I, Self> {
		match self {
			Term::Id(Id::Valid(ValidId::Iri(id))) => Ok(id),
			term => Err(term),
		}
	}

	/// Checks if this term is a keyword.
	pub fn is_keyword(&self) -> bool {
		matches!(self, Term::Keyword(_))
	}

	/// Returns a reference to the IRI representation of the term, if any.
	pub fn as_iri(&self) -> Option<&I> {
		match self {
			Term::Id(p) => p.as_iri(),
			_ => None,
		}
	}

	pub fn map_id<U, C>(
		self,
		f: impl FnOnce(rdf_types::Id<I, B>) -> rdf_types::Id<U, C>,
	) -> Term<U, C> {
		match self {
			Self::Null => Term::Null,
			Self::Keyword(k) => Term::Keyword(k),
			Self::Id(id) => Term::Id(id.map(f)),
		}
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> DisplayWithContext<N> for Term<T, B> {
	fn fmt_with(&self, vocabulary: &N, f: &mut fmt::Formatter) -> fmt::Result {
		use std::fmt::Display;
		match self {
			Self::Null => write!(f, "null"),
			Self::Id(id) => id.with(vocabulary).fmt(f),
			Self::Keyword(k) => k.fmt(f),
		}
	}
}

impl<T: AsRef<str>, B: AsRef<str>> Term<T, B> {
	pub fn as_str(&self) -> &str {
		match self {
			Term::Id(p) => p.as_str(),
			Term::Keyword(k) => k.into_str(),
			Term::Null => "",
		}
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> AsRefWithContext<str, N> for Term<T, B> {
	fn as_ref_with<'a>(&'a self, vocabulary: &'a N) -> &'a str {
		match self {
			Term::Id(p) => p.with(vocabulary).as_str(),
			Term::Keyword(k) => k.into_str(),
			Term::Null => "",
		}
	}
}

impl<'a, T, B> From<&'a Term<T, B>> for Term<&'a T, &'a B> {
	fn from(t: &'a Term<T, B>) -> Term<&'a T, &'a B> {
		match t {
			Term::Null => Term::Null,
			Term::Id(r) => Term::Id(r.into()),
			Term::Keyword(k) => Term::Keyword(*k),
		}
	}
}

impl<T, B> From<T> for Term<T, B> {
	fn from(id: T) -> Term<T, B> {
		Term::Id(Id::Valid(ValidId::Iri(id)))
	}
}

impl<T, B> From<Id<T, B>> for Term<T, B> {
	fn from(prop: Id<T, B>) -> Term<T, B> {
		Term::Id(prop)
	}
}

impl<T: fmt::Display, B: fmt::Display> fmt::Display for Term<T, B> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Term::Id(p) => p.fmt(f),
			Term::Keyword(kw) => kw.fmt(f),
			Term::Null => write!(f, "null"),
		}
	}
}

impl<T: fmt::Debug, B: fmt::Debug> fmt::Debug for Term<T, B> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Term::Id(p) => write!(f, "Term::Ref({p:?})"),
			Term::Keyword(kw) => write!(f, "Term::Keyword({kw})"),
			Term::Null => write!(f, "Term::Null"),
		}
	}
}
