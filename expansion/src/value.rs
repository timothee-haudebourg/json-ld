use crate::{expand_iri, ExpandedEntry, Warning};
use json_ld_core::{
	object::value::{Literal, LiteralString},
	Context, Id, Indexed, LangString, Object, Reference, Term, Value,
};
use json_ld_syntax::{AnyContextEntry, Direction, Keyword, LenientLanguageTagBuf, Nullable};
use locspan::{At, Meta};

pub(crate) type ExpandedValue<T, M, W> = (Option<Indexed<Object<T, M>>>, W);

#[derive(Debug)]
pub enum ValueExpansionError {
	InvalidLanguageTaggedString,
	InvalidBaseDirection,
	InvalidIndexValue,
	InvalidTypedValue,
	InvalidValueObject,
	InvalidValueObjectValue,
	InvalidLanguageTaggedValue,
}

/// Expand a value object.
pub(crate) fn expand_value<'e, T: Id, C: AnyContextEntry, W: FnMut(Meta<Warning, C::Metadata>)>(
	input_type: Option<Term<T>>,
	type_scoped_context: &Context<T, C>,
	expanded_entries: Vec<ExpandedEntry<'e, T, C, C::Metadata>>,
	Meta(value_entry, meta): &Meta<json_ld_syntax::Value<C, C::Metadata>, C::Metadata>,
	mut warnings: W,
) -> Result<ExpandedValue<T, C::Metadata, W>, Meta<ValueExpansionError, C::Metadata>> {
	let mut is_json = input_type == Some(Term::Keyword(Keyword::Json));
	let mut ty = None;
	let mut index = None;
	let mut language = None;
	let mut direction = None;

	for ExpandedEntry(_, expanded_key, Meta(value, value_metadata)) in expanded_entries {
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
						language = Some(Meta(value.to_owned(), value_metadata.clone()));
					}
				} else {
					return Err(ValueExpansionError::InvalidLanguageTaggedString.at(meta.clone()));
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
						return Err(ValueExpansionError::InvalidBaseDirection.at(meta.clone()));
					}
				} else {
					return Err(ValueExpansionError::InvalidBaseDirection.at(meta.clone()));
				}
			}
			// If expanded property is @index:
			Term::Keyword(Keyword::Index) => {
				// If value is not a string, an invalid @index value error has
				// been detected and processing is aborted.
				if let Some(value) = value.as_str() {
					index = Some(value.to_string())
				} else {
					return Err(ValueExpansionError::InvalidIndexValue.at(meta.clone()));
				}
			}
			// If expanded ...
			Term::Keyword(Keyword::Type) => {
				if let Some(ty_value) = value.as_str() {
					let expanded_ty = expand_iri(
						type_scoped_context,
						Meta(Nullable::Some(ty_value.into()), value_metadata.clone()),
						true,
						true,
						&mut warnings,
					);

					match expanded_ty {
						Term::Keyword(Keyword::Json) => {
							is_json = true;
						}
						Term::Ref(Reference::Id(expanded_ty)) => {
							is_json = false;
							ty = Some(expanded_ty)
						}
						_ => return Err(ValueExpansionError::InvalidTypedValue.at(meta.clone())),
					}
				} else {
					return Err(ValueExpansionError::InvalidTypedValue.at(meta.clone()));
				}
			}
			Term::Keyword(Keyword::Value) => (),
			_ => {
				return Err(ValueExpansionError::InvalidValueObject.at(meta.clone()));
			}
		}
	}

	// If input type is @json, set expanded value to value.
	// If processing mode is json-ld-1.0, an invalid value object value error has
	// been detected and processing is aborted.
	if is_json {
		if language.is_some() || direction.is_some() {
			return Err(ValueExpansionError::InvalidValueObject.at(meta.clone()));
		}
		return Ok((
			Some(Indexed::new(
				Object::Value(Value::Json(value_entry.clone().into_json())),
				index,
			)),
			warnings,
		));
	}

	// Otherwise, if value is not a scalar or null, an invalid value object value
	// error has been detected and processing is aborted.
	let result = match value_entry {
		json_ld_syntax::Value::Null => Literal::Null,
		json_ld_syntax::Value::String(s) => Literal::String(LiteralString::Expanded(s.clone())),
		json_ld_syntax::Value::Number(n) => Literal::Number(n.clone()),
		json_ld_syntax::Value::Boolean(b) => Literal::Boolean(*b),
		_ => {
			return Err(ValueExpansionError::InvalidValueObjectValue.at(meta.clone()));
		}
	};

	// If the result's @type entry is @json, then the @value entry may contain any
	// value, and is treated as a JSON literal.
	// NOTE already checked?

	// Otherwise, if the value of result's @value entry is null, or an empty array,
	// return null
	if matches!(result, Literal::Null) {
		return Ok((None, warnings));
	}

	// Otherwise, if the value of result's @value entry is not a string and result
	// contains the entry @language, an invalid language-tagged value error has
	// been detected (only strings can be language-tagged) and processing is
	// aborted.
	if language.is_some() || direction.is_some() {
		if ty.is_some() {
			return Err(ValueExpansionError::InvalidValueObject.at(meta.clone()));
		}

		if let Literal::String(s) = result {
			let lang = match language {
				Some(Meta(language, language_metadata)) => {
					let (language, error) = LenientLanguageTagBuf::new(language);

					if let Some(error) = error {
						warnings(Meta::new(
							Warning::MalformedLanguageTag(language.to_string(), error),
							language_metadata,
						))
					}

					Some(language)
				}
				None => None,
			};

			return match LangString::new(s, lang, direction) {
				Ok(result) => Ok((
					Some(Indexed::new(
						Object::Value(Value::LangString(result)),
						index,
					)),
					warnings,
				)),
				Err(_) => Err(ValueExpansionError::InvalidLanguageTaggedValue.at(meta.clone())),
			};
		} else {
			return Err(ValueExpansionError::InvalidLanguageTaggedValue.at(meta.clone()));
		}
	}

	// If active property is null or @graph, drop free-floating values as follows:
	// If result is a map which is empty, or contains only the entries @value or
	// @list, set result to null.
	// TODO

	Ok((
		Some(Indexed::new(
			Object::Value(Value::Literal(result, ty)),
			index,
		)),
		warnings,
	))
}
