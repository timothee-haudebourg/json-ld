use super::{expand_iri, Entry};
use crate::{
    object::*,
    syntax::{Keyword, Term},
    ContextMut, Direction, Error, ErrorCode, Id, Indexed, LangString, Reference,
};
use json::JsonValue;
use langtag::LanguageTagBuf;
use std::convert::TryFrom;

pub fn expand_value<'a, T: Id, C: ContextMut<T>>(
    input_type: Option<Term<T>>,
    type_scoped_context: &C,
    expanded_entries: Vec<Entry<(&str, Term<T>)>>,
    value_entry: &JsonValue,
) -> Result<Option<Indexed<Object<T>>>, Error> {
    let mut is_json = input_type == Some(Term::Keyword(Keyword::Json));
    let mut ty = None;
    let mut index = None;
    let mut language = None;
    let mut direction = None;

    for Entry((_, expanded_key), value) in expanded_entries {
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
                        language = Some(value.to_string());
                    }
                } else {
                    return Err(ErrorCode::InvalidLanguageTaggedString.into());
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
                        return Err(ErrorCode::InvalidBaseDirection.into());
                    }
                } else {
                    return Err(ErrorCode::InvalidBaseDirection.into());
                }
            }
            // If expanded property is @index:
            Term::Keyword(Keyword::Index) => {
                // If value is not a string, an invalid @index value error has
                // been detected and processing is aborted.
                if let Some(value) = value.as_str() {
                    index = Some(value.to_string())
                } else {
                    return Err(ErrorCode::InvalidIndexValue.into());
                }
            }
            // If expanded ...
            Term::Keyword(Keyword::Type) => {
                if let Some(ty_value) = value.as_str() {
                    let expanded_ty = expand_iri(type_scoped_context, ty_value, true, true);

                    match expanded_ty {
                        Term::Keyword(Keyword::Json) => {
                            is_json = true;
                        }
                        Term::Ref(Reference::Id(expanded_ty)) => {
                            is_json = false;
                            ty = Some(expanded_ty)
                        }
                        _ => return Err(ErrorCode::InvalidTypedValue.into()),
                    }
                } else {
                    return Err(ErrorCode::InvalidTypedValue.into());
                }
            }
            Term::Keyword(Keyword::Value) => (),
            _ => {
                return Err(ErrorCode::InvalidValueObject.into());
            }
        }
    }

    // If input type is @json, set expanded value to value.
    // If processing mode is json-ld-1.0, an invalid value object value error has
    // been detected and processing is aborted.
    if is_json {
        if language.is_some() || direction.is_some() {
            return Err(ErrorCode::InvalidValueObject.into());
        }
        return Ok(Some(Indexed::new(
            Object::Value(Value::Json(value_entry.clone())),
            index,
        )));
    }

    // Otherwise, if value is not a scalar or null, an invalid value object value
    // error has been detected and processing is aborted.
    let result = match value_entry {
        JsonValue::Null => Literal::Null,
        JsonValue::Short(_) | JsonValue::String(_) => {
            Literal::String(value_entry.as_str().unwrap().to_string())
        }
        JsonValue::Number(n) => Literal::Number(*n),
        JsonValue::Boolean(b) => Literal::Boolean(*b),
        _ => {
            return Err(ErrorCode::InvalidValueObjectValue.into());
        }
    };

    // If the result's @type entry is @json, then the @value entry may contain any
    // value, and is treated as a JSON literal.
    // NOTE already checked?

    // Otherwise, if the value of result's @value entry is null, or an empty array,
    // return null
    let is_empty = match result {
        Literal::Null => true,
        // Value::Array(ary) => ary.is_empty(),
        _ => false,
    };

    if is_empty {
        return Ok(None);
    }

    // Otherwise, if the value of result's @value entry is not a string and result
    // contains the entry @language, an invalid language-tagged value error has
    // been detected (only strings can be language-tagged) and processing is
    // aborted.
    if language.is_some() || direction.is_some() {
        if ty.is_some() {
            return Err(ErrorCode::InvalidValueObject.into());
        }

        if let Literal::String(str) = result {
            let lang = match language {
                Some(language) => match LanguageTagBuf::new(language.into_bytes()) {
                    Ok(lang) => Some(lang),
                    Err(_) => return Ok(None),
                },
                None => None,
            };

            let result = LangString::new(str, lang, direction).unwrap();
            return Ok(Some(Indexed::new(
                Object::Value(Value::LangString(result)),
                index,
            )));
        } else {
            return Err(ErrorCode::InvalidLanguageTaggedValue.into());
        }
    }

    // If active property is null or @graph, drop free-floating values as follows:
    // If result is a map which is empty, or contains only the entries @value or
    // @list, set result to null.
    // TODO

    return Ok(Some(Indexed::new(
        Object::Value(Value::Literal(result, ty)),
        index,
    )));
}
