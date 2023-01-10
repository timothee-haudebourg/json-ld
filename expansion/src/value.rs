use crate::{expand_iri, ExpandedEntry, Warning, WarningHandler};
use json_ld_core::{
	object::value::{Literal, LiteralString},
	Context, Id, Indexed, IndexedObject, LangString, Object, Term, ValidId, Value,
};
use json_ld_syntax::{Direction, ErrorCode, Keyword, LenientLanguageTagBuf, Nullable};
use locspan::{At, Meta};
use rdf_types::VocabularyMut;

pub(crate) type ExpandedValue<T, B, M, W> = (Option<IndexedObject<T, B, M>>, W);

#[derive(Debug, thiserror::Error)]
pub enum InvalidValue {
	#[error("Invalid language tagged string")]
	LanguageTaggedString,

	#[error("Invalid base `@direction`")]
	BaseDirection,

	#[error("Invalid `@index` value")]
	IndexValue,

	#[error("Invalid typed value")]
	TypedValue,

	#[error("Invalid value object")]
	ValueObject,

	#[error("Invalid value object value")]
	ValueObjectValue,

	#[error("Invalid language tagged value")]
	LanguageTaggedValue,
}

impl InvalidValue {
	pub fn code(&self) -> ErrorCode {
		match self {
			Self::LanguageTaggedString => ErrorCode::InvalidLanguageTaggedString,
			Self::BaseDirection => ErrorCode::InvalidBaseDirection,
			Self::IndexValue => ErrorCode::InvalidIndexValue,
			Self::TypedValue => ErrorCode::InvalidTypedValue,
			Self::ValueObject => ErrorCode::InvalidValueObject,
			Self::ValueObjectValue => ErrorCode::InvalidValueObjectValue,
			Self::LanguageTaggedValue => ErrorCode::InvalidLanguageTaggedValue,
		}
	}
}

pub type ValueExpansionResult<T, B, M, W> =
	Result<ExpandedValue<T, B, M, W>, Meta<InvalidValue, M>>;

/// Expand a value object.
pub(crate) fn expand_value<'e, T, B, M, N, C, W>(
	vocabulary: &mut N,
	input_type: Option<Meta<Term<T, B>, M>>,
	type_scoped_context: &Context<T, B, C, M>,
	expanded_entries: Vec<ExpandedEntry<'e, T, B, M>>,
	Meta(value_entry, meta): &Meta<json_syntax::Value<M>, M>,
	mut warnings: W,
) -> ValueExpansionResult<T, B, M, W>
where
	N: VocabularyMut<Iri = T, BlankId = B>,
	T: Clone + PartialEq,
	B: Clone + PartialEq,
	M: Clone,
	W: WarningHandler<B, N, M>,
{
	let mut is_json = input_type
		.as_ref()
		.map(|t| **t == Term::Keyword(Keyword::Json))
		.unwrap_or(false);
	let mut ty = None;
	let mut index = None;
	let mut language = None;
	let mut direction = None;

	for ExpandedEntry(key, expanded_key, Meta(value, value_metadata)) in expanded_entries {
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
					return Err(InvalidValue::LanguageTaggedString.at(meta.clone()));
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
						return Err(InvalidValue::BaseDirection.at(meta.clone()));
					}
				} else {
					return Err(InvalidValue::BaseDirection.at(meta.clone()));
				}
			}
			// If expanded property is @index:
			Term::Keyword(Keyword::Index) => {
				// If value is not a string, an invalid @index value error has
				// been detected and processing is aborted.
				if let Some(value) = value.as_str() {
					index = Some(json_ld_syntax::Entry::new(
						key.into_metadata().clone(),
						Meta(value.to_string(), value_metadata.clone()),
					))
				} else {
					return Err(InvalidValue::IndexValue.at(meta.clone()));
				}
			}
			// If expanded ...
			Term::Keyword(Keyword::Type) => {
				if let Some(ty_value) = value.as_str() {
					let Meta(expanded_ty, _) = expand_iri(
						vocabulary,
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
						Term::Id(Id::Valid(ValidId::Iri(expanded_ty))) => {
							is_json = false;
							ty = Some(expanded_ty)
						}
						_ => return Err(InvalidValue::TypedValue.at(meta.clone())),
					}
				} else {
					return Err(InvalidValue::TypedValue.at(meta.clone()));
				}
			}
			Term::Keyword(Keyword::Value) => (),
			_ => {
				return Err(InvalidValue::ValueObject.at(meta.clone()));
			}
		}
	}

	// If input type is @json, set expanded value to value.
	// If processing mode is json-ld-1.0, an invalid value object value error has
	// been detected and processing is aborted.
	if is_json {
		if language.is_some() || direction.is_some() {
			return Err(InvalidValue::ValueObject.at(meta.clone()));
		}
		return Ok((
			Some(Meta(
				Indexed::new(
					Object::Value(Value::Json(Meta(value_entry.clone(), meta.clone()))),
					index,
				),
				meta.clone(),
			)),
			warnings,
		));
	}

	// Otherwise, if value is not a scalar or null, an invalid value object value
	// error has been detected and processing is aborted.
	let result = match value_entry {
		json_syntax::Value::Null => Literal::Null,
		json_syntax::Value::String(s) => Literal::String(LiteralString::Expanded(s.clone())),
		json_syntax::Value::Number(n) => Literal::Number(n.clone()),
		json_syntax::Value::Boolean(b) => Literal::Boolean(*b),
		_ => {
			return Err(InvalidValue::ValueObjectValue.at(meta.clone()));
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
			return Err(InvalidValue::ValueObject.at(meta.clone()));
		}

		if let Literal::String(s) = result {
			let lang = match language {
				Some(Meta(language, language_metadata)) => {
					let (language, error) = LenientLanguageTagBuf::new(language);

					if let Some(error) = error {
						warnings.handle(
							vocabulary,
							Meta::new(
								Warning::MalformedLanguageTag(language.to_string(), error),
								language_metadata,
							),
						)
					}

					Some(language)
				}
				None => None,
			};

			return match LangString::new(s, lang, direction) {
				Ok(result) => Ok((
					Some(Meta(
						Indexed::new(Object::Value(Value::LangString(result)), index),
						meta.clone(),
					)),
					warnings,
				)),
				Err(_) => Err(InvalidValue::LanguageTaggedValue.at(meta.clone())),
			};
		} else {
			return Err(InvalidValue::LanguageTaggedValue.at(meta.clone()));
		}
	}

	// If active property is null or @graph, drop free-floating values as follows:
	// If result is a map which is empty, or contains only the entries @value or
	// @list, set result to null.
	// TODO

	Ok((
		Some(Meta(
			Indexed::new(Object::Value(Value::Literal(result, ty)), index),
			meta.clone(),
		)),
		warnings,
	))
}
