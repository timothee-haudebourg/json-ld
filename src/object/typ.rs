use crate::{BlankId, Id, Reference};
use iref::Iri;
use std::fmt;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type<T> {
	Json,
	Id(T),
	Blank(BlankId),
	Invalid(String),
}

impl<T> Type<T> {
	pub fn from_value_type(value_ty: super::value::Type<T>) -> Self {
		match value_ty {
			super::value::Type::Json => Self::Json,
			super::value::Type::Id(id) => Self::Id(id),
		}
	}

	pub fn from_reference(r: Reference<T>) -> Self {
		match r {
			Reference::Id(id) => Self::Id(id),
			Reference::Blank(id) => Self::Blank(id),
			Reference::Invalid(id) => Self::Invalid(id),
		}
	}

	pub fn into_reference(self) -> Result<Reference<T>, Type<T>> {
		match self {
			Type::Id(id) => Ok(Reference::Id(id)),
			Type::Blank(id) => Ok(Reference::Blank(id)),
			Type::Invalid(id) => Ok(Reference::Invalid(id)),
			typ => Err(typ),
		}
	}
}

impl<T: Id> Type<T> {
	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Self::Id(id) => Some(id.as_iri()),
			_ => None,
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Self::Json => "@json",
			Self::Id(id) => id.as_iri().into_str(),
			Self::Blank(id) => id.as_str(),
			Self::Invalid(id) => id,
		}
	}
}

impl<'a, T> From<&'a Type<T>> for TypeRef<'a, T> {
	fn from(t: &'a Type<T>) -> TypeRef<'a, T> {
		match t {
			Type::Json => Self::Json,
			Type::Id(id) => Self::Id(id),
			Type::Blank(id) => Self::Blank(id),
			Type::Invalid(id) => Self::Invalid(id),
		}
	}
}

impl<T: Id + fmt::Display> fmt::Display for Type<T> {
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
pub enum TypeRef<'a, T> {
	Json,
	Id(&'a T),
	Blank(&'a BlankId),
	Invalid(&'a str),
}

impl<'a, T> TypeRef<'a, T> {
	pub fn from_value_type(value_ty: super::value::TypeRef<'a, T>) -> Self {
		match value_ty {
			super::value::TypeRef::Json => Self::Json,
			super::value::TypeRef::Id(id) => Self::Id(id),
		}
	}

	pub fn from_reference(r: &'a Reference<T>) -> Self {
		match r {
			Reference::Id(id) => Self::Id(id),
			Reference::Blank(id) => Self::Blank(id),
			Reference::Invalid(id) => Self::Invalid(id),
		}
	}

	pub fn cloned(self) -> Type<T>
	where
		T: Clone,
	{
		match self {
			Self::Json => Type::Json,
			Self::Id(id) => Type::Id(id.clone()),
			Self::Blank(id) => Type::Blank(id.clone()),
			Self::Invalid(id) => Type::Invalid(id.to_string()),
		}
	}
}

impl<'a, T: Id> TypeRef<'a, T> {
	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Self::Id(id) => Some(id.as_iri()),
			_ => None,
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Self::Json => "@json",
			Self::Id(id) => id.as_iri().into_str(),
			Self::Blank(id) => id.as_str(),
			Self::Invalid(id) => id,
		}
	}
}

impl<'a, T: PartialEq> PartialEq<Type<T>> for TypeRef<'a, T> {
	fn eq(&self, other: &Type<T>) -> bool {
		let other_ref: TypeRef<T> = other.into();
		*self == other_ref
	}
}

impl<'a, T: PartialEq> PartialEq<TypeRef<'a, T>> for Type<T> {
	fn eq(&self, other: &TypeRef<'a, T>) -> bool {
		let self_ref: TypeRef<T> = self.into();
		self_ref == *other
	}
}

impl<'a, T: Id + fmt::Display> fmt::Display for TypeRef<'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Json => write!(f, "@json"),
			Self::Id(id) => id.fmt(f),
			Self::Blank(id) => id.fmt(f),
			Self::Invalid(id) => id.fmt(f),
		}
	}
}
