use crate::{
	object,
	syntax::{Keyword, Type},
	util::{self, AsAnyJson},
	Direction, Id, LangString,
};
use cc_traits::MapInsert;
use derivative::Derivative;
use generic_json::{Json, JsonClone, JsonHash};
use iref::IriBuf;
use langtag::LanguageTag;
use std::{
	fmt,
	hash::{Hash, Hasher},
};

#[derive(Derivative)]
#[derivative(Clone(bound = "J::String: Clone"))]
pub enum LiteralString<J: Json> {
	/// Literal string expanded from a JSON-LD document.
	Expanded(J::String),

	/// Literal string inferred during expansion.
	Inferred(String),
}

impl<J: Json> LiteralString<J> {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Expanded(s) => s.as_ref(),
			Self::Inferred(s) => s.as_str(),
		}
	}
}

impl<J: Json> AsRef<str> for LiteralString<J> {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl<J: Json> std::borrow::Borrow<str> for LiteralString<J> {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl<J: Json> std::ops::Deref for LiteralString<J> {
	type Target = str;

	fn deref(&self) -> &str {
		self.as_str()
	}
}

impl<J: Json, K: Json> PartialEq<LiteralString<K>> for LiteralString<J> {
	fn eq(&self, other: &LiteralString<K>) -> bool {
		self.as_str() == other.as_str()
	}
}

impl<J: Json> Eq for LiteralString<J> {}

impl<J: Json> Hash for LiteralString<J> {
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.as_str().hash(h)
	}
}

impl<J: Json> fmt::Debug for LiteralString<J> {
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
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Literal::String(s) => Some(s.as_ref()),
			_ => None,
		}
	}

	pub fn as_bool(&self) -> Option<bool> {
		match self {
			Literal::Boolean(b) => Some(*b),
			_ => None,
		}
	}

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
	/// A typed value.
	Literal(Literal<J>, Option<T>),

	/// A language tagged string.
	LangString(LangString<J>),

	/// A JSON literal value.
	Json(J),
}

impl<J: JsonClone, T: Id> Clone for Value<J, T> {
	fn clone(&self) -> Self {
		match self {
			Self::Literal(l, t) => Self::Literal(l.clone(), t.clone()),
			Self::LangString(s) => Self::LangString(s.clone()),
			Self::Json(j) => Self::Json(j.clone()),
		}
	}
}

impl<J: Json, T: Id> Value<J, T> {
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Value::Literal(lit, _) => lit.as_str(),
			Value::LangString(str) => Some(str.as_str()),
			Value::Json(_) => None,
		}
	}

	pub fn as_bool(&self) -> Option<bool> {
		match self {
			Value::Literal(lit, _) => lit.as_bool(),
			_ => None,
		}
	}

	pub fn as_number(&self) -> Option<&J::Number> {
		match self {
			Value::Literal(lit, _) => lit.as_number(),
			_ => None,
		}
	}

	/// Return the type of the value if any.
	///
	/// This will return `Some(Type::Json)` for JSON literal values.
	pub fn typ(&self) -> Option<Type<&T>> {
		match self {
			Value::Literal(_, Some(ty)) => Some(Type::Ref(ty)),
			Value::Json(_) => Some(Type::Json),
			_ => None,
		}
	}

	/// If the value is a language tagged string, return its associated language if any.
	///
	/// Returns `None` if the value is not a language tagged string.
	pub fn language(&self) -> Option<LanguageTag> {
		match self {
			Value::LangString(tag) => tag.language(),
			_ => None,
		}
	}

	/// If the value is a language tagged string, return its associated direction if any.
	///
	/// Returns `None` if the value is not a language tagged string.
	pub fn direction(&self) -> Option<Direction> {
		match self {
			Value::LangString(str) => str.direction(),
			_ => None,
		}
	}
}

impl<J: JsonHash, T: Id> object::Any<J, T> for Value<J, T> {
	fn as_ref(&self) -> object::Ref<J, T> {
		object::Ref::Value(self)
	}
}

impl<J: JsonHash, T: Id> Hash for Value<J, T> {
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
					json.as_json_with(meta.clone())
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
