use crate::syntax::Keyword;
use crate::{object, Direction, LangString, LenientLangTag, Type};
use educe::Educe;
use json_syntax::{JsonNumber, JsonNumberBuf, JsonValue};
use rdf_syntax::{Iri, IriBuf};
use rdf_syntax::{Literal, RDF_JSON};
use std::hash::Hash;
use xsd_types::{XSD_BOOLEAN, XSD_FLOAT, XSD_INTEGER};

/// Value type.
pub enum ValueType {
	Json,
	Id(IriBuf),
}

impl ValueType {
	pub fn as_id(&self) -> Option<crate::id::Ref<'_>> {
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

/// Literal type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LiteralType {
	/// JSON literal (`@json`).
	Json,

	/// Typed by an IRI.
	Iri(IriBuf),
}

impl LiteralType {
	/// Returns this type IRI.
	///
	/// If the type is `@json`, this will return [`RDF_JSON`].
	pub fn iri(&self) -> &Iri {
		match self {
			Self::Json => RDF_JSON,
			Self::Iri(iri) => iri,
		}
	}

	/// Returns whether this is the JSON type.
	///
	/// Returns `true` if it is `@json` or the [`RDF_JSON`] IRI.
	pub fn is_json(&self) -> bool {
		match self {
			Self::Json => true,
			Self::Iri(iri) => iri == RDF_JSON,
		}
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for LiteralType {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::Json => serializer.serialize_str("@json"),
			Self::Iri(iri) => serializer.serialize_str(iri.as_str()),
		}
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for LiteralType {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let s = String::deserialize(deserializer)?;
		if s == "@json" {
			Ok(Self::Json)
		} else {
			IriBuf::new(s)
				.map(Self::Iri)
				.map_err(serde::de::Error::custom)
		}
	}
}

/// Literal value.
///
/// A JSON-LD value object with a `@value` entry and an optional `@type`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LiteralValue {
	/// The value.
	#[cfg_attr(feature = "serde", serde(rename = "@value"))]
	pub value: JsonValue,

	/// Optional type.
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@type", default, skip_serializing_if = "Option::is_none")
	)]
	pub type_: Option<LiteralType>,
}

impl LiteralValue {
	/// Creates a new literal value.
	pub fn new(value: JsonValue, type_: Option<LiteralType>) -> Self {
		Self { value, type_ }
	}

	/// Creates a null literal value.
	pub fn null() -> Self {
		Self::new(JsonValue::Null, None)
	}

	/// Creates a JSON literal value (`@type: "@json"`).
	pub fn json(value: JsonValue) -> Self {
		Self::new(value, Some(LiteralType::Json))
	}

	/// Returns the value as a string if it is one.
	#[inline(always)]
	pub fn as_str(&self) -> Option<&str> {
		self.value.as_str().map(AsRef::as_ref)
	}

	/// Returns the value as a boolean if it is one.
	#[inline(always)]
	pub fn as_bool(&self) -> Option<bool> {
		self.value.as_boolean()
	}

	/// Returns the value as a number if it is one.
	#[inline(always)]
	pub fn as_number(&self) -> Option<&JsonNumber> {
		self.value.as_number()
	}

	/// Returns the type IRI.
	pub fn type_iri(&self) -> Option<&Iri> {
		match &self.type_ {
			Some(LiteralType::Iri(iri)) => Some(iri),
			_ => None,
		}
	}

	/// Returns true if this is a JSON literal.
	pub fn is_json(&self) -> bool {
		self.type_.as_ref().is_some_and(LiteralType::is_json)
	}

	// /// Puts this literal into canonical form using the given `buffer`.
	// ///
	// /// The buffer is used to compute the canonical form of numbers.
	// pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer) {
	// 	if let JsonValue::Number(n) = &mut self.value {
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum ValueObject {
	/// Language tagged string.
	LangString(LangString),

	/// Typed literal value (including JSON literals).
	Literal(LiteralValue),
}

impl ValueObject {
	/// Creates a `null` value object.
	#[inline(always)]
	pub fn null() -> Self {
		Self::Literal(LiteralValue::null())
	}

	#[inline(always)]
	pub fn as_str(&self) -> Option<&str> {
		match self {
			ValueObject::Literal(lit) => lit.as_str(),
			ValueObject::LangString(s) => Some(s.as_str()),
		}
	}

	#[inline(always)]
	pub fn as_literal(&self) -> Option<&LiteralValue> {
		match self {
			Self::Literal(lit) => Some(lit),
			_ => None,
		}
	}

	pub fn literal_type(&self) -> Option<&LiteralType> {
		match self {
			Self::Literal(lit) => lit.type_.as_ref(),
			_ => None,
		}
	}

	pub fn literal_type_iri(&self) -> Option<&Iri> {
		match self {
			Self::Literal(lit) => lit.type_iri(),
			_ => None,
		}
	}

	/// Set the literal value type, and returns the old type.
	///
	/// Has no effect and return `None` if the value is not a literal value.
	pub fn set_literal_type(&mut self, ty: Option<LiteralType>) -> Option<LiteralType> {
		match self {
			Self::Literal(lit) => std::mem::replace(&mut lit.type_, ty),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			ValueObject::Literal(lit) => lit.as_bool(),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn as_number(&self) -> Option<&JsonNumber> {
		match self {
			ValueObject::Literal(lit) => lit.as_number(),
			_ => None,
		}
	}

	/// Return the type of the value if any.
	pub fn typ(&self) -> Option<ValueTypeRef<'_>> {
		match self {
			ValueObject::Literal(lit) => match &lit.type_ {
				Some(LiteralType::Json) => Some(ValueTypeRef::Json),
				Some(LiteralType::Iri(iri)) => Some(ValueTypeRef::Id(iri)),
				None => None,
			},
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
			ValueObject::LangString(s) => s.direction(),
			_ => None,
		}
	}

	/// Returns true if this is a JSON literal.
	pub fn is_json(&self) -> bool {
		matches!(self, Self::Literal(lit) if lit.is_json())
	}

	/// Returns the JSON value if this is a JSON literal.
	pub fn as_json(&self) -> Option<&JsonValue> {
		match self {
			Self::Literal(lit) if lit.is_json() => Some(&lit.value),
			_ => None,
		}
	}

	// /// Puts this value object literal into canonical form using the given
	// /// `buffer`.
	// ///
	// /// The buffer is used to compute the canonical form of numbers.
	// pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer) {
	// 	match self {
	// 		Self::Literal(l) => l.canonicalize_with(buffer),
	// 		Self::LangString(_) => (),
	// 	}
	// }

	// /// Puts this literal into canonical form.
	// pub fn canonicalize(&mut self) {
	// 	let mut buffer = ryu_js::Buffer::new();
	// 	self.canonicalize_with(&mut buffer)
	// }

	#[inline(always)]
	pub fn entries(&self) -> Entries<'_> {
		match self {
			Self::Literal(lit) => Entries {
				value: Some(ValueEntryRef::Value(&lit.value)),
				type_: match &lit.type_ {
					Some(LiteralType::Json) => Some(ValueTypeRef::Json),
					Some(LiteralType::Iri(iri)) => Some(ValueTypeRef::Id(iri)),
					None => None,
				},
				language: None,
				direction: None,
			},
			Self::LangString(l) => Entries {
				value: Some(ValueEntryRef::LangString(l.as_str())),
				type_: None,
				language: l.language(),
				direction: l.direction(),
			},
		}
	}
}

impl object::AnyObject for ValueObject {
	#[inline(always)]
	fn as_ref(&self) -> object::Ref<'_> {
		object::Ref::Value(self)
	}
}

impl From<Literal> for ValueObject {
	fn from(literal: Literal) -> Self {
		match literal.type_ {
			rdf_syntax::LiteralType::Any(ty) => {
				if ty == XSD_BOOLEAN {
					match literal.value.as_str() {
						"true" => {
							return Self::Literal(LiteralValue::new(
								JsonValue::Boolean(true),
								Some(LiteralType::Iri(ty)),
							))
						}
						"false" => {
							return Self::Literal(LiteralValue::new(
								JsonValue::Boolean(false),
								Some(LiteralType::Iri(ty)),
							))
						}
						_ => (),
					}
				} else if ty == XSD_INTEGER || ty == XSD_FLOAT {
					if let Ok(number) = literal.value.parse::<JsonNumberBuf>() {
						return Self::Literal(LiteralValue::new(
							JsonValue::Number(number),
							Some(LiteralType::Iri(ty)),
						));
					}
				} else if ty == RDF_JSON {
					if let Ok(json) = json_syntax::from_str(&literal.value) {
						return Self::Literal(LiteralValue::json(json));
					}
				}

				Self::Literal(LiteralValue::new(
					JsonValue::String(literal.value.into()),
					Some(LiteralType::Iri(ty)),
				))
			}
			rdf_syntax::LiteralType::LangString(langtag) => {
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
	Value(&'a JsonValue),
	LangString(&'a str),
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
