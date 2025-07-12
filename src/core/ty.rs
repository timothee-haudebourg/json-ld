use std::convert::TryFrom;
use std::fmt;

use iref::{Iri, IriBuf};

use super::Term;
use crate::{syntax::Keyword, Id, ValidId};

/// Object type.
///
/// This is the value of a `@type` entry.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Type {
	/// `@id`.
	///
	/// The value must be interpreted as an IRI.
	Id,

	/// `@json`.
	///
	/// The value is an arbitrary JSON value.
	Json,

	/// `@none`
	None,

	/// `@vocab`
	Vocab,

	/// IRI type.
	Iri(IriBuf),
}

impl Type {
	/// Turns this type into an IRI if possible.
	pub fn into_iri(self) -> Result<IriBuf, Self> {
		match self {
			Type::Iri(id) => Ok(id),
			typ => Err(typ),
		}
	}

	/// Returns a reference to the type IRI if any.
	pub fn as_iri(&self) -> Option<&Iri> {
		match self {
			Type::Iri(id) => Some(id),
			_ => None,
		}
	}

	/// Returns a JSON-LD string representation of the type.
	pub fn as_str(&self) -> &str {
		match self {
			Type::Id => "@id",
			Type::Json => "@json",
			Type::None => "@none",
			Type::Vocab => "@vocab",
			Type::Iri(id) => id.as_ref(),
		}
	}
}

impl From<Type> for Term {
	fn from(t: Type) -> Term {
		match t {
			Type::Id => Term::Keyword(Keyword::Id),
			Type::Json => Term::Keyword(Keyword::Json),
			Type::None => Term::Keyword(Keyword::None),
			Type::Vocab => Term::Keyword(Keyword::Vocab),
			Type::Iri(id) => Term::Id(Id::Valid(ValidId::Iri(id))),
		}
	}
}

impl TryFrom<Term> for Type {
	type Error = Term;

	fn try_from(term: Term) -> Result<Type, Term> {
		match term {
			Term::Keyword(Keyword::Id) => Ok(Type::Id),
			Term::Keyword(Keyword::Json) => Ok(Type::Json),
			Term::Keyword(Keyword::None) => Ok(Type::None),
			Term::Keyword(Keyword::Vocab) => Ok(Type::Vocab),
			Term::Id(Id::Valid(ValidId::Iri(id))) => Ok(Type::Iri(id)),
			term => Err(term),
		}
	}
}

impl fmt::Display for Type {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}
