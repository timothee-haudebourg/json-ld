use crate::{object, Direction, LangString, LenientLangTag};
use educe::Educe;
use iref::{Iri, IriBuf};
use json_ld_syntax::{IntoJsonWithContext, Keyword};
use json_syntax::{Number, NumberBuf};
use rdf_types::vocabulary::{IriVocabulary, IriVocabularyMut};
use std::{hash::Hash, marker::PhantomData};

use super::InvalidExpandedJson;

/// Value type.
pub enum Type<T> {
	Json,
	Id(T),
}

impl<T> Type<T> {
	pub fn as_id(&self) -> Option<crate::id::Ref<T>> {
		match self {
			Self::Json => None,
			Self::Id(t) => Some(crate::id::Ref::Iri(t)),
		}
	}
}

/// Value type reference.
#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum TypeRef<'a, T> {
	Json,
	Id(&'a T),
}

impl<'a, T> TypeRef<'a, T> {
	pub fn as_syntax_type(&self) -> crate::Type<&'a T> {
		match self {
			Self::Json => crate::Type::Json,
			Self::Id(id) => crate::Type::Iri(id),
		}
	}

	pub fn into_reference<B>(self) -> Option<crate::id::Ref<'a, T, B>> {
		match self {
			Self::Json => None,
			Self::Id(t) => Some(crate::id::Ref::Iri(t)),
		}
	}
}

/// Literal value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Literal {
	/// The `null` value.
	Null,

	/// Boolean value.
	Boolean(bool),

	/// Number.
	Number(NumberBuf),

	/// String.
	String(json_syntax::String),
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

	pub fn into_json(self) -> json_syntax::Value {
		match self {
			Self::Null => json_syntax::Value::Null,
			Self::Boolean(b) => json_syntax::Value::Boolean(b),
			Self::Number(n) => json_syntax::Value::Number(n),
			Self::String(s) => json_syntax::Value::String(s),
		}
	}

	/// Puts this literal into canonical form using the given `buffer`.
	///
	/// The buffer is used to compute the canonical form of numbers.
	pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer) {
		if let Self::Number(n) = self {
			*n = NumberBuf::from_number(n.canonical_with(buffer))
		}
	}

	/// Puts this literal into canonical form.
	pub fn canonicalize(&mut self) {
		let mut buffer = ryu_js::Buffer::new();
		self.canonicalize_with(&mut buffer)
	}
}

/// Value object.
///
/// Either a typed literal value, or an internationalized language string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value<T = IriBuf> {
	/// Typed literal value.
	Literal(Literal, Option<T>),

	/// Language tagged string.
	LangString(LangString),

	/// JSON literal value.
	Json(json_syntax::Value),
}

impl<T> Value<T> {
	/// Creates a `null` value object.
	#[inline(always)]
	pub fn null() -> Self {
		Self::Literal(Literal::Null, None)
	}

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
	pub fn language(&self) -> Option<&LenientLangTag> {
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

	#[inline(always)]
	pub fn entries(&self) -> Entries<T> {
		match self {
			Self::Literal(l, ty) => Entries {
				value: Some(ValueEntryRef::Literal(l)),
				type_: ty.as_ref().map(TypeRef::Id),
				language: None,
				direction: None,
			},
			Self::LangString(l) => Entries {
				value: Some(ValueEntryRef::LangString(l.as_str())),
				type_: None,
				language: l.language(),
				direction: l.direction(),
			},
			Self::Json(j) => Entries {
				value: Some(ValueEntryRef::Json(j)),
				type_: Some(TypeRef::Json),
				language: None,
				direction: None,
			},
		}
	}

	pub(crate) fn try_from_json_object_in(
		vocabulary: &mut impl IriVocabularyMut<Iri = T>,
		mut object: json_syntax::Object,
		value_entry: json_syntax::object::Entry,
	) -> Result<Self, InvalidExpandedJson> {
		match object
			.remove_unique("@type")
			.map_err(InvalidExpandedJson::duplicate_key)?
		{
			Some(type_entry) => match type_entry.value {
				json_syntax::Value::String(ty) => match ty.as_str() {
					"@json" => Ok(Self::Json(value_entry.value)),
					iri => match Iri::new(iri) {
						Ok(iri) => {
							let ty = vocabulary.insert(iri);
							let lit = value_entry.value.try_into()?;
							Ok(Self::Literal(lit, Some(ty)))
						}
						Err(_) => Err(InvalidExpandedJson::InvalidValueType),
					},
				},
				_ => Err(InvalidExpandedJson::InvalidValueType),
			},
			None => {
				let language = object
					.remove_unique("@language")
					.map_err(InvalidExpandedJson::duplicate_key)?
					.map(json_syntax::object::Entry::into_value);
				let direction = object
					.remove_unique("@direction")
					.map_err(InvalidExpandedJson::duplicate_key)?
					.map(json_syntax::object::Entry::into_value);

				if language.is_some() || direction.is_some() {
					Ok(Self::LangString(LangString::try_from_json(
						object,
						value_entry.value,
						language,
						direction,
					)?))
				} else {
					let lit = value_entry.value.try_into()?;
					Ok(Self::Literal(lit, None))
				}
			}
		}
	}

	/// Puts this value object literal into canonical form using the given
	/// `buffer`.
	///
	/// The buffer is used to compute the canonical form of numbers.
	pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer) {
		match self {
			Self::Json(json) => json.canonicalize_with(buffer),
			Self::Literal(l, _) => l.canonicalize_with(buffer),
			Self::LangString(_) => (),
		}
	}

	/// Puts this literal into canonical form.
	pub fn canonicalize(&mut self) {
		let mut buffer = ryu_js::Buffer::new();
		self.canonicalize_with(&mut buffer)
	}

	/// Map the type IRI of this value, if any.
	pub fn map_ids<U>(self, map_iri: impl FnOnce(T) -> U) -> Value<U> {
		match self {
			Self::Literal(l, type_) => Value::Literal(l, type_.map(map_iri)),
			Self::LangString(s) => Value::LangString(s),
			Self::Json(json) => Value::Json(json),
		}
	}
}

impl TryFrom<json_syntax::Value> for Literal {
	type Error = InvalidExpandedJson;

	fn try_from(value: json_syntax::Value) -> Result<Self, Self::Error> {
		match value {
			json_syntax::Value::Null => Ok(Self::Null),
			json_syntax::Value::Boolean(b) => Ok(Self::Boolean(b)),
			json_syntax::Value::Number(n) => Ok(Self::Number(n)),
			json_syntax::Value::String(s) => Ok(Self::String(s)),
			_ => Err(InvalidExpandedJson::InvalidLiteral),
		}
	}
}

impl<T, B> object::Any<T, B> for Value<T> {
	#[inline(always)]
	fn as_ref(&self) -> object::Ref<T, B> {
		object::Ref::Value(self)
	}
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum EntryRef<'a, T> {
	Value(ValueEntryRef<'a>),
	Type(TypeRef<'a, T>),
	Language(&'a LenientLangTag),
	Direction(Direction),
}

impl<'a, T> EntryRef<'a, T> {
	pub fn into_key(self) -> EntryKey {
		match self {
			Self::Value(_) => EntryKey::Value,
			Self::Type(_) => EntryKey::Type,
			Self::Language(_) => EntryKey::Language,
			Self::Direction(_) => EntryKey::Direction,
		}
	}

	pub fn key(&self) -> EntryKey {
		self.into_key()
	}

	pub fn into_value(self) -> EntryValueRef<'a, T> {
		match self {
			Self::Value(v) => EntryValueRef::Value(v),
			Self::Type(v) => EntryValueRef::Type(v),
			Self::Language(v) => EntryValueRef::Language(v),
			Self::Direction(v) => EntryValueRef::Direction(v),
		}
	}

	pub fn value(&self) -> EntryValueRef<'a, T> {
		match self {
			Self::Value(v) => EntryValueRef::Value(*v),
			Self::Type(v) => EntryValueRef::Type(*v),
			Self::Language(v) => EntryValueRef::Language(v),
			Self::Direction(v) => EntryValueRef::Direction(*v),
		}
	}
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum EntryValueRef<'a, T> {
	Value(ValueEntryRef<'a>),
	Type(TypeRef<'a, T>),
	Language(&'a LenientLangTag),
	Direction(Direction),
}
pub enum ValueEntryRef<'a> {
	Literal(&'a Literal),
	LangString(&'a str),
	Json(&'a json_syntax::Value),
}

impl<'a> Clone for ValueEntryRef<'a> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<'a> Copy for ValueEntryRef<'a> {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntryKey {
	Value,
	Type,
	Language,
	Direction,
}

impl EntryKey {
	pub fn into_keyword(self) -> Keyword {
		match self {
			Self::Value => Keyword::Value,
			Self::Type => Keyword::Type,
			Self::Language => Keyword::Language,
			Self::Direction => Keyword::Direction,
		}
	}

	pub fn as_keyword(&self) -> Keyword {
		self.into_keyword()
	}

	pub fn into_str(&self) -> &'static str {
		match self {
			Self::Value => "@value",
			Self::Type => "@type",
			Self::Language => "@language",
			Self::Direction => "@direction",
		}
	}

	pub fn as_str(&self) -> &'static str {
		self.into_str()
	}
}

#[derive(Educe)]
#[educe(Clone)]
pub struct Entries<'a, T> {
	value: Option<ValueEntryRef<'a>>,
	type_: Option<TypeRef<'a, T>>,
	language: Option<&'a LenientLangTag>,
	direction: Option<Direction>,
}

impl<'a, T> Iterator for Entries<'a, T> {
	type Item = EntryRef<'a, T>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let mut len = 0;

		if self.value.is_some() {
			len += 1
		}

		if self.type_.is_some() {
			len += 1
		}

		if self.language.is_some() {
			len += 1
		}

		if self.direction.is_some() {
			len += 1
		}

		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.value.take().map(EntryRef::Value).or_else(|| {
			self.type_.take().map(EntryRef::Type).or_else(|| {
				self.language
					.take()
					.map(EntryRef::Language)
					.or_else(|| self.direction.take().map(EntryRef::Direction))
			})
		})
	}
}

impl<'a, T> ExactSizeIterator for Entries<'a, T> {}

impl<'a, T> DoubleEndedIterator for Entries<'a, T> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.direction.take().map(EntryRef::Direction).or_else(|| {
			self.language.take().map(EntryRef::Language).or_else(|| {
				self.type_
					.take()
					.map(EntryRef::Type)
					.or_else(|| self.value.take().map(EntryRef::Value))
			})
		})
	}
}

/// Reference to any fragment that can appear in a value object.
pub enum FragmentRef<'a, T> {
	/// Value object entry.
	Entry(EntryRef<'a, T>),

	/// Value object entry key.
	Key(EntryKey),

	/// Value object entry value.
	Value(EntryValueRef<'a, T>),

	/// JSON fragment in a "@json" typed value.
	JsonFragment(json_syntax::FragmentRef<'a>),
}

impl<'a, T> FragmentRef<'a, T> {
	pub fn into_iri(self) -> Option<&'a T> {
		match self {
			Self::Value(EntryValueRef::Type(TypeRef::Id(id))) => Some(id),
			_ => None,
		}
	}

	pub fn as_iri(&self) -> Option<&'a T> {
		match self {
			Self::Value(EntryValueRef::Type(TypeRef::Id(id))) => Some(id),
			_ => None,
		}
	}

	pub fn is_json_array(&self) -> bool {
		match self {
			Self::Value(EntryValueRef::Value(ValueEntryRef::Json(json))) => json.is_array(),
			Self::JsonFragment(json) => json.is_array(),
			_ => false,
		}
	}

	pub fn is_json_object(&self) -> bool {
		match self {
			Self::Value(EntryValueRef::Value(ValueEntryRef::Json(json))) => json.is_object(),
			Self::JsonFragment(json) => json.is_object(),
			_ => false,
		}
	}

	pub fn sub_fragments(&self) -> SubFragments<'a, T> {
		match self {
			Self::Entry(e) => SubFragments::Entry(Some(e.key()), Some(e.value())),
			Self::Value(EntryValueRef::Value(ValueEntryRef::Json(json))) => match json {
				json_syntax::Value::Array(a) => {
					SubFragments::JsonFragment(json_syntax::SubFragments::Array(a.iter()))
				}
				json_syntax::Value::Object(o) => {
					SubFragments::JsonFragment(json_syntax::SubFragments::Object(o.iter()))
				}
				_ => SubFragments::None(PhantomData),
			},
			Self::JsonFragment(f) => SubFragments::JsonFragment(f.sub_fragments()),
			_ => SubFragments::None(PhantomData),
		}
	}
}

pub enum SubFragments<'a, T> {
	None(PhantomData<T>),
	Entry(Option<EntryKey>, Option<EntryValueRef<'a, T>>),
	JsonFragment(json_syntax::SubFragments<'a>),
}

impl<'a, T: 'a> Iterator for SubFragments<'a, T> {
	type Item = FragmentRef<'a, T>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None(_) => None,
			Self::Entry(k, v) => k
				.take()
				.map(FragmentRef::Key)
				.or_else(|| v.take().map(FragmentRef::Value)),
			Self::JsonFragment(f) => f.next().map(|v| FragmentRef::JsonFragment(v)),
		}
	}
}

impl<T, N: IriVocabulary<Iri = T>> IntoJsonWithContext<N> for Value<T> {
	fn into_json_with(self, vocabulary: &N) -> json_syntax::Value {
		let mut obj = json_syntax::Object::new();

		let value = match self {
			Self::Literal(lit, ty) => {
				if let Some(ty) = ty {
					obj.insert("@type".into(), vocabulary.iri(&ty).unwrap().as_str().into());
				}

				lit.into_json()
			}
			Self::LangString(s) => {
				if let Some(language) = s.language() {
					obj.insert("@language".into(), language.as_str().into());
				}

				if let Some(direction) = s.direction() {
					obj.insert("@direction".into(), direction.as_str().into());
				}

				s.as_str().into()
			}
			Self::Json(json) => {
				obj.insert("@type".into(), "@json".into());

				json
			}
		};

		obj.insert("@value".into(), value);
		obj.into()
	}
}
