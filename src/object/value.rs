use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::convert::TryFrom;
use std::fmt;
use iref::{
	Iri,
	IriBuf
};
use json::JsonValue;
use crate::{
	Id,
	Keyword,
	Term,
	Property,
	LangString,
	util
};

#[derive(PartialEq, Eq)]
pub enum Value<T: Id> {
	/// The `null` value.
	Null,

	/// Boolean value.
	Boolean(bool),

	/// Number.
	Number(json::number::Number),

	/// Reference.
	Ref(IriBuf),

	/// A typed value.
	Typed(String, Type<T>),

	/// A language tagged string.
	LangString(LangString),

	/// A JSON literal value.
	Json(JsonValue),
}

impl<T: Id> Hash for Value<T> {
	fn hash<H: Hasher>(&self, h: &mut H) {
		match self {
			Value::Null => (),
			Value::Boolean(b) => b.hash(h),
			Value::Number(n) => util::hash_json_number(n, h),
			Value::Ref(r) => r.hash(h),
			Value::Typed(lit, ty) => {
				lit.hash(h);
				ty.hash(h)
			},
			Value::LangString(str) => str.hash(h),
			Value::Json(value) => util::hash_json(value, h)
		}
	}
}

impl<T: Id> Value<T> {
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Value::Ref(r) => Some(r.as_str()),
			Value::Typed(lit, _) => Some(lit.as_str()),
			Value::LangString(str) => Some(str.as_str()),
			_ => None
		}
	}

	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Value::Ref(r) => Some(r.as_iri()),
			_ => None
		}
	}
}

impl<T: Id> util::AsJson for Value<T> {
	fn as_json(&self) -> JsonValue {
		let mut obj = json::object::Object::new();

		match self {
			Value::Null => {
				obj.insert(Keyword::Value.into(), JsonValue::Null)
			},
			Value::Boolean(b) => {
				obj.insert(Keyword::Value.into(), b.as_json())
			},
			Value::Number(n) => {
				obj.insert(Keyword::Value.into(), JsonValue::Number(n.clone()))
			},
			Value::Ref(id) => {
				obj.insert(Keyword::Value.into(), id.as_json())
			},
			Value::Typed(lit, ty) => {
				obj.insert(Keyword::Value.into(), lit.as_json());
				obj.insert(Keyword::Type.into(), ty.as_json())
			},
			Value::LangString(str) => {
				obj.insert(Keyword::Value.into(), str.as_str().into());

				if let Some(language) = str.language() {
					obj.insert(Keyword::Language.into(), language.as_json());
				}

				if let Some(direction) = str.direction() {
					obj.insert(Keyword::Direction.into(), direction.as_json());
				}
			},
			Value::Json(json) => {
				obj.insert(Keyword::Value.into(), json.clone())
			}
		}

		JsonValue::Object(obj)
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type<T: Id> {
	Id,
	JSON,
	None,
	Vocab,
	Prop(Property<T>)
}

impl<T: Id> Type<T> {
	pub fn as_str(&self) -> &str {
		match self {
			Type::Id => "@id",
			Type::JSON => "@json",
			Type::None => "@none",
			Type::Vocab => "@vocab",
			Type::Prop(p) => p.as_str()
		}
	}
}

impl<T: Id> TryFrom<Term<T>> for Type<T> {
	type Error = Term<T>;

	fn try_from(term: Term<T>) -> Result<Type<T>, Term<T>> {
		match term {
			Term::Keyword(Keyword::Id) => Ok(Type::Id),
			Term::Keyword(Keyword::JSON) => Ok(Type::JSON),
			Term::Keyword(Keyword::None) => Ok(Type::None),
			Term::Keyword(Keyword::Vocab) => Ok(Type::Vocab),
			Term::Prop(prop) => Ok(Type::Prop(prop)),
			term => Err(term)
		}
	}
}

impl<T: Id> util::AsJson for Type<T> {
	fn as_json(&self) -> JsonValue {
		self.as_str().into()
	}
}

impl<T: Id> fmt::Display for Type<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
