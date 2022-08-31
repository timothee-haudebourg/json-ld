pub use json_syntax::{Kind, Number, NumberBuf, String};
use locspan::Meta;
use locspan_derive::*;
use smallvec::SmallVec;
use std::fmt;

pub mod object;
pub use object::Object;

pub type MetaValue<M> = Meta<Value<M>, M>;

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
pub enum Value<M> {
	Null,
	Boolean(#[stripped] bool),
	Number(#[stripped] NumberBuf),
	String(#[stripped] String),
	Array(Array<M>),
	Object(Object<M>),
}

impl<M> Value<M> {
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
	pub fn force_as_array(this @ Meta(value, meta): &Meta<Self, M>) -> Meta<&[Meta<Self, M>], &M> {
		match value {
			Self::Array(a) => Meta(a, meta),
			_ => Meta(core::slice::from_ref(this), meta),
		}
	}

	#[inline]
	pub fn as_array_mut(&mut self) -> Option<&mut Array<M>> {
		match self {
			Self::Array(a) => Some(a),
			_ => None,
		}
	}

	#[inline]
	pub fn as_object(&self) -> Option<&Object<M>> {
		match self {
			Self::Object(o) => Some(o),
			_ => None,
		}
	}

	#[inline]
	pub fn as_object_mut(&mut self) -> Option<&mut Object<M>> {
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
	pub fn into_array(self) -> Option<Array<M>> {
		match self {
			Self::Array(a) => Some(a),
			_ => None,
		}
	}

	#[inline]
	pub fn into_object(self) -> Option<Object<M>> {
		match self {
			Self::Object(o) => Some(o),
			_ => None,
		}
	}

	fn sub_items(&self) -> ValueSubFragments<M> {
		match self {
			Self::Array(a) => ValueSubFragments::Array(a.iter()),
			Self::Object(o) => ValueSubFragments::Object(o.iter()),
			_ => ValueSubFragments::None,
		}
	}

	pub fn traverse(&self) -> TraverseStripped<M> {
		let mut stack = SmallVec::new();
		stack.push(StrippedFragmentRef::Value(self));
		TraverseStripped { stack }
	}

	/// Recursively count the number of fragments for which `f` returns `true`.
	pub fn count(&self, mut f: impl FnMut(StrippedFragmentRef<M>) -> bool) -> usize {
		let mut count = 0;

		for i in self.traverse() {
			if f(i) {
				count += 1
			}
		}

		count
	}

	pub fn try_from_json_with(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>
	) -> Result<Meta<Self, M>, MetaValueFromJsonError<M>> {
		match value {
			json_syntax::Value::Null => Ok(Meta(Self::Null, meta)),
			json_syntax::Value::Boolean(b) => Ok(Meta(Self::Boolean(b), meta)),
			json_syntax::Value::Number(n) => Ok(Meta(Self::Number(n), meta)),
			json_syntax::Value::String(s) => Ok(Meta(Self::String(s), meta)),
			json_syntax::Value::Array(a) => {
				let mut result = Vec::with_capacity(a.len());

				for item in a {
					result.push(Self::try_from_json_with(item)?)
				}

				Ok(Meta(Self::Array(result), meta))
			}
			json_syntax::Value::Object(o) => {
				let mut result = Object::with_capacity(o.len());

				for json_syntax::object::Entry { key, value } in o {
					if let Some(duplicate) = result.insert(
						key,
						Self::try_from_json_with(value)?,
					) {
						let entry = result.remove(duplicate.key.value()).unwrap();
						return Err(Meta(
							ValueFromJsonError::DuplicateKey(duplicate),
							entry.key.into_metadata(),
						));
					}
				}

				Ok(Meta(Self::Object(result), meta))
			}
		}
	}
}

pub trait Traversal<'a> {
	type Fragment;
	type Traverse: Iterator<Item = Self::Fragment>;

	fn traverse(&'a self) -> Self::Traverse;
}

impl<'a, M: 'a> Traversal<'a> for MetaValue<M> {
	type Fragment = FragmentRef<'a, M>;
	type Traverse = Traverse<'a, M>;

	fn traverse(&'a self) -> Self::Traverse {
		let mut stack = SmallVec::new();
		stack.push(FragmentRef::Value(self));
		Traverse { stack }
	}
}

impl<M> From<bool> for Value<M> {
	fn from(b: bool) -> Self {
		Self::Boolean(b)
	}
}

impl<M> From<NumberBuf> for Value<M> {
	fn from(n: NumberBuf) -> Self {
		Self::Number(n)
	}
}

impl<M> From<String> for Value<M> {
	fn from(s: String) -> Self {
		Self::String(s)
	}
}

impl<M> From<::std::string::String> for Value<M> {
	fn from(s: ::std::string::String) -> Self {
		Self::String(s.into())
	}
}

impl<M> From<Array<M>> for Value<M> {
	fn from(a: Array<M>) -> Self {
		Self::Array(a)
	}
}

impl<M> From<Object<M>> for Value<M> {
	fn from(o: Object<M>) -> Self {
		Self::Object(o)
	}
}

/// JSON-LD array.
pub type Array<M> = Vec<Meta<Value<M>, M>>;

#[derive(Clone, Debug)]
pub enum ValueFromJsonError<M> {
	DuplicateKey(object::Entry<M>)
}

pub type MetaValueFromJsonError<M> =
	Meta<ValueFromJsonError<M>, M>;

impl<M> fmt::Display for ValueFromJsonError<M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::DuplicateKey(e) => write!(f, "duplicate key `{}`", e.key.value())
		}
	}
}

impl<M: Clone> crate::TryFromJson<M> for Value<M> {
	type Error = ValueFromJsonError<M>;

	fn try_from_json(
		value: Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<Self::Error, M>> {
		Self::try_from_json(value)
	}
}

impl<M: Clone> Value<M> {
	pub fn try_from_json(
		value: Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<ValueFromJsonError<M>, M>> {
		Self::try_from_json_with(value)
	}
}

pub enum ValueSubFragments<'a, M> {
	None,
	Array(core::slice::Iter<'a, MetaValue<M>>),
	Object(core::slice::Iter<'a, object::Entry<M>>),
}

impl<'a, M> Iterator for ValueSubFragments<'a, M> {
	type Item = FragmentRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Array(a) => a.next_back().map(|v| FragmentRef::Value(v)),
			Self::Object(e) => e.next_back().map(|e| FragmentRef::Value(e.as_value())),
		}
	}
}

/// JSON-LD value fragment.
pub enum FragmentRef<'a, M> {
	/// Object entry.
	Entry(&'a object::Entry<M>),

	/// Object entry key.
	Key(&'a Meta<object::Key, M>),

	/// JSON-LD value (may be an object entry value).
	Value(&'a MetaValue<M>)
}

impl<'a, M> FragmentRef<'a, M> {
	pub fn is_array(&self) -> bool {
		match self {
			Self::Value(v) => v.is_array(),
			_ => false,
		}
	}

	pub fn is_object(&self) -> bool {
		match self {
			Self::Value(v) => v.is_object(),
			_ => false,
		}
	}

	pub fn sub_items(&self) -> SubFragments<'a, M> {
		match self {
			Self::Entry(e) => SubFragments::Entry(Some(&e.key), Some(&e.value)),
			Self::Key(_) => SubFragments::None,
			Self::Value(v) => SubFragments::Value(v.sub_items())
		}
	}

	pub fn strip(self) -> StrippedFragmentRef<'a, M> {
		match self {
			Self::Entry(e) => StrippedFragmentRef::Entry(e),
			Self::Key(k) => StrippedFragmentRef::Key(k),
			Self::Value(v) => StrippedFragmentRef::Value(v)
		}
	}
}

impl<'a, M> locspan::Strip for FragmentRef<'a, M> {
	type Stripped = StrippedFragmentRef<'a, M>;

	fn strip(self) -> Self::Stripped {
		self.strip()
	}
}

/// JSON-LD value fragment.
pub enum StrippedFragmentRef<'a, M> {
	/// Object entry.
	Entry(&'a object::Entry<M>),

	/// Object entry key.
	Key(&'a object::Key),

	/// JSON-LD value (may be an object entry value).
	Value(&'a Value<M>)
}

impl<'a, M> StrippedFragmentRef<'a, M> {
	pub fn is_array(&self) -> bool {
		match self {
			Self::Value(v) => v.is_array(),
			_ => false,
		}
	}

	pub fn is_object(&self) -> bool {
		match self {
			Self::Value(v) => v.is_object(),
			_ => false,
		}
	}

	pub fn sub_items(&self) -> SubFragments<'a, M> {
		match self {
			Self::Entry(e) => SubFragments::Entry(Some(&e.key), Some(&e.value)),
			Self::Key(_) => SubFragments::None,
			Self::Value(v) => SubFragments::Value(v.sub_items())
		}
	}
}

pub enum SubFragments<'a, M> {
	None,
	Entry(
		Option<&'a Meta<object::Key, M>>,
		Option<&'a MetaValue<M>>,
	),
	Value(ValueSubFragments<'a, M>)
}

impl<'a, M> Iterator for SubFragments<'a, M> {
	type Item = FragmentRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Entry(k, v) => k
				.take()
				.map(FragmentRef::Key)
				.or_else(|| v.take().map(FragmentRef::Value)),
			Self::Value(s) => s.next()
		}
	}
}

pub struct Traverse<'a, M> {
	stack: SmallVec<[FragmentRef<'a, M>; 8]>,
}

impl<'a, M> Iterator for Traverse<'a, M> {
	type Item = FragmentRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.stack.pop() {
			Some(item) => {
				self.stack.extend(item.sub_items());
				Some(item)
			}
			None => None,
		}
	}
}

pub struct TraverseStripped<'a, M> {
	stack: SmallVec<[StrippedFragmentRef<'a, M>; 8]>,
}

impl<'a, M> Iterator for TraverseStripped<'a, M> {
	type Item = StrippedFragmentRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.stack.pop() {
			Some(item) => {
				self.stack.extend(item.sub_items().map(FragmentRef::strip));
				Some(item)
			}
			None => None,
		}
	}
}
