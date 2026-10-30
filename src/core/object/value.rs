use crate::object::{JsonLdObjectMut, JsonLdObjectRef};
use crate::syntax::JsonLdKeyword;
use crate::{JsonLdDirection, JsonLdLangString, JsonLdLenient, JsonLdType, JsonLdVisit, object};
use educe::Educe;
use json_syntax::{JsonNumber, JsonNumberBuf, JsonValue, ryu_js};
use langtag::LangTag;
use rdf_syntax::{IdRef, Iri, IriBuf};
use rdf_syntax::{Literal, RDF_JSON};
use std::hash::Hash;
use xsd_types::{XSD_BOOLEAN, XSD_FLOAT, XSD_INTEGER};

/// Value type.
pub enum JsonLdValueType {
	Json,
	Id(IriBuf),
}

impl JsonLdValueType {
	pub fn as_id(&self) -> Option<IdRef<'_>> {
		match self {
			Self::Json => None,
			Self::Id(t) => Some(IdRef::Iri(t)),
		}
	}
}

/// Value type reference.
#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum JsonLdValueTypeRef<'a> {
	Json,
	Id(&'a Iri),
}

impl<'a> JsonLdValueTypeRef<'a> {
	pub fn into_object_type(self) -> JsonLdType {
		match self {
			Self::Json => JsonLdType::Json,
			Self::Id(id) => JsonLdType::Iri(id.to_owned()),
		}
	}

	pub fn into_id_ref(self) -> Option<IdRef<'a>> {
		match self {
			Self::Json => None,
			Self::Id(t) => Some(IdRef::Iri(t)),
		}
	}
}

/// Literal type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum JsonLdLiteralType {
	/// JSON literal (`@json`).
	Json,

	/// Typed by an IRI.
	Iri(IriBuf),
}

impl JsonLdLiteralType {
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
impl serde::Serialize for JsonLdLiteralType {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::Json => serializer.serialize_str("@json"),
			Self::Iri(iri) => serializer.serialize_str(iri.as_str()),
		}
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for JsonLdLiteralType {
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JsonLdLiteralValue {
	/// The value.
	#[cfg_attr(feature = "serde", serde(rename = "@value"))]
	pub value: JsonValue,

	/// Optional type.
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@type", default, skip_serializing_if = "Option::is_none")
	)]
	pub type_: Option<JsonLdLiteralType>,
}

impl JsonLdLiteralValue {
	/// Creates a new literal value.
	pub fn new(value: JsonValue, type_: Option<JsonLdLiteralType>) -> Self {
		Self { value, type_ }
	}

	/// Creates a null literal value.
	pub fn null() -> Self {
		Self::new(JsonValue::Null, None)
	}

	/// Creates a JSON literal value (`@type: "@json"`).
	pub fn json(value: JsonValue) -> Self {
		Self::new(value, Some(JsonLdLiteralType::Json))
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
			Some(JsonLdLiteralType::Iri(iri)) => Some(iri),
			_ => None,
		}
	}

	/// Returns true if this is a JSON literal.
	pub fn is_json(&self) -> bool {
		self.type_.as_ref().is_some_and(JsonLdLiteralType::is_json)
	}

	/// Puts this literal into canonical form using the given `buffer`.
	///
	/// The buffer is used to compute the canonical form of numbers.
	pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer) {
		if let JsonValue::Number(n) = &mut self.value {
			*n = JsonNumberBuf::from_number(n.canonicalized_with(buffer))
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum JsonLdValue {
	/// Language tagged string.
	LangString(JsonLdLangString),

	/// Typed literal value (including JSON literals).
	Literal(JsonLdLiteralValue),
}

impl JsonLdValue {
	/// Creates a `null` value object.
	#[inline(always)]
	pub fn null() -> Self {
		Self::Literal(JsonLdLiteralValue::null())
	}

	#[inline(always)]
	pub fn as_str(&self) -> Option<&str> {
		match self {
			JsonLdValue::Literal(lit) => lit.as_str(),
			JsonLdValue::LangString(s) => Some(s.as_str()),
		}
	}

	#[inline(always)]
	pub fn as_literal(&self) -> Option<&JsonLdLiteralValue> {
		match self {
			Self::Literal(lit) => Some(lit),
			_ => None,
		}
	}

	pub fn literal_type(&self) -> Option<&JsonLdLiteralType> {
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
	pub fn set_literal_type(&mut self, ty: Option<JsonLdLiteralType>) -> Option<JsonLdLiteralType> {
		match self {
			Self::Literal(lit) => std::mem::replace(&mut lit.type_, ty),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			JsonLdValue::Literal(lit) => lit.as_bool(),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn as_number(&self) -> Option<&JsonNumber> {
		match self {
			JsonLdValue::Literal(lit) => lit.as_number(),
			_ => None,
		}
	}

	/// Return the type of the value if any.
	pub fn typ(&self) -> Option<JsonLdValueTypeRef<'_>> {
		match self {
			JsonLdValue::Literal(lit) => match &lit.type_ {
				Some(JsonLdLiteralType::Json) => Some(JsonLdValueTypeRef::Json),
				Some(JsonLdLiteralType::Iri(iri)) => Some(JsonLdValueTypeRef::Id(iri)),
				None => None,
			},
			_ => None,
		}
	}

	/// If the value is a language tagged string, return its associated language if any.
	///
	/// Returns `None` if the value is not a language tagged string.
	#[inline(always)]
	pub fn language(&self) -> Option<JsonLdLenient<&LangTag>> {
		match self {
			JsonLdValue::LangString(tag) => tag.language(),
			_ => None,
		}
	}

	/// If the value is a language tagged string, return its associated direction if any.
	///
	/// Returns `None` if the value is not a language tagged string.
	#[inline(always)]
	pub fn direction(&self) -> Option<JsonLdDirection> {
		match self {
			JsonLdValue::LangString(s) => s.direction(),
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

	/// Puts this value object literal into canonical form using the given
	/// `buffer`.
	///
	/// The buffer is used to compute the canonical form of numbers.
	pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer) {
		match self {
			Self::Literal(l) => l.canonicalize_with(buffer),
			Self::LangString(_) => (),
		}
	}

	/// Puts this literal into canonical form.
	pub fn canonicalize(&mut self) {
		let mut buffer = ryu_js::Buffer::new();
		self.canonicalize_with(&mut buffer)
	}

	#[inline(always)]
	pub fn entries(&self) -> JsonLdValueEntries<'_> {
		match self {
			Self::Literal(lit) => JsonLdValueEntries {
				value: Some(JsonLdValueValueEntryRef::Value(&lit.value)),
				type_: match &lit.type_ {
					Some(JsonLdLiteralType::Json) => Some(JsonLdValueTypeRef::Json),
					Some(JsonLdLiteralType::Iri(iri)) => Some(JsonLdValueTypeRef::Id(iri)),
					None => None,
				},
				language: None,
				direction: None,
			},
			Self::LangString(l) => JsonLdValueEntries {
				value: Some(JsonLdValueValueEntryRef::LangString(l.as_str())),
				type_: None,
				language: l.language(),
				direction: l.direction(),
			},
		}
	}
}

impl JsonLdVisit for JsonLdValue {
	fn visit_with(&self, f: &mut impl FnMut(JsonLdObjectRef)) {
		f(JsonLdObjectRef::Value(self))
	}

	fn visit_mut_with(&mut self, f: &mut impl FnMut(JsonLdObjectMut)) {
		f(JsonLdObjectMut::Value(self))
	}
}

impl object::AnyJsonLdObject for JsonLdValue {
	#[inline(always)]
	fn as_ref(&self) -> object::JsonLdObjectRef<'_> {
		object::JsonLdObjectRef::Value(self)
	}
}

impl From<Literal> for JsonLdValue {
	fn from(literal: Literal) -> Self {
		match literal.r#type {
			rdf_syntax::LiteralType::Any(ty) => {
				if ty == XSD_BOOLEAN {
					match literal.value.as_str() {
						"true" => {
							return Self::Literal(JsonLdLiteralValue::new(
								JsonValue::Boolean(true),
								Some(JsonLdLiteralType::Iri(ty)),
							));
						}
						"false" => {
							return Self::Literal(JsonLdLiteralValue::new(
								JsonValue::Boolean(false),
								Some(JsonLdLiteralType::Iri(ty)),
							));
						}
						_ => (),
					}
				} else if ty == XSD_INTEGER || ty == XSD_FLOAT {
					if let Ok(number) = literal.value.parse::<JsonNumberBuf>() {
						return Self::Literal(JsonLdLiteralValue::new(
							JsonValue::Number(number),
							Some(JsonLdLiteralType::Iri(ty)),
						));
					}
				} else if ty == RDF_JSON
					&& let Ok(json) = json_syntax::from_str(&literal.value)
				{
					return Self::Literal(JsonLdLiteralValue::json(json));
				}

				Self::Literal(JsonLdLiteralValue::new(
					JsonValue::String(literal.value.into()),
					Some(JsonLdLiteralType::Iri(ty)),
				))
			}
			rdf_syntax::LiteralType::LangString(langtag) => {
				Self::LangString(JsonLdLangString::from_language(literal.value, langtag))
			}
		}
	}
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum JsonLdValueEntryRef<'a> {
	Value(JsonLdValueValueEntryRef<'a>),
	Type(JsonLdValueTypeRef<'a>),
	Language(JsonLdLenient<&'a LangTag>),
	Direction(JsonLdDirection),
}

impl<'a> JsonLdValueEntryRef<'a> {
	pub fn into_key(self) -> JsonLdValueEntryKey {
		match self {
			Self::Value(_) => JsonLdValueEntryKey::Value,
			Self::Type(_) => JsonLdValueEntryKey::Type,
			Self::Language(_) => JsonLdValueEntryKey::Language,
			Self::Direction(_) => JsonLdValueEntryKey::Direction,
		}
	}

	pub fn key(&self) -> JsonLdValueEntryKey {
		self.into_key()
	}

	pub fn into_value(self) -> JsonLdValueEntryValueRef<'a> {
		match self {
			Self::Value(v) => JsonLdValueEntryValueRef::Value(v),
			Self::Type(v) => JsonLdValueEntryValueRef::Type(v),
			Self::Language(v) => JsonLdValueEntryValueRef::Language(v),
			Self::Direction(v) => JsonLdValueEntryValueRef::Direction(v),
		}
	}

	pub fn value(&self) -> JsonLdValueEntryValueRef<'a> {
		match self {
			Self::Value(v) => JsonLdValueEntryValueRef::Value(*v),
			Self::Type(v) => JsonLdValueEntryValueRef::Type(*v),
			Self::Language(v) => JsonLdValueEntryValueRef::Language(*v),
			Self::Direction(v) => JsonLdValueEntryValueRef::Direction(*v),
		}
	}
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum JsonLdValueEntryValueRef<'a> {
	Value(JsonLdValueValueEntryRef<'a>),
	Type(JsonLdValueTypeRef<'a>),
	Language(JsonLdLenient<&'a LangTag>),
	Direction(JsonLdDirection),
}

pub enum JsonLdValueValueEntryRef<'a> {
	Value(&'a JsonValue),
	LangString(&'a str),
}

impl<'a> Clone for JsonLdValueValueEntryRef<'a> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<'a> Copy for JsonLdValueValueEntryRef<'a> {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JsonLdValueEntryKey {
	Value,
	Type,
	Language,
	Direction,
}

impl JsonLdValueEntryKey {
	pub fn into_keyword(self) -> JsonLdKeyword {
		match self {
			Self::Value => JsonLdKeyword::Value,
			Self::Type => JsonLdKeyword::Type,
			Self::Language => JsonLdKeyword::Language,
			Self::Direction => JsonLdKeyword::Direction,
		}
	}

	pub fn as_keyword(&self) -> JsonLdKeyword {
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
pub struct JsonLdValueEntries<'a> {
	value: Option<JsonLdValueValueEntryRef<'a>>,
	type_: Option<JsonLdValueTypeRef<'a>>,
	language: Option<JsonLdLenient<&'a LangTag>>,
	direction: Option<JsonLdDirection>,
}

impl<'a> Iterator for JsonLdValueEntries<'a> {
	type Item = JsonLdValueEntryRef<'a>;

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
		self.value
			.take()
			.map(JsonLdValueEntryRef::Value)
			.or_else(|| {
				self.type_
					.take()
					.map(JsonLdValueEntryRef::Type)
					.or_else(|| {
						self.language
							.take()
							.map(JsonLdValueEntryRef::Language)
							.or_else(|| self.direction.take().map(JsonLdValueEntryRef::Direction))
					})
			})
	}
}

impl<'a> ExactSizeIterator for JsonLdValueEntries<'a> {}

impl<'a> DoubleEndedIterator for JsonLdValueEntries<'a> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.direction
			.take()
			.map(JsonLdValueEntryRef::Direction)
			.or_else(|| {
				self.language
					.take()
					.map(JsonLdValueEntryRef::Language)
					.or_else(|| {
						self.type_
							.take()
							.map(JsonLdValueEntryRef::Type)
							.or_else(|| self.value.take().map(JsonLdValueEntryRef::Value))
					})
			})
	}
}
