use rdf_syntax::{BlankId, BlankIdBuf};
use rdf_syntax::{Id, Iri, IriBuf};

use crate::Lenient;
use std::fmt;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type {
	Json,
	Id(IriBuf),
	Blank(BlankIdBuf),
	Invalid(String),
}

impl Type {
	pub fn from_value_type(value_ty: super::value::ValueType) -> Self {
		match value_ty {
			super::value::ValueType::Json => Self::Json,
			super::value::ValueType::Id(id) => Self::Id(id),
		}
	}

	pub fn from_id(r: Lenient<Id>) -> Self {
		match r {
			Lenient::Valid(Id::Iri(id)) => Self::Id(id),
			Lenient::Valid(Id::BlankId(id)) => Self::Blank(id),
			Lenient::Invalid(id) => Self::Invalid(id),
		}
	}

	pub fn into_id(self) -> Result<Lenient<Id>, Self> {
		match self {
			Type::Id(id) => Ok(Lenient::Valid(Id::Iri(id))),
			Type::Blank(id) => Ok(Lenient::Valid(Id::BlankId(id))),
			Type::Invalid(id) => Ok(Lenient::Invalid(id)),
			typ => Err(typ),
		}
	}
}

impl Type {
	pub fn as_iri(&self) -> Option<&Iri> {
		match self {
			Self::Id(id) => Some(id),
			_ => None,
		}
	}
}

impl Type {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Json => "@json",
			Self::Id(id) => id.as_ref(),
			Self::Blank(id) => id.as_ref(),
			Self::Invalid(id) => id,
		}
	}
}

impl<'a> From<&'a Type> for TypeRef<'a> {
	fn from(t: &'a Type) -> TypeRef<'a> {
		match t {
			Type::Json => Self::Json,
			Type::Id(id) => Self::Id(id),
			Type::Blank(id) => Self::Blank(id),
			Type::Invalid(id) => Self::Invalid(id),
		}
	}
}

impl fmt::Display for Type {
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
pub enum TypeRef<'a> {
	Json,
	Id(&'a Iri),
	Blank(&'a BlankId),
	Invalid(&'a str),
}

impl<'a> TypeRef<'a> {
	pub fn from_value_type(value_ty: super::value::ValueTypeRef<'a>) -> Self {
		match value_ty {
			super::value::ValueTypeRef::Json => Self::Json,
			super::value::ValueTypeRef::Id(id) => Self::Id(id),
		}
	}

	pub fn from_id(r: &'a Lenient<Id>) -> Self {
		match r {
			Lenient::Valid(Id::Iri(id)) => Self::Id(id),
			Lenient::Valid(Id::BlankId(id)) => Self::Blank(id),
			Lenient::Invalid(id) => Self::Invalid(id),
		}
	}

	pub fn to_owned(self) -> Type {
		match self {
			Self::Json => Type::Json,
			Self::Id(id) => Type::Id(id.to_owned()),
			Self::Blank(id) => Type::Blank(id.to_owned()),
			Self::Invalid(id) => Type::Invalid(id.to_owned()),
		}
	}
}

impl<'a> TypeRef<'a> {
	pub fn as_iri(&self) -> Option<&'a Iri> {
		match self {
			Self::Id(id) => Some(id),
			_ => None,
		}
	}
}

impl<'a> TypeRef<'a> {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Json => "@json",
			Self::Id(id) => id.as_ref(),
			Self::Blank(id) => id.as_ref(),
			Self::Invalid(id) => id,
		}
	}
}

impl<'a> PartialEq<Type> for TypeRef<'a> {
	fn eq(&self, other: &Type) -> bool {
		let other_ref: TypeRef = other.into();
		*self == other_ref
	}
}

impl<'a> PartialEq<TypeRef<'a>> for Type {
	fn eq(&self, other: &TypeRef<'a>) -> bool {
		let self_ref: TypeRef = self.into();
		self_ref == *other
	}
}

impl<'a> fmt::Display for TypeRef<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Json => write!(f, "@json"),
			Self::Id(id) => id.fmt(f),
			Self::Blank(id) => id.fmt(f),
			Self::Invalid(id) => id.fmt(f),
		}
	}
}
