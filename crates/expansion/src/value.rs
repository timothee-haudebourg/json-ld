use crate::{expand_iri, Action, ExpandedEntry, Warning, WarningHandler};
use json_ld_context_processing::algorithm::RejectVocab;
use json_ld_core::{
	object::value::Literal, Context, Environment, Id, Indexed, IndexedObject, LangString, Object,
	Term, ValidId, Value,
};
use json_ld_syntax::{Direction, ErrorCode, Keyword, LenientLangTagBuf, Nullable};
use rdf_types::VocabularyMut;

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

	#[error("Forbidden use of `@vocab`")]
	ForbiddenVocab,
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
			Self::ForbiddenVocab => ErrorCode::InvalidTypeValue,
		}
	}
}

impl From<RejectVocab> for InvalidValue {
	fn from(_value: RejectVocab) -> Self {
		Self::ForbiddenVocab
	}
}

pub type ValueExpansionResult<T, B> = Result<Option<IndexedObject<T, B>>, InvalidValue>;

/// Expand a value object.
pub(crate) fn expand_value<N, L, W>(
	env: &mut Environment<N, L, W>,
	vocab_policy: Action,
	input_type: Option<Term<N::Iri, N::BlankId>>,
	type_scoped_context: &Context<N::Iri, N::BlankId>,
	expanded_entries: Vec<ExpandedEntry<N::Iri, N::BlankId>>,
	value_entry: &json_syntax::Value,
) -> ValueExpansionResult<N::Iri, N::BlankId>
where
	N: VocabularyMut,
	N::Iri: Clone + PartialEq,
	N::BlankId: Clone + PartialEq,
	W: WarningHandler<N>,
{
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
					return Err(InvalidValue::LanguageTaggedString);
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
						return Err(InvalidValue::BaseDirection);
					}
				} else {
					return Err(InvalidValue::BaseDirection);
				}
			}
			// If expanded property is @index:
			Term::Keyword(Keyword::Index) => {
				// If value is not a string, an invalid @index value error has
				// been detected and processing is aborted.
				if let Some(value) = value.as_str() {
					index = Some(value.to_string())
				} else {
					return Err(InvalidValue::IndexValue);
				}
			}
			// If expanded ...
			Term::Keyword(Keyword::Type) => {
				if let Some(ty_value) = value.as_str() {
					let expanded_ty = expand_iri(
						env,
						type_scoped_context,
						Nullable::Some(ty_value.into()),
						true,
						Some(vocab_policy),
					)?;

					match expanded_ty {
						Some(Term::Keyword(Keyword::Json)) => {
							is_json = true;
						}
						Some(Term::Id(Id::Valid(ValidId::Iri(expanded_ty)))) => {
							is_json = false;
							ty = Some(expanded_ty)
						}
						_ => return Err(InvalidValue::TypedValue),
					}
				} else {
					return Err(InvalidValue::TypedValue);
				}
			}
			Term::Keyword(Keyword::Value) => (),
			_ => {
				return Err(InvalidValue::ValueObject);
			}
		}
	}

	// If input type is @json, set expanded value to value.
	// If processing mode is json-ld-1.0, an invalid value object value error has
	// been detected and processing is aborted.
	if is_json {
		if language.is_some() || direction.is_some() {
			return Err(InvalidValue::ValueObject);
		}
		return Ok(Some(Indexed::new(
			Object::Value(Value::Json(value_entry.clone())),
			index,
		)));
	}

	// Otherwise, if value is not a scalar or null, an invalid value object value
	// error has been detected and processing is aborted.
	let result = match value_entry {
		json_syntax::Value::Null => Literal::Null,
		json_syntax::Value::String(s) => Literal::String(s.clone()),
		json_syntax::Value::Number(n) => Literal::Number(n.clone()),
		json_syntax::Value::Boolean(b) => Literal::Boolean(*b),
		_ => {
			return Err(InvalidValue::ValueObjectValue);
		}
	};

	// If the result's @type entry is @json, then the @value entry may contain any
	// value, and is treated as a JSON literal.
	// NOTE already checked?

	// Otherwise, if the value of result's @value entry is null, or an empty array,
	// return null
	if matches!(result, Literal::Null) {
		return Ok(None);
	}

	// Otherwise, if the value of result's @value entry is not a string and result
	// contains the entry @language, an invalid language-tagged value error has
	// been detected (only strings can be language-tagged) and processing is
	// aborted.
	if language.is_some() || direction.is_some() {
		if ty.is_some() {
			return Err(InvalidValue::ValueObject);
		}

		if let Literal::String(s) = result {
			let lang = match language {
				Some(language) => {
					let (language, error) = LenientLangTagBuf::new(language);

					if let Some(error) = error {
						env.warnings.handle(
							env.vocabulary,
							Warning::MalformedLanguageTag(language.to_string(), error),
						)
					}

					Some(language)
				}
				None => None,
			};

			return match LangString::new(s, lang, direction) {
				Ok(result) => Ok(Some(Indexed::new(
					Object::Value(Value::LangString(result)),
					index,
				))),
				Err(_) => Err(InvalidValue::LanguageTaggedValue),
			};
		} else {
			return Err(InvalidValue::LanguageTaggedValue);
		}
	}

	// If active property is null or @graph, drop free-floating values as follows:
	// If result is a map which is empty, or contains only the entries @value or
	// @list, set result to null.
	// TODO

	Ok(Some(Indexed::new(
		Object::Value(Value::Literal(result, ty)),
		index,
	)))
}
