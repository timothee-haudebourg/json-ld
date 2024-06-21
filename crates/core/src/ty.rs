use super::Term;
use crate::{Id, ValidId};
use iref::IriBuf;
use json_ld_syntax::Keyword;
use std::convert::TryFrom;
use std::fmt;

/// Object type.
///
/// This is the value of a `@type` entry.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Type<I = IriBuf> {
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
	Iri(I),
}

impl<I> Type<I> {
	/// Turns this type into an IRI if possible.
	pub fn into_iri(self) -> Result<I, Type<I>> {
		match self {
			Type::Iri(id) => Ok(id),
			typ => Err(typ),
		}
	}

	/// Maps the IRI of this type.
	pub fn map<U>(self, f: impl FnOnce(I) -> U) -> Type<U> {
		match self {
			Type::Id => Type::Id,
			Type::Json => Type::Json,
			Type::None => Type::None,
			Type::Vocab => Type::Vocab,
			Type::Iri(t) => Type::Iri(f(t)),
		}
	}
}

impl<'a, I: Clone> Type<&'a I> {
	/// Clones the referenced IRI.
	pub fn cloned(self) -> Type<I> {
		match self {
			Type::Id => Type::Id,
			Type::Json => Type::Json,
			Type::None => Type::None,
			Type::Vocab => Type::Vocab,
			Type::Iri(t) => Type::Iri(t.clone()),
		}
	}
}

impl<I> Type<I> {
	/// Returns a reference to the type IRI if any.
	pub fn as_iri(&self) -> Option<&I> {
		match self {
			Type::Iri(id) => Some(id),
			_ => None,
		}
	}
}

impl<I: AsRef<str>> Type<I> {
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

impl<'a, I> From<&'a Type<I>> for Type<&'a I> {
	fn from(t: &'a Type<I>) -> Type<&'a I> {
		match t {
			Type::Id => Type::Id,
			Type::Json => Type::Json,
			Type::None => Type::None,
			Type::Vocab => Type::Vocab,
			Type::Iri(id) => Type::Iri(id),
		}
	}
}

impl<I, B> From<Type<I>> for Term<I, B> {
	fn from(t: Type<I>) -> Term<I, B> {
		match t {
			Type::Id => Term::Keyword(Keyword::Id),
			Type::Json => Term::Keyword(Keyword::Json),
			Type::None => Term::Keyword(Keyword::None),
			Type::Vocab => Term::Keyword(Keyword::Vocab),
			Type::Iri(id) => Term::Id(Id::Valid(ValidId::Iri(id))),
		}
	}
}

impl<I, B> TryFrom<Term<I, B>> for Type<I> {
	type Error = Term<I, B>;

	fn try_from(term: Term<I, B>) -> Result<Type<I>, Term<I, B>> {
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

impl<I: fmt::Display> fmt::Display for Type<I> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Type::Id => write!(f, "@id"),
			Type::Json => write!(f, "@json"),
			Type::None => write!(f, "@none"),
			Type::Vocab => write!(f, "@vocab"),
			Type::Iri(id) => id.fmt(f),
		}
	}
}
