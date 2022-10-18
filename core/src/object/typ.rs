use crate::{Id, ValidId};
use std::fmt;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type<T, B> {
	Json,
	Id(T),
	Blank(B),
	Invalid(String),
}

impl<T, B> Type<T, B> {
	pub fn from_value_type(value_ty: super::value::Type<T>) -> Self {
		match value_ty {
			super::value::Type::Json => Self::Json,
			super::value::Type::Id(id) => Self::Id(id),
		}
	}

	pub fn from_reference(r: Id<T, B>) -> Self {
		match r {
			Id::Valid(ValidId::Iri(id)) => Self::Id(id),
			Id::Valid(ValidId::Blank(id)) => Self::Blank(id),
			Id::Invalid(id) => Self::Invalid(id),
		}
	}

	pub fn into_reference(self) -> Result<Id<T, B>, Self> {
		match self {
			Type::Id(id) => Ok(Id::Valid(ValidId::Iri(id))),
			Type::Blank(id) => Ok(Id::Valid(ValidId::Blank(id))),
			Type::Invalid(id) => Ok(Id::Invalid(id)),
			typ => Err(typ),
		}
	}
}

impl<T, B> Type<T, B> {
	pub fn as_iri(&self) -> Option<&T> {
		match self {
			Self::Id(id) => Some(id),
			_ => None,
		}
	}
}

impl<T: AsRef<str>, B: AsRef<str>> Type<T, B> {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Json => "@json",
			Self::Id(id) => id.as_ref(),
			Self::Blank(id) => id.as_ref(),
			Self::Invalid(id) => id,
		}
	}
}

impl<'a, T, B> From<&'a Type<T, B>> for TypeRef<'a, T, B> {
	fn from(t: &'a Type<T, B>) -> TypeRef<'a, T, B> {
		match t {
			Type::Json => Self::Json,
			Type::Id(id) => Self::Id(id),
			Type::Blank(id) => Self::Blank(id),
			Type::Invalid(id) => Self::Invalid(id),
		}
	}
}

impl<T: fmt::Display, B: fmt::Display> fmt::Display for Type<T, B> {
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
pub enum TypeRef<'a, T, B> {
	Json,
	Id(&'a T),
	Blank(&'a B),
	Invalid(&'a str),
}

impl<'a, T, B> TypeRef<'a, T, B> {
	pub fn from_value_type(value_ty: super::value::TypeRef<'a, T>) -> Self {
		match value_ty {
			super::value::TypeRef::Json => Self::Json,
			super::value::TypeRef::Id(id) => Self::Id(id),
		}
	}

	pub fn from_reference(r: &'a Id<T, B>) -> Self {
		match r {
			Id::Valid(ValidId::Iri(id)) => Self::Id(id),
			Id::Valid(ValidId::Blank(id)) => Self::Blank(id),
			Id::Invalid(id) => Self::Invalid(id),
		}
	}

	pub fn cloned(self) -> Type<T, B>
	where
		T: Clone,
		B: Clone,
	{
		match self {
			Self::Json => Type::Json,
			Self::Id(id) => Type::Id(id.clone()),
			Self::Blank(id) => Type::Blank(id.clone()),
			Self::Invalid(id) => Type::Invalid(id.to_string()),
		}
	}
}

impl<'a, T, B> TypeRef<'a, T, B> {
	pub fn as_iri(&self) -> Option<&'a T> {
		match self {
			Self::Id(id) => Some(id),
			_ => None,
		}
	}
}

impl<'a, T: AsRef<str>, B: AsRef<str>> TypeRef<'a, T, B> {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Json => "@json",
			Self::Id(id) => id.as_ref(),
			Self::Blank(id) => id.as_ref(),
			Self::Invalid(id) => id,
		}
	}
}

impl<'a, T: PartialEq, B: PartialEq> PartialEq<Type<T, B>> for TypeRef<'a, T, B> {
	fn eq(&self, other: &Type<T, B>) -> bool {
		let other_ref: TypeRef<T, B> = other.into();
		*self == other_ref
	}
}

impl<'a, T: PartialEq, B: PartialEq> PartialEq<TypeRef<'a, T, B>> for Type<T, B> {
	fn eq(&self, other: &TypeRef<'a, T, B>) -> bool {
		let self_ref: TypeRef<T, B> = self.into();
		self_ref == *other
	}
}

impl<'a, T: fmt::Display, B: fmt::Display> fmt::Display for TypeRef<'a, T, B> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Json => write!(f, "@json"),
			Self::Id(id) => id.fmt(f),
			Self::Blank(id) => id.fmt(f),
			Self::Invalid(id) => id.fmt(f),
		}
	}
}
