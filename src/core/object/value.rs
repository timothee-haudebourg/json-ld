use crate::syntax::Keyword;
use crate::{object, Direction, LangString, LenientLangTag, Type};
use educe::Educe;
use iref::{Iri, IriBuf};
use json_syntax::{JsonString, Number, NumberBuf};
use rdf_types::{Literal, LiteralType, RDF_JSON};
use std::hash::Hash;
use xsd_types::{XSD_BOOLEAN, XSD_FLOAT, XSD_INTEGER};

/// Value type.
pub enum ValueType {
	Json,
	Id(IriBuf),
}

impl ValueType {
	pub fn as_id(&self) -> Option<crate::id::Ref> {
		match self {
			Self::Json => None,
			Self::Id(t) => Some(crate::id::Ref::Iri(t)),
		}
	}
}

/// Value type reference.
#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum ValueTypeRef<'a> {
	Json,
	Id(&'a Iri),
}

impl<'a> ValueTypeRef<'a> {
	pub fn into_object_type(self) -> Type {
		match self {
			Self::Json => Type::Json,
			Self::Id(id) => Type::Iri(id.to_owned()),
		}
	}

	pub fn into_reference(self) -> Option<crate::id::Ref<'a>> {
		match self {
			Self::Json => None,
			Self::Id(t) => Some(crate::id::Ref::Iri(t)),
		}
	}
}

/// Literal value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LiteralValue {
	/// The `null` value.
	Null,

	/// Boolean value.
	Boolean(bool),

	/// Number.
	Number(NumberBuf),

	/// String.
	String(JsonString),
}

impl LiteralValue {
	/// Returns this value as a string if it is one.
	#[inline(always)]
	pub fn as_str(&self) -> Option<&str> {
		match self {
			LiteralValue::String(s) => Some(s.as_ref()),
			_ => None,
		}
	}

	/// Returns this value as a boolean if it is one.
	#[inline(always)]
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			LiteralValue::Boolean(b) => Some(*b),
			_ => None,
		}
	}

	/// Returns this value as a number if it is one.
	#[inline(always)]
	pub fn as_number(&self) -> Option<&Number> {
		match self {
			LiteralValue::Number(n) => Some(n),
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

	// /// Puts this literal into canonical form using the given `buffer`.
	// ///
	// /// The buffer is used to compute the canonical form of numbers.
	// pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer) {
	// 	if let Self::Number(n) = self {
	// 		*n = NumberBuf::from_number(n.canonical_with(buffer))
	// 	}
	// }

	// /// Puts this literal into canonical form.
	// pub fn canonicalize(&mut self) {
	// 	let mut buffer = ryu_js::Buffer::new();
	// 	self.canonicalize_with(&mut buffer)
	// }
}

/// Value object.
///
/// Either a typed literal value, or an internationalized language string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueObject {
	/// Typed literal value.
	Literal(LiteralValue, Option<IriBuf>),

	/// Language tagged string.
	LangString(LangString),

	/// JSON literal value.
	Json(json_syntax::Value),
}

impl ValueObject {
	/// Creates a `null` value object.
	#[inline(always)]
	pub fn null() -> Self {
		Self::Literal(LiteralValue::Null, None)
	}

	#[inline(always)]
	pub fn as_str(&self) -> Option<&str> {
		match self {
			ValueObject::Literal(lit, _) => lit.as_str(),
			ValueObject::LangString(str) => Some(str.as_str()),
			ValueObject::Json(_) => None,
		}
	}

	#[inline(always)]
	pub fn as_literal(&self) -> Option<(&LiteralValue, Option<&Iri>)> {
		match self {
			Self::Literal(lit, ty) => Some((lit, ty.as_deref())),
			_ => None,
		}
	}

	pub fn literal_type(&self) -> Option<&Iri> {
		match self {
			Self::Literal(_, ty) => ty.as_deref(),
			_ => None,
		}
	}

	/// Set the literal value type, and returns the old type.
	///
	/// Has no effect and return `None` if the value is not a literal value.
	pub fn set_literal_type(&mut self, mut ty: Option<IriBuf>) -> Option<IriBuf> {
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
	pub fn map_literal_type<F: FnOnce(Option<IriBuf>) -> Option<IriBuf>>(&mut self, f: F) {
		if let Self::Literal(_, ty) = self {
			*ty = f(ty.take())
		}
	}

	#[inline(always)]
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			ValueObject::Literal(lit, _) => lit.as_bool(),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn as_number(&self) -> Option<&Number> {
		match self {
			ValueObject::Literal(lit, _) => lit.as_number(),
			_ => None,
		}
	}

	/// Return the type of the value if any.
	///
	/// This will return `Some(Type::Json)` for JSON literal values.
	pub fn typ(&self) -> Option<ValueTypeRef> {
		match self {
			ValueObject::Literal(_, Some(ty)) => Some(ValueTypeRef::Id(ty)),
			ValueObject::Json(_) => Some(ValueTypeRef::Json),
			_ => None,
		}
	}

	/// If the value is a language tagged string, return its associated language if any.
	///
	/// Returns `None` if the value is not a language tagged string.
	#[inline(always)]
	pub fn language(&self) -> Option<&LenientLangTag> {
		match self {
			ValueObject::LangString(tag) => tag.language(),
			_ => None,
		}
	}

	/// If the value is a language tagged string, return its associated direction if any.
	///
	/// Returns `None` if the value is not a language tagged string.
	#[inline(always)]
	pub fn direction(&self) -> Option<Direction> {
		match self {
			ValueObject::LangString(str) => str.direction(),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn entries(&self) -> Entries {
		match self {
			Self::Literal(l, ty) => Entries {
				value: Some(ValueEntryRef::Literal(l)),
				type_: ty.as_deref().map(ValueTypeRef::Id),
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
				type_: Some(ValueTypeRef::Json),
				language: None,
				direction: None,
			},
		}
	}

	// /// Puts this value object literal into canonical form using the given
	// /// `buffer`.
	// ///
	// /// The buffer is used to compute the canonical form of numbers.
	// pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer) {
	// 	match self {
	// 		Self::Json(json) => json.canonicalize_with(buffer),
	// 		Self::Literal(l, _) => l.canonicalize_with(buffer),
	// 		Self::LangString(_) => (),
	// 	}
	// }

	// /// Puts this literal into canonical form.
	// pub fn canonicalize(&mut self) {
	// 	let mut buffer = ryu_js::Buffer::new();
	// 	self.canonicalize_with(&mut buffer)
	// }
}

impl object::AnyObject for ValueObject {
	#[inline(always)]
	fn as_ref(&self) -> object::Ref {
		object::Ref::Value(self)
	}
}

impl From<Literal> for ValueObject {
	fn from(literal: Literal) -> Self {
		match literal.type_ {
			LiteralType::Any(ty) => {
				if ty == XSD_BOOLEAN {
					match literal.value.as_str() {
						"true" => return Self::Literal(LiteralValue::Boolean(true), Some(ty)),
						"false" => return Self::Literal(LiteralValue::Boolean(false), Some(ty)),
						_ => (),
					}
				} else if ty == XSD_INTEGER || ty == XSD_FLOAT {
					if let Ok(number) = literal.value.parse() {
						return Self::Literal(LiteralValue::Number(number), Some(ty));
					}
				} else if ty == RDF_JSON {
					if let Ok(json) = json_syntax::from_str(&literal.value) {
						return Self::Json(json);
					}
				}

				Self::Literal(
					LiteralValue::String(literal.value.into()),
					Some(RDF_JSON.to_owned()),
				)
			}
			LiteralType::LangString(langtag) => {
				Self::LangString(LangString::new_with_language(literal.value, langtag))
			}
		}
	}
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum EntryRef<'a> {
	Value(ValueEntryRef<'a>),
	Type(ValueTypeRef<'a>),
	Language(&'a LenientLangTag),
	Direction(Direction),
}

impl<'a> EntryRef<'a> {
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

	pub fn into_value(self) -> EntryValueRef<'a> {
		match self {
			Self::Value(v) => EntryValueRef::Value(v),
			Self::Type(v) => EntryValueRef::Type(v),
			Self::Language(v) => EntryValueRef::Language(v),
			Self::Direction(v) => EntryValueRef::Direction(v),
		}
	}

	pub fn value(&self) -> EntryValueRef<'a> {
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
pub enum EntryValueRef<'a> {
	Value(ValueEntryRef<'a>),
	Type(ValueTypeRef<'a>),
	Language(&'a LenientLangTag),
	Direction(Direction),
}

pub enum ValueEntryRef<'a> {
	Literal(&'a LiteralValue),
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
pub struct Entries<'a> {
	value: Option<ValueEntryRef<'a>>,
	type_: Option<ValueTypeRef<'a>>,
	language: Option<&'a LenientLangTag>,
	direction: Option<Direction>,
}

impl<'a> Iterator for Entries<'a> {
	type Item = EntryRef<'a>;

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

impl<'a> ExactSizeIterator for Entries<'a> {}

impl<'a> DoubleEndedIterator for Entries<'a> {
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
