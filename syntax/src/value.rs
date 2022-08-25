use crate::context::{self, ValueRef};
pub use json_syntax::{Kind, Number, NumberBuf, String};
use locspan::Meta;
use locspan_derive::*;
use smallvec::SmallVec;
use std::fmt;

pub mod object;
pub use object::Object;

pub type MetaValue<M, C = context::Value<M>> = Meta<Value<M, C>, M>;

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
pub enum Value<M, C = context::Value<M>> {
	Null,
	Boolean(#[stripped] bool),
	Number(#[stripped] NumberBuf),
	String(#[stripped] String),
	Array(Array<M, C>),
	Object(Object<M, C>),
}

impl<M, C> Value<M, C> {
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
	pub fn as_array_mut(&mut self) -> Option<&mut Array<M, C>> {
		match self {
			Self::Array(a) => Some(a),
			_ => None,
		}
	}

	#[inline]
	pub fn as_object(&self) -> Option<&Object<M, C>> {
		match self {
			Self::Object(o) => Some(o),
			_ => None,
		}
	}

	#[inline]
	pub fn as_object_mut(&mut self) -> Option<&mut Object<M, C>> {
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
	pub fn into_array(self) -> Option<Array<M, C>> {
		match self {
			Self::Array(a) => Some(a),
			_ => None,
		}
	}

	#[inline]
	pub fn into_object(self) -> Option<Object<M, C>> {
		match self {
			Self::Object(o) => Some(o),
			_ => None,
		}
	}

	fn sub_items(&self) -> ValueSubFragments<C>
	where
		C: context::AnyValue<Metadata = M>,
	{
		match self {
			Self::Array(a) => ValueSubFragments::Array(a.iter()),
			Self::Object(o) => ValueSubFragments::Object(o.context().map(Meta::value), o.iter()),
			_ => ValueSubFragments::None,
		}
	}

	pub fn traverse(&self) -> TraverseStripped<C>
	where
		C: context::AnyValue<Metadata = M>,
	{
		let mut stack = SmallVec::new();
		stack.push(StrippedFragmentRef::Value(self));
		TraverseStripped { stack }
	}

	/// Recursively count the number of fragments for which `f` returns `true`.
	pub fn count(&self, mut f: impl FnMut(StrippedFragmentRef<C>) -> bool) -> usize
	where
		C: context::AnyValue<Metadata = M>,
	{
		let mut count = 0;

		for i in self.traverse() {
			if f(i) {
				count += 1
			}
		}

		count
	}

	pub fn try_from_json_with<E, F>(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
		context_from_json: F,
	) -> Result<Meta<Self, M>, MetaValueFromJsonError<M, C, E>>
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
					} else if let Some(duplicate) = result.insert(
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

impl<'a, C: 'a + context::AnyValue> Traversal<'a> for MetaValue<C::Metadata, C> {
	type Fragment = FragmentRef<'a, C>;
	type Traverse = Traverse<'a, C>;

	fn traverse(&'a self) -> Self::Traverse {
		let mut stack = SmallVec::new();
		stack.push(FragmentRef::Value(self));
		Traverse { stack }
	}
}

impl<M, C> From<bool> for Value<M, C> {
	fn from(b: bool) -> Self {
		Self::Boolean(b)
	}
}

impl<M, C> From<NumberBuf> for Value<M, C> {
	fn from(n: NumberBuf) -> Self {
		Self::Number(n)
	}
}

impl<M, C> From<String> for Value<M, C> {
	fn from(s: String) -> Self {
		Self::String(s)
	}
}

impl<M, C> From<::std::string::String> for Value<M, C> {
	fn from(s: ::std::string::String) -> Self {
		Self::String(s.into())
	}
}

impl<M, C> From<Array<M, C>> for Value<M, C> {
	fn from(a: Array<M, C>) -> Self {
		Self::Array(a)
	}
}

impl<M, C> From<Object<M, C>> for Value<M, C> {
	fn from(o: Object<M, C>) -> Self {
		Self::Object(o)
	}
}

/// JSON-LD array.
pub type Array<M, C> = Vec<Meta<Value<M, C>, M>>;

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
pub enum ValueFromJsonError<M, C = context::Value<M>, E = context::InvalidContext> {
	DuplicateContext(object::ContextEntry<M, C>),
	DuplicateKey(object::Entry<M, C>),
	InvalidContext(E),
}

pub type MetaValueFromJsonError<M, C = context::Value<M>, E = context::InvalidContext> =
	Meta<ValueFromJsonError<M, C, E>, M>;

impl<M, C, E: fmt::Display> fmt::Display for ValueFromJsonError<M, C, E> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::DuplicateContext(_) => write!(f, "duplicate context entry"),
			Self::DuplicateKey(e) => write!(f, "duplicate key `{}`", e.key.value()),
			Self::InvalidContext(e) => e.fmt(f),
		}
	}
}

impl<M: Clone> crate::TryFromJson<M> for Value<M, context::Value<M>> {
	type Error = ValueFromJsonError<M>;

	fn try_from_json(
		value: Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<Self::Error, M>> {
		Self::try_from_json(value)
	}
}

impl<M: Clone> Value<M, context::Value<M>> {
	pub fn try_from_json(
		value: Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<ValueFromJsonError<M>, M>> {
		Self::try_from_json_with(value, crate::TryFromJson::try_from_json)
	}
}

pub enum ValueSubFragments<'a, C: context::AnyValue> {
	None,
	Array(core::slice::Iter<'a, MetaValue<C::Metadata, C>>),
	Object(
		Option<&'a C>,
		core::slice::Iter<'a, object::Entry<C::Metadata, C>>,
	),
}

impl<'a, C: context::AnyValue> Iterator for ValueSubFragments<'a, C> {
	type Item = FragmentRef<'a, C>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Array(a) => a.next_back().map(|v| FragmentRef::Value(v)),
			Self::Object(c, e) => match c.take() {
				Some(c) => {
					let item = match c.as_value_ref() {
						ValueRef::One(Meta(c, _)) => context::FragmentRef::Context(c),
						ValueRef::Many(m) => context::FragmentRef::ContextArray(m),
					};

					Some(FragmentRef::ContextFragment(item))
				}
				None => e.next_back().map(|e| FragmentRef::Value(e.as_value())),
			},
		}
	}
}

/// JSON-LD value fragment.
pub enum FragmentRef<'a, C: context::AnyValue> {
	/// Object entry.
	Entry(&'a object::Entry<C::Metadata, C>),

	/// Object entry key.
	Key(&'a Meta<object::Key, C::Metadata>),

	/// JSON-LD value (may be an object entry value).
	Value(&'a MetaValue<C::Metadata, C>),

	/// Context value fragment.
	ContextFragment(crate::context::FragmentRef<'a, C>),
}

impl<'a, C: context::AnyValue> FragmentRef<'a, C> {
	pub fn is_array(&self) -> bool {
		match self {
			Self::Value(v) => v.is_array(),
			Self::ContextFragment(c) => c.is_array(),
			_ => false,
		}
	}

	pub fn is_object(&self) -> bool {
		match self {
			Self::Value(v) => v.is_object(),
			Self::ContextFragment(c) => c.is_object(),
			_ => false,
		}
	}

	pub fn sub_items(&self) -> SubFragments<'a, C> {
		match self {
			Self::Entry(e) => SubFragments::Entry(Some(&e.key), Some(&e.value)),
			Self::Key(_) => SubFragments::None,
			Self::Value(v) => SubFragments::Value(v.sub_items()),
			Self::ContextFragment(c) => SubFragments::Context(c.sub_items()),
		}
	}

	pub fn strip(self) -> StrippedFragmentRef<'a, C> {
		match self {
			Self::Entry(e) => StrippedFragmentRef::Entry(e),
			Self::Key(k) => StrippedFragmentRef::Key(k),
			Self::Value(v) => StrippedFragmentRef::Value(v),
			Self::ContextFragment(f) => StrippedFragmentRef::ContextFragment(f),
		}
	}
}

impl<'a, C: context::AnyValue> locspan::Strip for FragmentRef<'a, C> {
	type Stripped = StrippedFragmentRef<'a, C>;

	fn strip(self) -> Self::Stripped {
		self.strip()
	}
}

/// JSON-LD value fragment.
pub enum StrippedFragmentRef<'a, C: context::AnyValue> {
	/// Object entry.
	Entry(&'a object::Entry<C::Metadata, C>),

	/// Object entry key.
	Key(&'a object::Key),

	/// JSON-LD value (may be an object entry value).
	Value(&'a Value<C::Metadata, C>),

	/// Context value fragment.
	ContextFragment(crate::context::FragmentRef<'a, C>),
}

impl<'a, C: context::AnyValue> StrippedFragmentRef<'a, C> {
	pub fn is_array(&self) -> bool {
		match self {
			Self::Value(v) => v.is_array(),
			Self::ContextFragment(c) => c.is_array(),
			_ => false,
		}
	}

	pub fn is_object(&self) -> bool {
		match self {
			Self::Value(v) => v.is_object(),
			Self::ContextFragment(c) => c.is_object(),
			_ => false,
		}
	}

	pub fn sub_items(&self) -> SubFragments<'a, C> {
		match self {
			Self::Entry(e) => SubFragments::Entry(Some(&e.key), Some(&e.value)),
			Self::Key(_) => SubFragments::None,
			Self::Value(v) => SubFragments::Value(v.sub_items()),
			Self::ContextFragment(c) => SubFragments::Context(c.sub_items()),
		}
	}
}

pub enum SubFragments<'a, C: context::AnyValue> {
	None,
	Entry(
		Option<&'a Meta<object::Key, C::Metadata>>,
		Option<&'a MetaValue<C::Metadata, C>>,
	),
	Value(ValueSubFragments<'a, C>),
	Context(context::SubFragments<'a, C>),
}

impl<'a, C: context::AnyValue> Iterator for SubFragments<'a, C> {
	type Item = FragmentRef<'a, C>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Entry(k, v) => k
				.take()
				.map(FragmentRef::Key)
				.or_else(|| v.take().map(FragmentRef::Value)),
			Self::Value(s) => s.next(),
			Self::Context(s) => s.next().map(FragmentRef::ContextFragment),
		}
	}
}

pub struct Traverse<'a, C: context::AnyValue> {
	stack: SmallVec<[FragmentRef<'a, C>; 8]>,
}

impl<'a, C: 'a + context::AnyValue> Iterator for Traverse<'a, C> {
	type Item = FragmentRef<'a, C>;

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

pub struct TraverseStripped<'a, C: context::AnyValue> {
	stack: SmallVec<[StrippedFragmentRef<'a, C>; 8]>,
}

impl<'a, C: 'a + context::AnyValue> Iterator for TraverseStripped<'a, C> {
	type Item = StrippedFragmentRef<'a, C>;

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
