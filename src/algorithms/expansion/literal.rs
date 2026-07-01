use std::borrow::Cow;

use json_syntax::{JsonNumber, JsonNumberBuf, JsonValue};

use crate::{
	IndexedObject, JsonLdError, LangString, Lenient, NodeObject, Nullable, Object, Type,
	ValueObject,
	algorithms::{JsonLdLocated, JsonLdLocatedError, JsonLdLocationStack, Warning},
	object::{LiteralValue, value::LiteralType},
};

use super::{Expander, node_id_of_term};

pub enum ExpandableLiteralValue<'a> {
	Boolean(bool),
	Number(&'a JsonNumber),
	String(Cow<'a, str>),
}

impl<'a> ExpandableLiteralValue<'a> {
	pub fn new(value: &'a JsonValue) -> Self {
		match value {
			JsonValue::Boolean(b) => Self::Boolean(*b),
			JsonValue::Number(n) => Self::Number(n),
			JsonValue::String(s) => Self::String(Cow::Borrowed(s)),
			_ => panic!("not a literal value"),
		}
	}

	pub fn is_string(&self) -> bool {
		matches!(self, Self::String(_))
	}

	pub fn as_str(&self) -> Option<&str> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}
}

impl<'a> From<&'a str> for ExpandableLiteralValue<'a> {
	fn from(value: &'a str) -> Self {
		Self::String(Cow::Borrowed(value))
	}
}

impl<'a> Expander<'a> {
	/// Expand a literal value.
	/// See <https://www.w3.org/TR/json-ld11-api/#value-expansion>.
	pub fn expand_literal(
		&self,
		warn: impl FnOnce(Warning),
		value: ExpandableLiteralValue,
		location: JsonLdLocationStack<'_>,
	) -> Result<IndexedObject, JsonLdLocatedError> {
		let active_property_definition = self.active_property_definition();
		let active_property_type =
			if let Some(active_property_definition) = active_property_definition {
				active_property_definition.typ().cloned()
			} else {
				None
			};

		match active_property_type {
			// If the `active_property` has a type mapping in `active_context` that is `@id`, and the
			// `value` is a string, return a new map containing a single entry where the key is `@id` and
			// the value is the result of IRI expanding `value` using `true` for `document_relative` and
			// `false` for vocab.
			Some(Type::Id) if value.is_string() => {
				let mut node = NodeObject::new();
				let id = node_id_of_term(self.active_context.expand_iri_with(
					Nullable::Some(value.as_str().unwrap().into()),
					true,
					false,
					warn,
				));

				node.id = id;
				Ok(Object::node(node).into())
			}

			// If `active_property` has a type mapping in active context that is `@vocab`, and the
			// value is a string, return a new map containing a single entry where the key is
			// `@id` and the value is the result of IRI expanding `value` using `true` for
			// document relative.
			Some(Type::Vocab) if value.is_string() => {
				let mut node = NodeObject::new();
				let id = node_id_of_term(self.active_context.expand_iri_with(
					Nullable::Some(value.as_str().unwrap().into()),
					true,
					true,
					warn,
				));

				node.id = id;
				Ok(Object::node(node).into())
			}

			_ => {
				// Otherwise, initialize `result` to a map with an `@value` entry whose value is set to
				// `value`.
				let result_value: JsonValue = match value {
					ExpandableLiteralValue::Boolean(b) => JsonValue::Boolean(b),
					ExpandableLiteralValue::Number(n) => JsonValue::Number(unsafe {
						JsonNumberBuf::new_unchecked(n.as_bytes().into())
					}),
					ExpandableLiteralValue::String(s) => JsonValue::String(s.into_owned().into()),
				};

				// If `active_property` has a type mapping in active context, other than `@id`,
				// `@vocab`, or `@none`, add `@type` to `result` and set its value to the value
				// associated with the type mapping.
				let mut ty = None;
				match active_property_type {
					None | Some(Type::Id) | Some(Type::Vocab) | Some(Type::None) => {
						// Otherwise, if value is a string:
						if let JsonValue::String(s) = result_value {
							// Initialize `language` to the language mapping for
							// `active_property` in `active_context`, if any, otherwise to the
							// default language of `active_context`.
							let language = if let Some(active_property_definition) =
								active_property_definition
							{
								if let Some(language) = active_property_definition.language() {
									language.cloned().option()
								} else {
									self.active_context
										.default_language()
										.map(Lenient::into_owned)
								}
							} else {
								self.active_context
									.default_language()
									.map(Lenient::into_owned)
							};

							// Initialize `direction` to the direction mapping for
							// `active_property` in `active_context`, if any, otherwise to the
							// default base direction of `active_context`.
							let direction = if let Some(active_property_definition) =
								active_property_definition
							{
								if let Some(direction) = active_property_definition.direction() {
									direction.option()
								} else {
									self.active_context.default_base_direction()
								}
							} else {
								self.active_context.default_base_direction()
							};

							// If `language` is not null, add `@language` to result with the value
							// `language`.
							// If `direction` is not null, add `@direction` to result with the
							// value `direction`.
							return match LangString::new(s, language, direction) {
								Ok(lang_str) => {
									Ok(Object::Value(ValueObject::LangString(lang_str)).into())
								}
								Err(s) => Ok(Object::Value(ValueObject::Literal(
									LiteralValue::new(JsonValue::String(s), None),
								))
								.into()),
							};
						}
					}

					Some(t) => {
						if let Ok(t) = t.into_iri() {
							ty = Some(t)
						} else {
							return Err(JsonLdError::InvalidTypeValue.at(location));
						}
					}
				}

				Ok(Object::Value(ValueObject::Literal(LiteralValue::new(
					result_value,
					ty.map(LiteralType::Iri),
				)))
				.into())
			}
		}
	}
}
