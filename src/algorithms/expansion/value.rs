use json_syntax::JsonValue;
use rdf_syntax::Id;

use crate::{
	IndexedJsonLdObject, JsonLdBacktraceBuilder, JsonLdDirection, JsonLdError, JsonLdIndexed,
	JsonLdLenient, JsonLdLocatedError, JsonLdNullable, JsonLdObject, JsonLdTerm, JsonLdValue,
	JsonLdWarning, LangString,
	context::RawJsonLdProcessedContext,
	object::{LiteralValue, value::LiteralType},
	syntax::Keyword,
};

use super::{ExpandedEntry, Expander};

pub type ValueExpansionResult = Result<Option<IndexedJsonLdObject>, JsonLdLocatedError>;

impl<'a> Expander<'a> {
	/// Expand a value object.
	pub fn expand_value(
		&self,
		mut warn: impl FnMut(JsonLdWarning),
		input_type: Option<JsonLdTerm>,
		type_scoped_context: &RawJsonLdProcessedContext,
		expanded_entries: Vec<ExpandedEntry>,
		value_entry: &JsonValue,
		location: JsonLdBacktraceBuilder<'_>,
	) -> ValueExpansionResult {
		let mut is_json = input_type
			.as_ref()
			.map(|t| *t == JsonLdTerm::Keyword(Keyword::Json))
			.unwrap_or(false);
		let mut ty = None;
		let mut index = None;
		let mut language = None;
		let mut direction = None;

		for ExpandedEntry {
			key,
			expanded_key,
			value,
		} in expanded_entries
		{
			// Location of this specific entry, rather than the whole value
			// object, so errors about e.g. a malformed `@language` point at
			// the `@language` entry itself.
			let entry_loc = location.object_value(key);

			match expanded_key {
				// If expanded property is @language:
				JsonLdTerm::Keyword(Keyword::Language) => {
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
						return Err(JsonLdError::InvalidLanguageTaggedString.at(entry_loc));
					}
				}
				// If expanded property is @direction:
				JsonLdTerm::Keyword(Keyword::Direction) => {
					// If processing mode is json-ld-1.0, continue with the next key
					// from element.
					// TODO processing mode.

					// If value is neither "ltr" nor "rtl", an invalid base direction
					// error has been detected and processing is aborted.
					if let Some(value) = value.as_str() {
						if let Ok(value) = JsonLdDirection::try_from(value) {
							direction = Some(value);
						} else {
							return Err(JsonLdError::InvalidBaseDirection.at(entry_loc));
						}
					} else {
						return Err(JsonLdError::InvalidBaseDirection.at(entry_loc));
					}
				}
				// If expanded property is @index:
				JsonLdTerm::Keyword(Keyword::Index) => {
					// If value is not a string, an invalid @index value error has
					// been detected and processing is aborted.
					if let Some(value) = value.as_str() {
						index = Some(value.to_string())
					} else {
						return Err(JsonLdError::InvalidIndexValue.at(entry_loc));
					}
				}
				// If expanded ...
				JsonLdTerm::Keyword(Keyword::Type) => {
					if let Some(ty_value) = value.as_str() {
						let expanded_ty = type_scoped_context.expand_iri(
							// env,
							// type_scoped_context,
							JsonLdNullable::Some(ty_value.into()),
							true,
							true,
						);

						match expanded_ty {
							JsonLdTerm::Keyword(Keyword::Json) => {
								is_json = true;
							}
							JsonLdTerm::Id(JsonLdLenient::Valid(Id::Iri(expanded_ty))) => {
								is_json = false;
								ty = Some(expanded_ty)
							}
							_ => return Err(JsonLdError::InvalidTypedValue.at(entry_loc)),
						}
					} else {
						return Err(JsonLdError::InvalidTypedValue.at(entry_loc));
					}
				}
				JsonLdTerm::Keyword(Keyword::Value) => (),
				_ => {
					return Err(JsonLdError::InvalidValueObject.at(entry_loc));
				}
			}
		}

		// If input type is @json, set expanded value to value.
		// If processing mode is json-ld-1.0, an invalid value object value error has
		// been detected and processing is aborted.
		if is_json {
			if language.is_some() || direction.is_some() {
				return Err(JsonLdError::InvalidValueObject.at(location));
			}
			return Ok(Some(JsonLdIndexed::new(
				JsonLdObject::Value(JsonLdValue::Literal(LiteralValue::json(
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
				return Err(JsonLdError::InvalidValueObjectValue.at(location));
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
				return Err(JsonLdError::InvalidValueObject.at(location));
			}

			if let JsonValue::String(s) = result_value {
				let lang = match language {
					Some(language) => {
						let (language, error) = JsonLdLenient::from_string(language);

						if let Some(error) = error {
							warn(JsonLdWarning::MalformedLanguageTag(
								language.to_string(),
								error,
							))
						}

						Some(language)
					}
					None => None,
				};

				return match LangString::new(s, lang, direction) {
					Ok(result) => Ok(Some(JsonLdIndexed::new(
						JsonLdObject::Value(JsonLdValue::LangString(result)),
						index,
					))),
					Err(_) => Err(JsonLdError::InvalidLanguageTaggedValue.at(location)),
				};
			} else {
				return Err(JsonLdError::InvalidLanguageTaggedValue.at(location));
			}
		}

		// If active property is null or @graph, drop free-floating values as follows:
		// If result is a map which is empty, or contains only the entries @value or
		// @list, set result to null.
		// TODO

		Ok(Some(JsonLdIndexed::new(
			JsonLdObject::Value(JsonLdValue::Literal(LiteralValue::new(
				result_value,
				ty.map(LiteralType::Iri),
			))),
			index,
		)))
	}
}
