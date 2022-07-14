use crate::{context, AnyContextEntry};
pub use json_syntax::{Kind, Number, NumberBuf, String};
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
	pub fn kind(&self) -> Kind {
		match self {
			Self::Null => Kind::Null,
			Self::Boolean(_) => Kind::Boolean,
			Self::Number(_) => Kind::Number,
			Self::String(_) => Kind::String,
			Self::Array(_) => Kind::Array,
			Self::Object(_) => Kind::Object,
		}
	}

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

	/// Recursively count the number of values for which `f` returns `true`.
	pub fn count(&self, f: impl Clone + Fn(ComponentRef<C>) -> bool) -> usize
	where
		C: AnyContextEntry<Metadata = M> + context::Count<C>,
	{
		let mut result = if f(ComponentRef::Value(self)) { 1 } else { 0 };

		match self {
			Self::Array(array) => {
				for item in array {
					result += item.count(f.clone())
				}
			}
			Self::Object(object) => {
				if let Some(context) = object.context() {
					result += context.count(f.clone())
				}

				for entry in object {
					result += entry.value.count(f.clone())
				}
			}
			_ => (),
		}

		result
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

	pub fn try_from_json_with<E, F>(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
		context_from_json: F,
	) -> Result<Meta<Self, M>, Meta<ValueFromJsonError<M, C, E>, M>>
	where
		F: Clone + Fn(Meta<json_syntax::Value<M>, M>) -> Result<Meta<C, M>, Meta<E, M>>,
	{
		match value {
			json_syntax::Value::Null => Ok(Meta(Self::Null, meta)),
			json_syntax::Value::Boolean(b) => Ok(Meta(Self::Boolean(b), meta)),
			json_syntax::Value::Number(n) => Ok(Meta(Self::Number(n), meta)),
			json_syntax::Value::String(s) => Ok(Meta(Self::String(s), meta)),
			json_syntax::Value::Array(a) => {
				let mut result = Vec::with_capacity(a.len());

				for item in a {
					result.push(Self::try_from_json_with(item, context_from_json.clone())?)
				}

				Ok(Meta(Self::Array(result), meta))
			}
			json_syntax::Value::Object(o) => {
				let mut result = Object::with_capacity(o.len());

				for json_syntax::object::Entry { key, value } in o {
					if key.as_str() == "@context" {
						let context = context_from_json(value)
							.map_err(|e| e.map(ValueFromJsonError::InvalidContext))?;
						if let Some(duplicate) = result.set_context(key.into_metadata(), context) {
							let entry = result.remove_context().unwrap();
							return Err(Meta(
								ValueFromJsonError::DuplicateContext(duplicate),
								entry.key_metadata,
							));
						}
					} else {
						if let Some(duplicate) = result.insert(
							key,
							Self::try_from_json_with(value, context_from_json.clone())?,
						) {
							let entry = result.remove(duplicate.key.value()).unwrap();
							return Err(Meta(
								ValueFromJsonError::DuplicateKey(duplicate),
								entry.key.into_metadata(),
							));
						}
					}
				}

				Ok(Meta(Self::Object(result), meta))
			}
		}
	}
}

impl<C, M> From<bool> for Value<C, M> {
	fn from(b: bool) -> Self {
		Self::Boolean(b)
	}
}

impl<C, M> From<NumberBuf> for Value<C, M> {
	fn from(n: NumberBuf) -> Self {
		Self::Number(n)
	}
}

impl<C, M> From<String> for Value<C, M> {
	fn from(s: String) -> Self {
		Self::String(s)
	}
}

impl<C, M> From<::std::string::String> for Value<C, M> {
	fn from(s: ::std::string::String) -> Self {
		Self::String(s.into())
	}
}

impl<C, M> From<Array<C, M>> for Value<C, M> {
	fn from(a: Array<C, M>) -> Self {
		Self::Array(a)
	}
}

impl<C, M> From<Object<C, M>> for Value<C, M> {
	fn from(o: Object<C, M>) -> Self {
		Self::Object(o)
	}
}

/// JSON-LD array.
pub type Array<C, M> = Vec<Meta<Value<C, M>, M>>;

impl<M> crate::TryFromJson<M> for bool {
	type Error = crate::Unexpected;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<Self::Error, M>> {
		match value {
			json_syntax::Value::Boolean(b) => Ok(Meta(b, meta)),
			unexpected => Err(Meta(
				crate::Unexpected(unexpected.kind(), &[json_syntax::Kind::Boolean]),
				meta,
			)),
		}
	}
}

#[derive(Clone, Debug)]
pub enum ValueFromJsonError<M, C = crate::ContextEntry<M>, E = crate::context::InvalidContext> {
	DuplicateContext(object::ContextEntry<C, M>),
	DuplicateKey(object::Entry<C, M>),
	InvalidContext(E),
}

impl<M: Clone> crate::TryFromJson<M> for Value<crate::ContextEntry<M>, M> {
	type Error = ValueFromJsonError<M>;

	fn try_from_json(
		value: Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<Self::Error, M>> {
		Self::try_from_json(value)
	}
}

impl<M: Clone> Value<crate::ContextEntry<M>, M> {
	pub fn try_from_json(
		value: Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<ValueFromJsonError<M>, M>> {
		Self::try_from_json_with(value, crate::TryFromJson::try_from_json)
	}
}

pub enum ComponentRef<'a, C: crate::AnyContextEntry> {
	Value(&'a Value<C, C::Metadata>),
	Context(crate::context::ContextComponentRef<'a, C>),
}

impl<'a, C: crate::AnyContextEntry> ComponentRef<'a, C> {
	pub fn is_array(&self) -> bool {
		match self {
			Self::Value(v) => v.is_array(),
			Self::Context(c) => c.is_array(),
		}
	}

	pub fn is_object(&self) -> bool {
		match self {
			Self::Value(v) => v.is_object(),
			Self::Context(c) => c.is_object(),
		}
	}
}
