use crate::{
	object,
	syntax::{Keyword, Type},
	util, Direction, Id, LangString,
};
use iref::IriBuf;
use json::JsonValue;
use langtag::LanguageTag;
use std::hash::{Hash, Hasher};

/// Literal value.
#[derive(Clone)]
pub enum Literal {
	/// The `null` value.
	Null,

	/// Boolean value.
	Boolean(bool),

	/// Number.
	Number(json::number::Number),

	/// String.
	String(String),
}

impl PartialEq for Literal {
	fn eq(&self, other: &Literal) -> bool {
		use Literal::*;
		match (self, other) {
			(Null, Null) => true,
			(Boolean(a), Boolean(b)) => a == b,
			(Number(a), Number(b)) => a.as_parts() == b.as_parts(),
			(String(a), String(b)) => a == b,
			_ => false,
		}
	}
}

impl Eq for Literal {}

impl Hash for Literal {
	fn hash<H: Hasher>(&self, h: &mut H) {
		match self {
			Literal::Null => (),
			Literal::Boolean(b) => b.hash(h),
			Literal::Number(n) => util::hash_json_number(n, h),
			Literal::String(s) => s.hash(h),
		}
	}
}

impl Literal {
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Literal::String(s) => Some(s.as_str()),
			_ => None,
		}
	}

	pub fn as_bool(&self) -> Option<bool> {
		match self {
			Literal::Boolean(b) => Some(*b),
			_ => None,
		}
	}

	pub fn as_number(&self) -> Option<json::number::Number> {
		match self {
			Literal::Number(n) => Some(*n),
			_ => None,
		}
	}
}

/// Value object.
///
/// Either a typed literal value, or an internationalized language string.
#[derive(PartialEq, Eq, Clone)]
pub enum Value<T: Id = IriBuf> {
	/// A typed value.
	Literal(Literal, Option<T>),

	/// A language tagged string.
	LangString(LangString),

	/// A JSON literal value.
	Json(JsonValue),
}

impl<T: Id> Value<T> {
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

	pub fn as_number(&self) -> Option<json::number::Number> {
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

impl<T: Id> object::Any<T> for Value<T> {
	fn as_ref(&self) -> object::Ref<T> {
		object::Ref::Value(self)
	}
}

impl<T: Id> Hash for Value<T> {
	fn hash<H: Hasher>(&self, h: &mut H) {
		match self {
			Value::Literal(lit, ty) => {
				lit.hash(h);
				ty.hash(h);
			}
			Value::LangString(str) => str.hash(h),
			Value::Json(json) => util::hash_json(json, h),
		}
	}
}

impl<T: Id> util::AsJson for Value<T> {
	fn as_json(&self) -> JsonValue {
		let mut obj = json::object::Object::new();

		match self {
			Value::Literal(lit, ty) => {
				match lit {
					Literal::Null => obj.insert(Keyword::Value.into(), JsonValue::Null),
					Literal::Boolean(b) => obj.insert(Keyword::Value.into(), b.as_json()),
					Literal::Number(n) => obj.insert(Keyword::Value.into(), JsonValue::Number(*n)),
					Literal::String(s) => obj.insert(Keyword::Value.into(), s.as_json()),
				}

				if let Some(ty) = ty {
					obj.insert(Keyword::Type.into(), ty.as_json())
				}
			}
			Value::LangString(str) => {
				obj.insert(Keyword::Value.into(), str.as_str().into());

				if let Some(language) = str.language() {
					obj.insert(Keyword::Language.into(), language.as_json());
				}

				if let Some(direction) = str.direction() {
					obj.insert(Keyword::Direction.into(), direction.as_json());
				}
			}
			Value::Json(json) => {
				obj.insert(Keyword::Value.into(), json.clone());
				obj.insert(Keyword::Type.into(), Keyword::Json.as_json())
			}
		}

		JsonValue::Object(obj)
	}
}
