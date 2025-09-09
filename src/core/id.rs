use iref::{Iri, IriBuf};
use rdf_types::{BlankId, BlankIdBuf, InvalidBlankId};
use std::convert::TryFrom;
use std::fmt;
use std::hash::Hash;

pub type ValidId = rdf_types::Id;

use crate::Term;

/// Node identifier.
///
/// Used to reference a node across a document or to a remote document.
/// It can be an identifier (IRI), a blank node identifier for local blank nodes
/// or an invalid reference (a string that is neither an IRI nor blank node identifier).
///
/// # `Hash` implementation
///
/// It is guaranteed that the `Hash` implementation of `Id` is *transparent*,
/// meaning that the hash of `Id::Valid(id)` the same as `id`, and the hash of
/// `Id::Invalid(id)` is the same as `id`.
///
/// This may be useful to define custom [`indexmap::Equivalent<Id>`]
/// implementation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Id {
	/// Valid node identifier.
	Valid(ValidId),

	/// Invalid reference.
	Invalid(String),
}

impl Id {
	pub fn from_string(s: String) -> Self {
		match IriBuf::new(s) {
			Ok(iri) => Self::Valid(ValidId::Iri(iri.into())),
			Err(e) => match BlankIdBuf::new(e.0) {
				Ok(blank) => Self::Valid(ValidId::BlankId(blank.into())),
				Err(InvalidBlankId(s)) => Self::Invalid(s),
			},
		}
	}

	pub fn iri(iri: IriBuf) -> Self {
		Self::Valid(ValidId::Iri(iri))
	}

	pub fn blank(b: BlankIdBuf) -> Self {
		Self::Valid(ValidId::BlankId(b))
	}

	/// Checks if this is a valid reference.
	///
	/// Returns `true` is this reference is a node identifier or a blank node identifier,
	/// `false` otherwise.
	#[inline(always)]
	pub fn is_valid(&self) -> bool {
		!matches!(self, Self::Invalid(_))
	}

	/// Get a string representation of the reference.
	///
	/// This will either return a string slice of an IRI, or a blank node identifier.
	#[inline(always)]
	pub fn as_str(&self) -> &str {
		match self {
			Id::Valid(ValidId::Iri(id)) => id.as_ref(),
			Id::Valid(ValidId::BlankId(id)) => id.as_ref(),
			Id::Invalid(id) => id.as_str(),
		}
	}

	pub fn into_blank(self) -> Option<BlankIdBuf> {
		match self {
			Self::Valid(ValidId::BlankId(b)) => Some(b),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn is_blank(&self) -> bool {
		matches!(self, Id::Valid(ValidId::BlankId(_)))
	}

	#[inline(always)]
	pub fn as_blank(&self) -> Option<&BlankId> {
		match self {
			Id::Valid(ValidId::BlankId(k)) => Some(k),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn is_iri(&self) -> bool {
		matches!(self, Id::Valid(ValidId::Iri(_)))
	}

	#[inline(always)]
	pub fn as_iri(&self) -> Option<&Iri> {
		match self {
			Id::Valid(ValidId::Iri(k)) => Some(k),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn into_term(self) -> Term {
		Term::Id(self)
	}

	pub fn as_ref(&self) -> Ref {
		match self {
			Self::Valid(ValidId::Iri(t)) => Ref::Iri(t),
			Self::Valid(ValidId::BlankId(id)) => Ref::Blank(id),
			Self::Invalid(id) => Ref::Invalid(id.as_str()),
		}
	}

	pub fn map(self, f: impl FnOnce(ValidId) -> ValidId) -> Self {
		match self {
			Self::Valid(id) => Id::Valid(f(id)),
			Self::Invalid(id) => Id::Invalid(id),
		}
	}
}

impl indexmap::Equivalent<Id> for ValidId {
	fn equivalent(&self, key: &Id) -> bool {
		match key {
			Id::Valid(id) => self == id,
			_ => false,
		}
	}
}

impl<'a> indexmap::Equivalent<Id> for &'a Iri {
	fn equivalent(&self, key: &Id) -> bool {
		match key {
			Id::Valid(ValidId::Iri(iri)) => *self == iri,
			_ => false,
		}
	}
}

impl indexmap::Equivalent<Id> for iref::IriBuf {
	fn equivalent(&self, key: &Id) -> bool {
		match key {
			Id::Valid(ValidId::Iri(iri)) => self == iri,
			_ => false,
		}
	}
}

impl indexmap::Equivalent<Id> for rdf_types::BlankId {
	fn equivalent(&self, key: &Id) -> bool {
		match key {
			Id::Valid(ValidId::BlankId(b)) => self == b,
			_ => false,
		}
	}
}

impl indexmap::Equivalent<Id> for rdf_types::BlankIdBuf {
	fn equivalent(&self, key: &Id) -> bool {
		match key {
			Id::Valid(ValidId::BlankId(b)) => self == b,
			_ => false,
		}
	}
}

impl From<IriBuf> for Id {
	#[inline(always)]
	fn from(iri: IriBuf) -> Id {
		Self::iri(iri)
	}
}

impl PartialEq<Iri> for Id {
	fn eq(&self, other: &Iri) -> bool {
		self.as_iri() == Some(other)
	}
}

impl PartialEq<Term> for Id {
	#[inline]
	fn eq(&self, term: &Term) -> bool {
		match term {
			Term::Id(prop) => self == prop,
			_ => false,
		}
	}
}

impl PartialEq<Id> for Term {
	#[inline]
	fn eq(&self, r: &Id) -> bool {
		match self {
			Term::Id(prop) => prop == r,
			_ => false,
		}
	}
}

impl From<ValidId> for Id {
	fn from(r: ValidId) -> Self {
		Id::Valid(r)
	}
}

impl TryFrom<Term> for Id {
	type Error = Term;

	#[inline]
	fn try_from(term: Term) -> Result<Id, Term> {
		match term {
			Term::Id(prop) => Ok(prop),
			term => Err(term),
		}
	}
}

impl fmt::Display for Id {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Id::Valid(id) => id.fmt(f),
			Id::Invalid(id) => id.fmt(f),
		}
	}
}

impl TryFrom<Id> for ValidId {
	type Error = String;

	fn try_from(r: Id) -> Result<Self, Self::Error> {
		match r {
			Id::Valid(r) => Ok(r),
			Id::Invalid(id) => Err(id),
		}
	}
}

impl<'a> TryFrom<&'a Id> for &'a ValidId {
	type Error = &'a String;

	fn try_from(r: &'a Id) -> Result<Self, Self::Error> {
		match r {
			Id::Valid(r) => Ok(r),
			Id::Invalid(id) => Err(id),
		}
	}
}

impl<'a> TryFrom<&'a mut Id> for &'a mut ValidId {
	type Error = &'a mut String;

	fn try_from(r: &'a mut Id) -> Result<Self, Self::Error> {
		match r {
			Id::Valid(r) => Ok(r),
			Id::Invalid(id) => Err(id),
		}
	}
}

impl indexmap::Equivalent<Id> for Iri {
	fn equivalent(&self, key: &Id) -> bool {
		match key.as_iri() {
			Some(iri) => self == iri,
			None => false,
		}
	}
}

/// Id to a reference.
#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Ref<'a> {
	/// Node identifier, essentially an IRI.
	Iri(&'a Iri),

	/// Blank node identifier.
	Blank(&'a BlankId),

	/// Invalid reference.
	Invalid(&'a str),
}

// pub trait IdentifyAll {
// 	fn identify_all(&mut self, generator: &mut impl Generator);
// }

// pub trait Relabel {
// 	fn relabel_with<N: Vocabulary<Iri = T, BlankId = B>, G: Generator<N>>(
// 		&mut self,
// 		vocabulary: &mut N,
// 		generator: &mut G,
// 		relabeling: &mut HashMap<B, ValidId>,
// 	) where
// 		T: Clone + Eq + Hash,
// 		B: Clone + Eq + Hash;

// 	fn relabel<G: Generator>(&mut self, generator: &mut G, relabeling: &mut HashMap<B, ValidId>)
// 	where
// 		T: Clone + Eq + Hash,
// 		B: Clone + Eq + Hash,
// 		(): Vocabulary<Iri = T, BlankId = B>,
// 	{
// 		self.relabel_with(
// 			rdf_types::vocabulary::no_vocabulary_mut(),
// 			generator,
// 			relabeling,
// 		)
// 	}
// }
