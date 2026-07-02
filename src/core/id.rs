use btree_indexmap::{Comparable, Equivalent};
use rdf_syntax::{BlankId, BlankIdBuf, Id};
use rdf_syntax::{Iri, IriBuf};
use std::cmp::Ordering;
use std::convert::TryFrom;

use crate::{Lenient, Term, Validate};

impl Validate for Id {
	type Invalid = String;
}

impl Lenient<Id> {
	pub fn iri(iri: IriBuf) -> Self {
		Self::Valid(Id::Iri(iri))
	}

	pub fn blank(b: BlankIdBuf) -> Self {
		Self::Valid(Id::BlankId(b))
	}

	pub fn into_blank(self) -> Option<BlankIdBuf> {
		match self {
			Self::Valid(Id::BlankId(b)) => Some(b),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn is_blank(&self) -> bool {
		matches!(self, Self::Valid(Id::BlankId(_)))
	}

	#[inline(always)]
	pub fn as_blank(&self) -> Option<&BlankId> {
		match self {
			Self::Valid(Id::BlankId(k)) => Some(k),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn is_iri(&self) -> bool {
		matches!(self, Self::Valid(Id::Iri(_)))
	}

	#[inline(always)]
	pub fn as_iri(&self) -> Option<&Iri> {
		match self {
			Self::Valid(Id::Iri(k)) => Some(k),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn into_term(self) -> Term {
		Term::Id(self)
	}
}

impl Equivalent<Lenient<Id>> for Id {
	fn equivalent(&self, key: &Lenient<Id>) -> bool {
		match key {
			Lenient::Valid(id) => self == id,
			_ => false,
		}
	}
}

impl Comparable<Lenient<Id>> for Id {
	fn compare(&self, key: &Lenient<Id>) -> Ordering {
		match key {
			Lenient::Valid(id) => self.cmp(id),
			Lenient::Invalid(_) => Ordering::Less, // Valid < Invalid
		}
	}
}

impl Equivalent<Lenient<Id>> for &Iri {
	fn equivalent(&self, key: &Lenient<Id>) -> bool {
		match key {
			Lenient::Valid(Id::Iri(iri)) => *self == iri,
			_ => false,
		}
	}
}

impl Comparable<Lenient<Id>> for &Iri {
	fn compare(&self, key: &Lenient<Id>) -> Ordering {
		match key {
			Lenient::Valid(Id::Iri(iri)) => (*self).cmp(iri.as_iri()),
			Lenient::Valid(Id::BlankId(_)) => Ordering::Greater, // Iri > BlankId
			Lenient::Invalid(_) => Ordering::Less,
		}
	}
}

impl Equivalent<Lenient<Id>> for rdf_syntax::IriBuf {
	fn equivalent(&self, key: &Lenient<Id>) -> bool {
		match key {
			Lenient::Valid(Id::Iri(iri)) => self == iri,
			_ => false,
		}
	}
}

impl Comparable<Lenient<Id>> for rdf_syntax::IriBuf {
	fn compare(&self, key: &Lenient<Id>) -> Ordering {
		match key {
			Lenient::Valid(Id::Iri(iri)) => self.as_iri().cmp(iri.as_iri()),
			Lenient::Valid(Id::BlankId(_)) => Ordering::Greater,
			Lenient::Invalid(_) => Ordering::Less,
		}
	}
}

impl Equivalent<Lenient<Id>> for rdf_syntax::BlankId {
	fn equivalent(&self, key: &Lenient<Id>) -> bool {
		match key {
			Lenient::Valid(Id::BlankId(b)) => self == b,
			_ => false,
		}
	}
}

impl Comparable<Lenient<Id>> for rdf_syntax::BlankId {
	fn compare(&self, key: &Lenient<Id>) -> Ordering {
		match key {
			Lenient::Valid(Id::BlankId(b)) => self.cmp(b),
			Lenient::Valid(Id::Iri(_)) => Ordering::Less, // BlankId < Iri
			Lenient::Invalid(_) => Ordering::Less,
		}
	}
}

impl Equivalent<Lenient<Id>> for rdf_syntax::BlankIdBuf {
	fn equivalent(&self, key: &Lenient<Id>) -> bool {
		match key {
			Lenient::Valid(Id::BlankId(b)) => self == b,
			_ => false,
		}
	}
}

impl Comparable<Lenient<Id>> for rdf_syntax::BlankIdBuf {
	fn compare(&self, key: &Lenient<Id>) -> Ordering {
		match key {
			Lenient::Valid(Id::BlankId(b)) => self.as_blank_id().cmp(b),
			Lenient::Valid(Id::Iri(_)) => Ordering::Less,
			Lenient::Invalid(_) => Ordering::Less,
		}
	}
}

impl From<IriBuf> for Lenient<Id> {
	#[inline(always)]
	fn from(iri: IriBuf) -> Self {
		Self::iri(iri)
	}
}

impl PartialEq<Iri> for Lenient<Id> {
	fn eq(&self, other: &Iri) -> bool {
		self.as_iri() == Some(other)
	}
}

impl PartialEq<Term> for Id {
	#[inline]
	fn eq(&self, term: &Term) -> bool {
		match term {
			Term::Id(Lenient::Valid(prop)) => self == prop,
			_ => false,
		}
	}
}

impl PartialEq<Lenient<Id>> for Term {
	#[inline]
	fn eq(&self, r: &Lenient<Id>) -> bool {
		match self {
			Term::Id(prop) => prop == r,
			_ => false,
		}
	}
}

impl TryFrom<Term> for Lenient<Id> {
	type Error = Term;

	#[inline]
	fn try_from(term: Term) -> Result<Self, Term> {
		match term {
			Term::Id(prop) => Ok(prop),
			term => Err(term),
		}
	}
}

impl TryFrom<Lenient<Id>> for Id {
	type Error = String;

	fn try_from(r: Lenient<Id>) -> Result<Self, Self::Error> {
		match r {
			Lenient::Valid(r) => Ok(r),
			Lenient::Invalid(id) => Err(id),
		}
	}
}

impl<'a> TryFrom<&'a Lenient<Id>> for &'a Id {
	type Error = &'a String;

	fn try_from(r: &'a Lenient<Id>) -> Result<Self, Self::Error> {
		match r {
			Lenient::Valid(r) => Ok(r),
			Lenient::Invalid(id) => Err(id),
		}
	}
}

impl<'a> TryFrom<&'a mut Lenient<Id>> for &'a mut Id {
	type Error = &'a mut String;

	fn try_from(r: &'a mut Lenient<Id>) -> Result<Self, Self::Error> {
		match r {
			Lenient::Valid(r) => Ok(r),
			Lenient::Invalid(id) => Err(id),
		}
	}
}

impl Equivalent<Lenient<Id>> for Iri {
	fn equivalent(&self, key: &Lenient<Id>) -> bool {
		match key.as_iri() {
			Some(iri) => self == iri,
			None => false,
		}
	}
}

impl Comparable<Lenient<Id>> for Iri {
	fn compare(&self, key: &Lenient<Id>) -> Ordering {
		match key {
			Lenient::Valid(Id::Iri(iri)) => self.cmp(iri.as_iri()),
			Lenient::Valid(Id::BlankId(_)) => Ordering::Greater,
			Lenient::Invalid(_) => Ordering::Less,
		}
	}
}
