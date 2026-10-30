use btree_indexmap::{Comparable, Equivalent};
use rdf_syntax::{BlankId, BlankIdBuf, Id};
use rdf_syntax::{Iri, IriBuf};
use std::cmp::Ordering;
use std::convert::TryFrom;

use crate::{JsonLdLenient, JsonLdTerm, JsonLdValidate};

impl JsonLdValidate for Id {
	type Invalid = String;
}

impl JsonLdLenient<Id> {
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
	pub fn into_term(self) -> JsonLdTerm {
		JsonLdTerm::Id(self)
	}
}

impl Equivalent<JsonLdLenient<Id>> for Id {
	fn equivalent(&self, key: &JsonLdLenient<Id>) -> bool {
		match key {
			JsonLdLenient::Valid(id) => self == id,
			_ => false,
		}
	}
}

impl Comparable<JsonLdLenient<Id>> for Id {
	fn compare(&self, key: &JsonLdLenient<Id>) -> Ordering {
		match key {
			JsonLdLenient::Valid(id) => self.cmp(id),
			JsonLdLenient::Invalid(_) => Ordering::Less, // Valid < Invalid
		}
	}
}

impl Equivalent<JsonLdLenient<Id>> for &Iri {
	fn equivalent(&self, key: &JsonLdLenient<Id>) -> bool {
		match key {
			JsonLdLenient::Valid(Id::Iri(iri)) => *self == iri,
			_ => false,
		}
	}
}

impl Comparable<JsonLdLenient<Id>> for &Iri {
	fn compare(&self, key: &JsonLdLenient<Id>) -> Ordering {
		match key {
			JsonLdLenient::Valid(Id::Iri(iri)) => (*self).cmp(iri.as_iri()),
			JsonLdLenient::Valid(Id::BlankId(_)) => Ordering::Greater, // Iri > BlankId
			JsonLdLenient::Invalid(_) => Ordering::Less,
		}
	}
}

impl Equivalent<JsonLdLenient<Id>> for rdf_syntax::IriBuf {
	fn equivalent(&self, key: &JsonLdLenient<Id>) -> bool {
		match key {
			JsonLdLenient::Valid(Id::Iri(iri)) => self == iri,
			_ => false,
		}
	}
}

impl Comparable<JsonLdLenient<Id>> for rdf_syntax::IriBuf {
	fn compare(&self, key: &JsonLdLenient<Id>) -> Ordering {
		match key {
			JsonLdLenient::Valid(Id::Iri(iri)) => self.as_iri().cmp(iri.as_iri()),
			JsonLdLenient::Valid(Id::BlankId(_)) => Ordering::Greater,
			JsonLdLenient::Invalid(_) => Ordering::Less,
		}
	}
}

impl Equivalent<JsonLdLenient<Id>> for rdf_syntax::BlankId {
	fn equivalent(&self, key: &JsonLdLenient<Id>) -> bool {
		match key {
			JsonLdLenient::Valid(Id::BlankId(b)) => self == b,
			_ => false,
		}
	}
}

impl Comparable<JsonLdLenient<Id>> for rdf_syntax::BlankId {
	fn compare(&self, key: &JsonLdLenient<Id>) -> Ordering {
		match key {
			JsonLdLenient::Valid(Id::BlankId(b)) => self.cmp(b),
			JsonLdLenient::Valid(Id::Iri(_)) => Ordering::Less, // BlankId < Iri
			JsonLdLenient::Invalid(_) => Ordering::Less,
		}
	}
}

impl Equivalent<JsonLdLenient<Id>> for rdf_syntax::BlankIdBuf {
	fn equivalent(&self, key: &JsonLdLenient<Id>) -> bool {
		match key {
			JsonLdLenient::Valid(Id::BlankId(b)) => self == b,
			_ => false,
		}
	}
}

impl Comparable<JsonLdLenient<Id>> for rdf_syntax::BlankIdBuf {
	fn compare(&self, key: &JsonLdLenient<Id>) -> Ordering {
		match key {
			JsonLdLenient::Valid(Id::BlankId(b)) => self.as_blank_id().cmp(b),
			JsonLdLenient::Valid(Id::Iri(_)) => Ordering::Less,
			JsonLdLenient::Invalid(_) => Ordering::Less,
		}
	}
}

impl From<IriBuf> for JsonLdLenient<Id> {
	#[inline(always)]
	fn from(iri: IriBuf) -> Self {
		Self::iri(iri)
	}
}

impl PartialEq<Iri> for JsonLdLenient<Id> {
	fn eq(&self, other: &Iri) -> bool {
		self.as_iri() == Some(other)
	}
}

impl PartialEq<JsonLdTerm> for Id {
	#[inline]
	fn eq(&self, term: &JsonLdTerm) -> bool {
		match term {
			JsonLdTerm::Id(JsonLdLenient::Valid(prop)) => self == prop,
			_ => false,
		}
	}
}

impl PartialEq<JsonLdLenient<Id>> for JsonLdTerm {
	#[inline]
	fn eq(&self, r: &JsonLdLenient<Id>) -> bool {
		match self {
			JsonLdTerm::Id(prop) => prop == r,
			_ => false,
		}
	}
}

impl TryFrom<JsonLdTerm> for JsonLdLenient<Id> {
	type Error = JsonLdTerm;

	#[inline]
	fn try_from(term: JsonLdTerm) -> Result<Self, JsonLdTerm> {
		match term {
			JsonLdTerm::Id(prop) => Ok(prop),
			term => Err(term),
		}
	}
}

impl TryFrom<JsonLdLenient<Id>> for Id {
	type Error = String;

	fn try_from(r: JsonLdLenient<Id>) -> Result<Self, Self::Error> {
		match r {
			JsonLdLenient::Valid(r) => Ok(r),
			JsonLdLenient::Invalid(id) => Err(id),
		}
	}
}

impl<'a> TryFrom<&'a JsonLdLenient<Id>> for &'a Id {
	type Error = &'a String;

	fn try_from(r: &'a JsonLdLenient<Id>) -> Result<Self, Self::Error> {
		match r {
			JsonLdLenient::Valid(r) => Ok(r),
			JsonLdLenient::Invalid(id) => Err(id),
		}
	}
}

impl<'a> TryFrom<&'a mut JsonLdLenient<Id>> for &'a mut Id {
	type Error = &'a mut String;

	fn try_from(r: &'a mut JsonLdLenient<Id>) -> Result<Self, Self::Error> {
		match r {
			JsonLdLenient::Valid(r) => Ok(r),
			JsonLdLenient::Invalid(id) => Err(id),
		}
	}
}

impl Equivalent<JsonLdLenient<Id>> for Iri {
	fn equivalent(&self, key: &JsonLdLenient<Id>) -> bool {
		match key.as_iri() {
			Some(iri) => self == iri,
			None => false,
		}
	}
}

impl Comparable<JsonLdLenient<Id>> for Iri {
	fn compare(&self, key: &JsonLdLenient<Id>) -> Ordering {
		match key {
			JsonLdLenient::Valid(Id::Iri(iri)) => self.cmp(iri.as_iri()),
			JsonLdLenient::Valid(Id::BlankId(_)) => Ordering::Greater,
			JsonLdLenient::Invalid(_) => Ordering::Less,
		}
	}
}
