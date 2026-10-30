use rdf_syntax::{BlankId, BlankIdBuf};
use rdf_syntax::{Id, Iri, IriBuf};

use crate::JsonLdLenient;
use std::fmt;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum JsonLdObjectType {
	Json,
	Id(IriBuf),
	Blank(BlankIdBuf),
	Invalid(String),
}

impl JsonLdObjectType {
	pub fn from_value_type(value_ty: super::value::JsonLdValueType) -> Self {
		match value_ty {
			super::value::JsonLdValueType::Json => Self::Json,
			super::value::JsonLdValueType::Id(id) => Self::Id(id),
		}
	}

	pub fn from_id(r: JsonLdLenient<Id>) -> Self {
		match r {
			JsonLdLenient::Valid(Id::Iri(id)) => Self::Id(id),
			JsonLdLenient::Valid(Id::BlankId(id)) => Self::Blank(id),
			JsonLdLenient::Invalid(id) => Self::Invalid(id),
		}
	}

	pub fn into_id(self) -> Result<JsonLdLenient<Id>, Self> {
		match self {
			JsonLdObjectType::Id(id) => Ok(JsonLdLenient::Valid(Id::Iri(id))),
			JsonLdObjectType::Blank(id) => Ok(JsonLdLenient::Valid(Id::BlankId(id))),
			JsonLdObjectType::Invalid(id) => Ok(JsonLdLenient::Invalid(id)),
			typ => Err(typ),
		}
	}
}

impl JsonLdObjectType {
	pub fn as_iri(&self) -> Option<&Iri> {
		match self {
			Self::Id(id) => Some(id),
			_ => None,
		}
	}
}

impl JsonLdObjectType {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Json => "@json",
			Self::Id(id) => id.as_ref(),
			Self::Blank(id) => id.as_ref(),
			Self::Invalid(id) => id,
		}
	}
}

impl<'a> From<&'a JsonLdObjectType> for JsonLdObjectTypeRef<'a> {
	fn from(t: &'a JsonLdObjectType) -> JsonLdObjectTypeRef<'a> {
		match t {
			JsonLdObjectType::Json => Self::Json,
			JsonLdObjectType::Id(id) => Self::Id(id),
			JsonLdObjectType::Blank(id) => Self::Blank(id),
			JsonLdObjectType::Invalid(id) => Self::Invalid(id),
		}
	}
}

impl fmt::Display for JsonLdObjectType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Json => write!(f, "@json"),
			Self::Id(id) => id.fmt(f),
			Self::Blank(id) => id.fmt(f),
			Self::Invalid(id) => id.fmt(f),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsonLdObjectTypeRef<'a> {
	Json,
	Id(&'a Iri),
	Blank(&'a BlankId),
	Invalid(&'a str),
}

impl<'a> JsonLdObjectTypeRef<'a> {
	pub fn from_value_type(value_ty: super::value::JsonLdValueTypeRef<'a>) -> Self {
		match value_ty {
			super::value::JsonLdValueTypeRef::Json => Self::Json,
			super::value::JsonLdValueTypeRef::Id(id) => Self::Id(id),
		}
	}

	pub fn from_id(r: &'a JsonLdLenient<Id>) -> Self {
		match r {
			JsonLdLenient::Valid(Id::Iri(id)) => Self::Id(id),
			JsonLdLenient::Valid(Id::BlankId(id)) => Self::Blank(id),
			JsonLdLenient::Invalid(id) => Self::Invalid(id),
		}
	}

	pub fn to_owned(self) -> JsonLdObjectType {
		match self {
			Self::Json => JsonLdObjectType::Json,
			Self::Id(id) => JsonLdObjectType::Id(id.to_owned()),
			Self::Blank(id) => JsonLdObjectType::Blank(id.to_owned()),
			Self::Invalid(id) => JsonLdObjectType::Invalid(id.to_owned()),
		}
	}
}

impl<'a> JsonLdObjectTypeRef<'a> {
	pub fn as_iri(&self) -> Option<&'a Iri> {
		match self {
			Self::Id(id) => Some(id),
			_ => None,
		}
	}
}

impl<'a> JsonLdObjectTypeRef<'a> {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Json => "@json",
			Self::Id(id) => id.as_ref(),
			Self::Blank(id) => id.as_ref(),
			Self::Invalid(id) => id,
		}
	}
}

impl<'a> PartialEq<JsonLdObjectType> for JsonLdObjectTypeRef<'a> {
	fn eq(&self, other: &JsonLdObjectType) -> bool {
		let other_ref: JsonLdObjectTypeRef = other.into();
		*self == other_ref
	}
}

impl<'a> PartialEq<JsonLdObjectTypeRef<'a>> for JsonLdObjectType {
	fn eq(&self, other: &JsonLdObjectTypeRef<'a>) -> bool {
		let self_ref: JsonLdObjectTypeRef = self.into();
		self_ref == *other
	}
}

impl<'a> fmt::Display for JsonLdObjectTypeRef<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Json => write!(f, "@json"),
			Self::Id(id) => id.fmt(f),
			Self::Blank(id) => id.fmt(f),
			Self::Invalid(id) => id.fmt(f),
		}
	}
}
