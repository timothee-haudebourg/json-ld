use json_syntax::JsonValue;
use rdf_syntax::Id;

use crate::{
	algorithms::{Error, JsonLdLocated, JsonLdLocationStack, Warning},
	context::RawProcessedContext,
	object::{value::LiteralType, LiteralValue},
	syntax::Keyword,
	Direction, Indexed, IndexedObject, LangString, Lenient, Nullable, Object, Term, ValueObject,
};

use super::{ExpandedEntry, Expander};

pub type ValueExpansionResult = Result<Option<IndexedObject>, JsonLdLocated<Error>>;

impl<'a> Expander<'a> {
	/// Expand a value object.
	pub fn expand_value(
		&self,
		mut warn: impl FnMut(Warning),
		input_type: Option<Term>,
		type_scoped_context: &RawProcessedContext,
		expanded_entries: Vec<ExpandedEntry>,
		value_entry: &JsonValue,
		location: JsonLdLocationStack<'_>,
	) -> ValueExpansionResult {
		let mut is_json = input_type
			.as_ref()
			.map(|t| *t == Term::Keyword(Keyword::Json))
			.unwrap_or(false);
		let mut ty = None;
		let mut index = None;
		let mut language = None;
		let mut direction = None;

		for ExpandedEntry(_, expanded_key, value) in expanded_entries {
			match expanded_key {
				// If expanded property is @language:
				Term::Keyword(Keyword::Language) => {
					// If value is not a string, an invalid language-tagged string
					// error has been detected and processing is aborted.
					if let Some(value) = value.as_str() {
						// Otherwise, set expanded value to value. If value is not
						// well-formed according to section 2.2.9 of [BCP47],
						// processors SHOULD issue a warning.
						// TODO warning.

						if value != "@none" {
							language = Some(value.to_owned());
						}
					} else {
						return Err(Error::InvalidLanguageTaggedString.at(location));
					}
				}
				// If expanded property is @direction:
				Term::Keyword(Keyword::Direction) => {
					// If processing mode is json-ld-1.0, continue with the next key
					// from element.
					// TODO processing mode.

					// If value is neither "ltr" nor "rtl", an invalid base direction
					// error has been detected and processing is aborted.
					if let Some(value) = value.as_str() {
						if let Ok(value) = Direction::try_from(value) {
							direction = Some(value);
						} else {
							return Err(Error::InvalidBaseDirection.at(location));
						}
					} else {
						return Err(Error::InvalidBaseDirection.at(location));
					}
				}
				// If expanded property is @index:
				Term::Keyword(Keyword::Index) => {
					// If value is not a string, an invalid @index value error has
					// been detected and processing is aborted.
					if let Some(value) = value.as_str() {
						index = Some(value.to_string())
					} else {
						return Err(Error::InvalidIndexValue.at(location));
					}
				}
				// If expanded ...
				Term::Keyword(Keyword::Type) => {
					if let Some(ty_value) = value.as_str() {
						let expanded_ty = type_scoped_context.expand_iri(
							// env,
							// type_scoped_context,
							Nullable::Some(ty_value.into()),
							true,
							true,
						);

						match expanded_ty {
							Term::Keyword(Keyword::Json) => {
								is_json = true;
							}
							Term::Id(Lenient::Valid(Id::Iri(expanded_ty))) => {
								is_json = false;
								ty = Some(expanded_ty)
							}
							_ => return Err(Error::InvalidTypedValue.at(location)),
						}
					} else {
						return Err(Error::InvalidTypedValue.at(location));
					}
				}
				Term::Keyword(Keyword::Value) => (),
				_ => {
					return Err(Error::InvalidValueObject.at(location));
				}
			}
		}

		// If input type is @json, set expanded value to value.
		// If processing mode is json-ld-1.0, an invalid value object value error has
		// been detected and processing is aborted.
		if is_json {
			if language.is_some() || direction.is_some() {
				return Err(Error::InvalidValueObject.at(location));
			}
			return Ok(Some(Indexed::new(
				Object::Value(ValueObject::Literal(LiteralValue::json(
					value_entry.clone(),
				))),
				index,
			)));
		}

		// Otherwise, if value is not a scalar or null, an invalid value object value
		// error has been detected and processing is aborted.
		let result_value = match value_entry {
			JsonValue::Null
			| JsonValue::String(_)
			| JsonValue::Number(_)
			| JsonValue::Boolean(_) => value_entry.clone(),
			_ => {
				return Err(Error::InvalidValueObjectValue.at(location));
			}
		};

		// If the result's @type entry is @json, then the @value entry may contain any
		// value, and is treated as a JSON literal.
		// NOTE already checked?

		// Otherwise, if the value of result's @value entry is null, or an empty array,
		// return null
		if result_value.is_null() {
			return Ok(None);
		}

		// Otherwise, if the value of result's @value entry is not a string and result
		// contains the entry @language, an invalid language-tagged value error has
		// been detected (only strings can be language-tagged) and processing is
		// aborted.
		if language.is_some() || direction.is_some() {
			if ty.is_some() {
				return Err(Error::InvalidValueObject.at(location));
			}

			if let JsonValue::String(s) = result_value {
				let lang = match language {
					Some(language) => {
						let (language, error) = Lenient::from_string(language);

						if let Some(error) = error {
							warn(Warning::MalformedLanguageTag(language.to_string(), error))
						}

						Some(language)
					}
					None => None,
				};

				return match LangString::new(s, lang, direction) {
					Ok(result) => Ok(Some(Indexed::new(
						Object::Value(ValueObject::LangString(result)),
						index,
					))),
					Err(_) => Err(Error::InvalidLanguageTaggedValue.at(location)),
				};
			} else {
				return Err(Error::InvalidLanguageTaggedValue.at(location));
			}
		}

		// If active property is null or @graph, drop free-floating values as follows:
		// If result is a map which is empty, or contains only the entries @value or
		// @list, set result to null.
		// TODO

		Ok(Some(Indexed::new(
			Object::Value(ValueObject::Literal(LiteralValue::new(
				result_value,
				ty.map(LiteralType::Iri),
			))),
			index,
		)))
	}
}
