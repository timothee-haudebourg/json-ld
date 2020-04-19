use std::collections::HashSet;
use std::convert::TryFrom;
use json::JsonValue;
use crate::{Error, ErrorCode, Keyword, Direction, Id, Key, Value, Literal, Object, ObjectData};
use crate::util::as_array;
use crate::context::MutableActiveContext;
use super::{Entry, expand_iri};

pub fn expand_value<'a, T: Id, C: MutableActiveContext<T>>(input_type: Option<Key<T>>, type_scoped_context: &C, expanded_entries: Vec<Entry<(&str, Key<T>)>>, value_entry: &JsonValue) -> Result<Option<Object<T>>, Error> {
	// If input type is @json, set expanded value to value.
	// If processing mode is json-ld-1.0, an invalid value object value error has
	// been detected and processing is aborted.

	// Otherwise, if value is not a scalar or null, an invalid value object value
	// error has been detected and processing is aborted.
	let mut result = if input_type == Some(Key::Keyword(Keyword::JSON)) {
		Literal::Json(value_entry.clone())
	} else {
		match value_entry {
			JsonValue::Null => {
				Literal::Null
			},
			JsonValue::Short(_) | JsonValue::String(_) => {
				Literal::String {
					data: value_entry.as_str().unwrap().to_string(),
					language: None,
					direction: None
				}
			},
			JsonValue::Number(n) => {
				Literal::Number(*n)
			},
			JsonValue::Boolean(b) => {
				Literal::Boolean(*b)
			},
			_ => {
				return Err(ErrorCode::InvalidValueObject.into());
			}
		}
	};

	let mut result_data = ObjectData::new();

	let mut types = HashSet::new();
	let mut language = None;
	let mut direction = None;

	for Entry((_, expanded_key), value) in expanded_entries {
		match expanded_key {
			// If expanded property is @language:
			Key::Keyword(Keyword::Language) => {
				// If value is not a string, an invalid language-tagged string
				// error has been detected and processing is aborted.
				if let Some(value) = value.as_str() {
					// Otherwise, set expanded value to value. If value is not
					// well-formed according to section 2.2.9 of [BCP47],
					// processors SHOULD issue a warning.
					// TODO warning.

					if value != "@none" {
						language = Some(value);
					}
				} else {
					return Err(ErrorCode::InvalidLanguageTaggedString.into())
				}
			},
			// If expanded property is @direction:
			Key::Keyword(Keyword::Direction) => {
				// If processing mode is json-ld-1.0, continue with the next key
				// from element.
				// TODO processing mode.

				// If value is neither "ltr" nor "rtl", an invalid base direction
				// error has been detected and processing is aborted.
				if let Some(value) = value.as_str() {
					if let Ok(value) = Direction::try_from(value) {
						direction = Some(value);
					} else {
						return Err(ErrorCode::InvalidBaseDirection.into())
					}
				} else {
					return Err(ErrorCode::InvalidBaseDirection.into())
				}
			},
			// If expanded property is @index:
			Key::Keyword(Keyword::Index) => {
				// If value is not a string, an invalid @index value error has
				// been detected and processing is aborted.
				if let Some(value) = value.as_str() {
					result_data.index = Some(value.to_string())
				} else {
					return Err(ErrorCode::InvalidIndexValue.into())
				}
			},
			// If expanded ...
			Key::Keyword(Keyword::Type) => {
				// If value is neither a string nor an array of strings, an
				// invalid type value error has been detected and processing
				// is aborted.
				let value = as_array(value);
				// Set `expanded_value` to the result of IRI expanding each
				// of its values using `type_scoped_context` for active
				// context, and true for document relative.
				for ty in value {
					if let Some(ty) = ty.as_str() {
						let expanded_ty = expand_iri(type_scoped_context, ty, true, true);

						if expanded_ty == Key::Keyword(Keyword::JSON) {
							result = Literal::Json(value_entry.clone())
						}

						types.insert(expanded_ty);
					} else {
						return Err(ErrorCode::InvalidTypeValue.into())
					}
				}
			},
			Key::Keyword(Keyword::Value) => (),
			_ => {
				return Err(ErrorCode::InvalidValueObject.into());
			}
		}
	}

	// If the result's @type entry is @json, then the @value entry may contain any
	// value, and is treated as a JSON literal.
	// NOTE already checked?

	// Otherwise, if the value of result's @value entry is null, or an empty array,
	// return null
	let is_empty = match result {
		Literal::Null => true,
		// Value::Array(ary) => ary.is_empty(),
		_ => false
	};

	if is_empty {
		return Ok(None)
	}

	// Otherwise, if the value of result's @value entry is not a string and result
	// contains the entry @language, an invalid language-tagged value error has
	// been detected (only strings can be language-tagged) and processing is
	// aborted.
	if let Some(lang) = language {
		if let Literal::String { ref mut language, .. } = &mut result {
			*language = Some(lang.to_string())
		} else {
			return Err(ErrorCode::InvalidLanguageTaggedValue.into())
		}
	}

	if let Some(dir) = direction {
		if let Literal::String { ref mut direction, .. } = &mut result {
			*direction = Some(dir)
		} else {
			return Err(ErrorCode::InvalidLanguageTaggedValue.into())
		}
	}

	// If active property is null or @graph, drop free-floating values as follows:
	// If result is a map which is empty, or contains only the entries @value or
	// @list, set result to null.
	// TODO

	return Ok(Some(Object::Value(Value::Literal(result, types), result_data)));
}
