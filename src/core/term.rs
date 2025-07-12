use iref::{Iri, IriBuf};
use std::fmt;

use crate::{syntax::Keyword, Id, ValidId};

/// Identifier, keyword or `@null`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Term {
	/// `@null` value.
	Null,

	/// Node identifier.
	Id(Id),

	/// Keyword.
	Keyword(Keyword),
}

impl Term {
	/// Checks if this term is `@null`.
	pub fn is_null(&self) -> bool {
		matches!(self, Term::Null)
	}

	/// Turns this term into an IRI if possible.
	///
	/// If it is not an IRI, returns the term itself.
	pub fn into_iri(self) -> Result<IriBuf, Self> {
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
	pub fn as_iri(&self) -> Option<&Iri> {
		match self {
			Term::Id(p) => p.as_iri(),
			_ => None,
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Term::Id(p) => p.as_str(),
			Term::Keyword(k) => k.into_str(),
			Term::Null => "",
		}
	}
}

impl From<IriBuf> for Term {
	fn from(id: IriBuf) -> Term {
		Term::Id(Id::Valid(ValidId::Iri(id)))
	}
}

impl From<Id> for Term {
	fn from(prop: Id) -> Term {
		Term::Id(prop)
	}
}

impl fmt::Display for Term {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}
