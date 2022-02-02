use crate::{
	lang::LenientLanguageTag,
	object,
	syntax::{self, Keyword},
	util::{self, AsAnyJson},
	Direction, Id, LangString,
};
use cc_traits::MapInsert;
use derivative::Derivative;
use generic_json::{Json, JsonClone, JsonHash};
use iref::IriBuf;
use std::{
	fmt,
	hash::{Hash, Hasher},
};

/// Value type.
pub enum Type<T> {
	Json,
	Id(T),
}

impl<T> Type<T> {
	pub fn as_reference(&self) -> Option<crate::reference::Ref<T>> {
		match self {
			Self::Json => None,
			Self::Id(t) => Some(crate::reference::Ref::Id(t)),
		}
	}
}

/// Value type reference.
pub enum TypeRef<'a, T> {
	Json,
	Id(&'a T),
}

impl<'a, T> TypeRef<'a, T> {
	pub fn as_syntax_type(&self) -> syntax::Type<&'a T> {
		match self {
			Self::Json => syntax::Type::Json,
			Self::Id(id) => syntax::Type::Ref(id),
		}
	}

	pub fn into_reference(self) -> Option<crate::reference::Ref<'a, T>> {
		match self {
			Self::Json => None,
			Self::Id(t) => Some(crate::reference::Ref::Id(t)),
		}
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound = "J::String: Clone"))]
pub enum LiteralString<J: Json> {
	/// Literal string expanded from a JSON-LD document.
	Expanded(J::String),

	/// Literal string inferred during expansion.
	Inferred(String),
}

impl<J: Json> LiteralString<J> {
	#[inline(always)]
	pub fn as_str(&self) -> &str {
		match self {
			Self::Expanded(s) => s.as_ref(),
			Self::Inferred(s) => s.as_str(),
		}
	}
}

impl<J: Json> AsRef<str> for LiteralString<J> {
	#[inline(always)]
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl<J: Json> std::borrow::Borrow<str> for LiteralString<J> {
	#[inline(always)]
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl<J: Json> std::ops::Deref for LiteralString<J> {
	type Target = str;

	#[inline(always)]
	fn deref(&self) -> &str {
		self.as_str()
	}
}

impl<J: Json, K: Json> PartialEq<LiteralString<K>> for LiteralString<J> {
	#[inline(always)]
	fn eq(&self, other: &LiteralString<K>) -> bool {
		self.as_str() == other.as_str()
	}
}

impl<J: Json> Eq for LiteralString<J> {}

impl<J: Json> Hash for LiteralString<J> {
	#[inline(always)]
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.as_str().hash(h)
	}
}

impl<J: Json> fmt::Debug for LiteralString<J> {
	#[inline(always)]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

/// Literal value.
#[derive(Derivative)]
#[derivative(Clone(bound = "J::Number: Clone, J::String: Clone"))]
pub enum Literal<J: Json> {
	/// The `null` value.
	Null,

	/// Boolean value.
	Boolean(bool),

	/// Number.
	Number(J::Number),

	/// String.
	String(LiteralString<J>),
}

impl<J: Json> PartialEq for Literal<J> {
	#[inline(always)]
	fn eq(&self, other: &Self) -> bool {
		use Literal::*;
		match (self, other) {
			(Null, Null) => true,
			(Boolean(a), Boolean(b)) => a == b,
			(Number(a), Number(b)) => a == b,
			(String(a), String(b)) => a == b,
			_ => false,
		}
	}
}

impl<J: Json> Eq for Literal<J> {}

impl<J: JsonHash> Hash for Literal<J> {
	#[inline(always)]
	fn hash<H: Hasher>(&self, h: &mut H) {
		match self {
			Literal::Null => (),
			Literal::Boolean(b) => b.hash(h),
			Literal::Number(n) => n.hash(h),
			Literal::String(s) => s.hash(h),
		}
	}
}

impl<J: Json> Literal<J> {
	/// Returns this value as a string if it is one.
	#[inline(always)]
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Literal::String(s) => Some(s.as_ref()),
			_ => None,
		}
	}

	/// Returns this value as a boolean if it is one.
	#[inline(always)]
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			Literal::Boolean(b) => Some(*b),
			_ => None,
		}
	}

	/// Returns this value as a number if it is one.
	#[inline(always)]
	pub fn as_number(&self) -> Option<&J::Number> {
		match self {
			Literal::Number(n) => Some(n),
			_ => None,
		}
	}
}

/// Value object.
///
/// Either a typed literal value, or an internationalized language string.
#[derive(PartialEq, Eq)]
pub enum Value<J: Json, T: Id = IriBuf> {
	/// Typed literal value.
	Literal(Literal<J>, Option<T>),

	/// Language tagged string.
	LangString(LangString<J>),

	/// JSON literal value.
	Json(J),
}

impl<J: JsonClone, T: Id> Clone for Value<J, T> {
	#[inline(always)]
	fn clone(&self) -> Self {
		match self {
			Self::Literal(l, t) => Self::Literal(l.clone(), t.clone()),
			Self::LangString(s) => Self::LangString(s.clone()),
			Self::Json(j) => Self::Json(j.clone()),
		}
	}
}

impl<J: Json, T: Id> Value<J, T> {
	#[inline(always)]
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Value::Literal(lit, _) => lit.as_str(),
			Value::LangString(str) => Some(str.as_str()),
			Value::Json(_) => None,
		}
	}

	#[inline(always)]
	pub fn as_literal(&self) -> Option<(&Literal<J>, Option<&T>)> {
		match self {
			Self::Literal(lit, ty) => Some((lit, ty.as_ref())),
			_ => None,
		}
	}

	pub fn literal_type(&self) -> Option<&T> {
		match self {
			Self::Literal(_, ty) => ty.as_ref(),
			_ => None,
		}
	}

	/// Set the literal value type, and returns the old type.
	///
	/// Has no effect and return `None` if the value is not a literal value.
	pub fn set_literal_type(&mut self, mut ty: Option<T>) -> Option<T> {
		match self {
			Self::Literal(_, old_ty) => {
				std::mem::swap(old_ty, &mut ty);
				ty
			}
			_ => None,
		}
	}

	/// Maps the literal value type.
	///
	/// Has no effect if the value is not a literal value.
	pub fn map_literal_type<F: FnOnce(Option<T>) -> Option<T>>(&mut self, f: F) {
		if let Self::Literal(_, ty) = self {
			*ty = f(ty.take())
		}
	}

	#[inline(always)]
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			Value::Literal(lit, _) => lit.as_bool(),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn as_number(&self) -> Option<&J::Number> {
		match self {
			Value::Literal(lit, _) => lit.as_number(),
			_ => None,
		}
	}

	/// Return the type of the value if any.
	///
	/// This will return `Some(Type::Json)` for JSON literal values.
	pub fn typ(&self) -> Option<TypeRef<T>> {
		match self {
			Value::Literal(_, Some(ty)) => Some(TypeRef::Id(ty)),
			Value::Json(_) => Some(TypeRef::Json),
			_ => None,
		}
	}

	/// If the value is a language tagged string, return its associated language if any.
	///
	/// Returns `None` if the value is not a language tagged string.
	#[inline(always)]
	pub fn language(&self) -> Option<LenientLanguageTag> {
		match self {
			Value::LangString(tag) => tag.language(),
			_ => None,
		}
	}

	/// If the value is a language tagged string, return its associated direction if any.
	///
	/// Returns `None` if the value is not a language tagged string.
	#[inline(always)]
	pub fn direction(&self) -> Option<Direction> {
		match self {
			Value::LangString(str) => str.direction(),
			_ => None,
		}
	}
}

impl<J: JsonHash, T: Id> object::Any<J, T> for Value<J, T> {
	#[inline(always)]
	fn as_ref(&self) -> object::Ref<J, T> {
		object::Ref::Value(self)
	}
}

impl<J: JsonHash, T: Id> Hash for Value<J, T> {
	#[inline]
	fn hash<H: Hasher>(&self, h: &mut H) {
		match self {
			Value::Literal(lit, ty) => {
				lit.hash(h);
				ty.hash(h);
			}
			Value::LangString(s) => s.hash(h),
			Value::Json(json) => crate::util::hash_json(json, h), // TODO replace by the hash function provided by J whenever possible.
		}
	}
}

impl<J: JsonClone, K: util::JsonFrom<J>, T: Id> util::AsJson<J, K> for Value<J, T> {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		let mut obj = K::Object::default();

		match self {
			Value::Literal(lit, ty) => {
				match lit {
					Literal::Null => obj.insert(
						K::new_key(Keyword::Value.into_str(), meta(None)),
						K::null(meta(None)),
					),
					Literal::Boolean(b) => obj.insert(
						K::new_key(Keyword::Value.into_str(), meta(None)),
						b.as_json_with(meta(None)),
					),
					Literal::Number(n) => obj.insert(
						K::new_key(Keyword::Value.into_str(), meta(None)),
						K::number(n.clone().into(), meta(None)),
					),
					Literal::String(s) => obj.insert(
						K::new_key(Keyword::Value.into_str(), meta(None)),
						s.as_json_with(meta(None)),
					),
				};

				if let Some(ty) = ty {
					obj.insert(
						K::new_key(Keyword::Type.into_str(), meta(None)),
						ty.as_json(meta(None)),
					);
				}
			}
			Value::LangString(str) => {
				obj.insert(
					K::new_key(Keyword::Value.into_str(), meta(None)),
					str.as_str().as_json_with(meta(None)),
				);

				if let Some(language) = str.language() {
					obj.insert(
						K::new_key(Keyword::Language.into_str(), meta(None)),
						language.as_json_with(meta(None)),
					);
				}

				if let Some(direction) = str.direction() {
					obj.insert(
						K::new_key(Keyword::Direction.into_str(), meta(None)),
						direction.as_json_with(meta(None)),
					);
				}
			}
			Value::Json(json) => {
				obj.insert(
					K::new_key(Keyword::Value.into_str(), meta(None)),
					json.as_json_with(meta.clone()),
				);
				obj.insert(
					K::new_key(Keyword::Type.into_str(), meta(None)),
					Keyword::Json.as_json_with(meta(None)),
				);
			}
		}

		K::object(obj, meta(None))
	}
}
