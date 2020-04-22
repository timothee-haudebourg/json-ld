use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use json::JsonValue;
use crate::{
	Id,
	Keyword,
	LangString,
	util
};

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

	/// A JSON literal value.
	Json(JsonValue)
}

impl PartialEq for Literal {
	fn eq(&self, other: &Literal) -> bool {
		use Literal::*;
		match (self, other) {
			(Null, Null) => true,
			(Boolean(a), Boolean(b)) => a == b,
			(Number(a), Number(b)) => {
				a.as_parts() == b.as_parts()
			},
			(String(a), String(b)) => a == b,
			_ => false
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
			Literal::Json(value) => util::hash_json(value, h)
		}
	}
}

impl Literal {
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Literal::String(s) => Some(s.as_str()),
			_ => None
		}
	}
}

#[derive(PartialEq, Eq, Clone)]
pub enum Value<T: Id> {
	/// A typed value.
	Literal(Literal, HashSet<T>),

	/// A language tagged string.
	LangString(LangString)
}

impl<T: Id> Hash for Value<T> {
	fn hash<H: Hasher>(&self, h: &mut H) {
		match self {
			Value::Literal(lit, tys) => {
				lit.hash(h);
				util::hash_set(tys, h)
			},
			Value::LangString(str) => str.hash(h)
		}
	}
}

impl<T: Id> Value<T> {
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Value::Literal(lit, _) => lit.as_str(),
			Value::LangString(str) => Some(str.as_str())
		}
	}
}

impl<T: Id> util::AsJson for Value<T> {
	fn as_json(&self) -> JsonValue {
		let mut obj = json::object::Object::new();

		match self {
			Value::Literal(lit, tys) => {
				let mut tys = if tys.is_empty() {
					None
				} else {
					Some(if tys.len() == 1 {
						tys.iter().next().unwrap().as_json()
					} else {
						tys.as_json()
					})
				};

				match lit {
					Literal::Null => {
						obj.insert(Keyword::Value.into(), JsonValue::Null)
					},
					Literal::Boolean(b) => {
						obj.insert(Keyword::Value.into(), b.as_json())
					},
					Literal::Number(n) => {
						obj.insert(Keyword::Value.into(), JsonValue::Number(n.clone()))
					},
					Literal::String(s) => {
						obj.insert(Keyword::Value.into(), s.as_json())
					},
					Literal::Json(json) => {
						obj.insert(Keyword::Value.into(), json.clone());

						tys = if let Some(tys) = tys {
							let mut ary = match tys {
								JsonValue::Array(ary) => ary,
								json => vec![json]
							};

							ary.push(Keyword::Json.as_json());

							Some(JsonValue::Array(ary))
						} else {
							Some(Keyword::Json.as_json())
						}
					}
				}

				if let Some(tys) = tys {
					obj.insert(Keyword::Type.into(), tys)
				}
			},
			Value::LangString(str) => {
				obj.insert(Keyword::Value.into(), str.as_str().into());

				if let Some(language) = str.language() {
					obj.insert(Keyword::Language.into(), language.as_json());
				}

				if let Some(direction) = str.direction() {
					obj.insert(Keyword::Direction.into(), direction.as_json());
				}
			}
		}

		JsonValue::Object(obj)
	}

}
