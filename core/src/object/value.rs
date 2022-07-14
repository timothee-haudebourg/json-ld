use crate::{object, Direction, Id, LangString, LenientLanguageTag};
use iref::IriBuf;
use json_syntax::{Number, NumberBuf};
use locspan_derive::*;
use std::{
	cmp::Ordering,
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
	pub fn as_syntax_type(&self) -> crate::Type<&'a T> {
		match self {
			Self::Json => crate::Type::Json,
			Self::Id(id) => crate::Type::Ref(id),
		}
	}

	pub fn into_reference(self) -> Option<crate::reference::Ref<'a, T>> {
		match self {
			Self::Json => None,
			Self::Id(t) => Some(crate::reference::Ref::Id(t)),
		}
	}
}

#[derive(Clone)]
pub enum LiteralString {
	/// Literal string expanded from a JSON-LD document.
	Expanded(json_syntax::String),

	/// Literal string inferred during expansion.
	Inferred(String),
}

impl LiteralString {
	#[inline(always)]
	pub fn as_str(&self) -> &str {
		match self {
			Self::Expanded(s) => s.as_ref(),
			Self::Inferred(s) => s.as_str(),
		}
	}
}

impl AsRef<str> for LiteralString {
	#[inline(always)]
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl std::borrow::Borrow<str> for LiteralString {
	#[inline(always)]
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl std::ops::Deref for LiteralString {
	type Target = str;

	#[inline(always)]
	fn deref(&self) -> &str {
		self.as_str()
	}
}

impl PartialEq for LiteralString {
	#[inline(always)]
	fn eq(&self, other: &LiteralString) -> bool {
		self.as_str() == other.as_str()
	}
}

impl Eq for LiteralString {}

impl Hash for LiteralString {
	#[inline(always)]
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.as_str().hash(h)
	}
}

impl PartialOrd for LiteralString {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.as_str().partial_cmp(other.as_str())
	}
}

impl Ord for LiteralString {
	fn cmp(&self, other: &Self) -> Ordering {
		self.as_str().cmp(other.as_str())
	}
}

impl fmt::Debug for LiteralString {
	#[inline(always)]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

/// Literal value.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Literal {
	/// The `null` value.
	Null,

	/// Boolean value.
	Boolean(bool),

	/// Number.
	Number(NumberBuf),

	/// String.
	String(LiteralString),
}

impl Literal {
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
	pub fn as_number(&self) -> Option<&Number> {
		match self {
			Literal::Number(n) => Some(n),
			_ => None,
		}
	}
}

/// Value object.
///
/// Either a typed literal value, or an internationalized language string.
#[derive(
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	StrippedPartialEq,
	StrippedEq,
	StrippedPartialOrd,
	StrippedOrd,
	StrippedHash,
)]
#[stripped_ignore(M)]
#[stripped(T)]
pub enum Value<T = IriBuf, M = ()> {
	/// Typed literal value.
	Literal(#[stripped] Literal, #[stripped] Option<T>),

	/// Language tagged string.
	LangString(#[stripped] LangString),

	/// JSON literal value.
	Json(json_syntax::Value<M>),
}

impl<T: Id, M> Value<T, M> {
	#[inline(always)]
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Value::Literal(lit, _) => lit.as_str(),
			Value::LangString(str) => Some(str.as_str()),
			Value::Json(_) => None,
		}
	}

	#[inline(always)]
	pub fn as_literal(&self) -> Option<(&Literal, Option<&T>)> {
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
	pub fn as_number(&self) -> Option<&Number> {
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

impl<T: Id, M> object::Any<T, M> for Value<T, M> {
	#[inline(always)]
	fn as_ref(&self) -> object::Ref<T, M> {
		object::Ref::Value(self)
	}
}

// impl<J: JsonClone, K: utils::JsonFrom<J>, T: Id> utils::AsJson<J, K> for Value<T> {
// 	fn as_json_with(
// 		&self,
// 		meta: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
// 	) -> K {
// 		let mut obj = <K as Json>::Object::default();

// 		match self {
// 			Value::Literal(lit, ty) => {
// 				match lit {
// 					Literal::Null => obj.insert(
// 						K::new_key(Keyword::Value.into_str(), meta(None)),
// 						K::null(meta(None)),
// 					),
// 					Literal::Boolean(b) => obj.insert(
// 						K::new_key(Keyword::Value.into_str(), meta(None)),
// 						b.as_json_with(meta(None)),
// 					),
// 					Literal::Number(n) => obj.insert(
// 						K::new_key(Keyword::Value.into_str(), meta(None)),
// 						K::number(n.clone().into(), meta(None)),
// 					),
// 					Literal::String(s) => obj.insert(
// 						K::new_key(Keyword::Value.into_str(), meta(None)),
// 						s.as_json_with(meta(None)),
// 					),
// 				};

// 				if let Some(ty) = ty {
// 					obj.insert(
// 						K::new_key(Keyword::Type.into_str(), meta(None)),
// 						ty.as_json(meta(None)),
// 					);
// 				}
// 			}
// 			Value::LangString(str) => {
// 				obj.insert(
// 					K::new_key(Keyword::Value.into_str(), meta(None)),
// 					str.as_str().as_json_with(meta(None)),
// 				);

// 				if let Some(language) = str.language() {
// 					obj.insert(
// 						K::new_key(Keyword::Language.into_str(), meta(None)),
// 						language.as_json_with(meta(None)),
// 					);
// 				}

// 				if let Some(direction) = str.direction() {
// 					obj.insert(
// 						K::new_key(Keyword::Direction.into_str(), meta(None)),
// 						direction.as_json_with(meta(None)),
// 					);
// 				}
// 			}
// 			Value::Json(json) => {
// 				obj.insert(
// 					K::new_key(Keyword::Value.into_str(), meta(None)),
// 					json.as_json_with(meta.clone()),
// 				);
// 				obj.insert(
// 					K::new_key(Keyword::Type.into_str(), meta(None)),
// 					Keyword::Json.as_json_with(meta(None)),
// 				);
// 			}
// 		}

// 		K::object(obj, meta(None))
// 	}
// }
