use json_syntax::{Number, NumberBuf, String};
use locspan::Meta;
use locspan_derive::*;

pub mod object;
pub use object::Object;

#[derive(
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	Debug,
	StrippedPartialEq,
	StrippedEq,
	StrippedPartialOrd,
	StrippedOrd,
	StrippedHash,
)]
#[stripped_ignore(M)]
pub enum Value<C, M> {
	Null,
	Boolean(#[stripped] bool),
	Number(#[stripped] NumberBuf),
	String(#[stripped] String),
	Array(Array<C, M>),
	Object(Object<C, M>),
}

impl<C, M> Value<C, M> {
	#[inline]
	pub fn is_null(&self) -> bool {
		matches!(self, Self::Null)
	}

	#[inline]
	pub fn is_boolean(&self) -> bool {
		matches!(self, Self::Boolean(_))
	}

	#[inline]
	pub fn is_number(&self) -> bool {
		matches!(self, Self::Number(_))
	}

	#[inline]
	pub fn is_string(&self) -> bool {
		matches!(self, Self::String(_))
	}

	#[inline]
	pub fn is_array(&self) -> bool {
		matches!(self, Self::Array(_))
	}

	#[inline]
	pub fn is_object(&self) -> bool {
		matches!(self, Self::Object(_))
	}

	#[inline]
	pub fn as_boolean(&self) -> Option<bool> {
		match self {
			Self::Boolean(b) => Some(*b),
			_ => None,
		}
	}

	#[inline]
	pub fn as_boolean_mut(&mut self) -> Option<&mut bool> {
		match self {
			Self::Boolean(b) => Some(b),
			_ => None,
		}
	}

	#[inline]
	pub fn as_number(&self) -> Option<&Number> {
		match self {
			Self::Number(n) => Some(n),
			_ => None,
		}
	}

	#[inline]
	pub fn as_number_mut(&mut self) -> Option<&mut NumberBuf> {
		match self {
			Self::Number(n) => Some(n),
			_ => None,
		}
	}

	#[inline]
	pub fn as_string(&self) -> Option<&str> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	#[inline]
	pub fn as_str(&self) -> Option<&str> {
		self.as_string()
	}

	#[inline]
	pub fn as_string_mut(&mut self) -> Option<&mut String> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	#[inline]
	pub fn as_array(&self) -> Option<&[Meta<Self, M>]> {
		match self {
			Self::Array(a) => Some(a),
			_ => None,
		}
	}

	#[inline]
	pub fn force_as_array(this: &Meta<Self, M>) -> &[Meta<Self, M>] {
		match this.value() {
			Self::Array(a) => a,
			_ => core::slice::from_ref(this),
		}
	}

	#[inline]
	pub fn as_array_mut(&mut self) -> Option<&mut Array<C, M>> {
		match self {
			Self::Array(a) => Some(a),
			_ => None,
		}
	}

	#[inline]
	pub fn as_object(&self) -> Option<&Object<C, M>> {
		match self {
			Self::Object(o) => Some(o),
			_ => None,
		}
	}

	#[inline]
	pub fn as_object_mut(&mut self) -> Option<&mut Object<C, M>> {
		match self {
			Self::Object(o) => Some(o),
			_ => None,
		}
	}

	#[inline]
	pub fn into_boolean(self) -> Option<bool> {
		match self {
			Self::Boolean(b) => Some(b),
			_ => None,
		}
	}

	#[inline]
	pub fn into_number(self) -> Option<NumberBuf> {
		match self {
			Self::Number(n) => Some(n),
			_ => None,
		}
	}

	#[inline]
	pub fn into_string(self) -> Option<String> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	#[inline]
	pub fn into_array(self) -> Option<Array<C, M>> {
		match self {
			Self::Array(a) => Some(a),
			_ => None,
		}
	}

	#[inline]
	pub fn into_object(self) -> Option<Object<C, M>> {
		match self {
			Self::Object(o) => Some(o),
			_ => None,
		}
	}

	pub fn into_json(self) -> json_syntax::Value<M> {
		match self {
			Self::Null => json_syntax::Value::Null,
			Self::Boolean(b) => json_syntax::Value::Boolean(b),
			Self::Number(n) => json_syntax::Value::Number(n),
			Self::String(s) => json_syntax::Value::String(s),
			Self::Array(a) => json_syntax::Value::Array(
				a.into_iter()
					.map(|item| item.map(Self::into_json))
					.collect(),
			),
			Self::Object(o) => json_syntax::Value::Object(o.into_json()),
		}
	}
}

/// JSON-LD array.
pub type Array<C, M> = Vec<Meta<Value<C, M>, M>>;
