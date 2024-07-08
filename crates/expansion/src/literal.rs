use crate::{expand_iri, node_id_of_term, ActiveProperty, WarningHandler};
use json_ld_context_processing::algorithm::{Action, RejectVocab};
use json_ld_core::{
	object::value::Literal, Context, Environment, IndexedObject, LangString, Node, Object, Type,
	Value,
};
use json_ld_syntax::{ErrorCode, LenientLangTag, Nullable};
use json_syntax::Number;
use rdf_types::VocabularyMut;

pub(crate) enum GivenLiteralValue<'a> {
	Boolean(bool),
	Number(&'a Number),
	String(&'a str),
}

impl<'a> GivenLiteralValue<'a> {
	pub fn new(value: &'a json_syntax::Value) -> Self {
		match value {
			json_syntax::Value::Boolean(b) => Self::Boolean(*b),
			json_syntax::Value::Number(n) => Self::Number(n),
			json_syntax::Value::String(s) => Self::String(s),
			_ => panic!("not a literal value"),
		}
	}

	pub fn is_string(&self) -> bool {
		matches!(self, Self::String(_))
	}

	pub fn as_str(&self) -> Option<&'a str> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}
}

pub(crate) enum LiteralValue<'a> {
	Given(GivenLiteralValue<'a>),
	Inferred(json_ld_syntax::String),
}

impl<'a> LiteralValue<'a> {
	pub fn is_string(&self) -> bool {
		match self {
			Self::Given(v) => v.is_string(),
			Self::Inferred(_) => true,
		}
	}

	pub fn as_str(&self) -> Option<&str> {
		match self {
			Self::Given(v) => v.as_str(),
			Self::Inferred(s) => Some(s.as_str()),
		}
	}
}

pub(crate) type ExpandedLiteral<T, B> = IndexedObject<T, B>;

#[derive(Debug, thiserror::Error)]
pub enum LiteralExpansionError {
	#[error("Invalid `@type` value")]
	InvalidTypeValue,

	#[error("Forbidden use of `@vocab`")]
	ForbiddenVocab,
}

impl LiteralExpansionError {
	pub fn code(&self) -> ErrorCode {
		match self {
			Self::InvalidTypeValue => ErrorCode::InvalidTypeValue,
			Self::ForbiddenVocab => ErrorCode::InvalidTypeValue,
		}
	}
}

impl From<RejectVocab> for LiteralExpansionError {
	fn from(_value: RejectVocab) -> Self {
		Self::ForbiddenVocab
	}
}

pub(crate) type LiteralExpansionResult<T, B> = Result<ExpandedLiteral<T, B>, LiteralExpansionError>;

/// Expand a literal value.
/// See <https://www.w3.org/TR/json-ld11-api/#value-expansion>.
pub(crate) fn expand_literal<N, L, W>(
	mut env: Environment<N, L, W>,
	vocab_policy: Action,
	active_context: &Context<N::Iri, N::BlankId>,
	active_property: ActiveProperty<'_>,
	value: LiteralValue,
) -> LiteralExpansionResult<N::Iri, N::BlankId>
where
	N: VocabularyMut,
	N::Iri: Clone,
	N::BlankId: Clone,
	W: WarningHandler<N>,
{
	let active_property_definition = active_property.get_from(active_context);
	let active_property_type = if let Some(active_property_definition) = active_property_definition
	{
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
			let mut node = Node::new();
			let id = node_id_of_term(
				expand_iri(
					&mut env,
					active_context,
					Nullable::Some(value.as_str().unwrap().into()),
					true,
					None,
				)
				.unwrap()
				.unwrap(),
			);

			node.id = id;
			Ok(Object::node(node).into())
		}

		// If `active_property` has a type mapping in active context that is `@vocab`, and the
		// value is a string, return a new map containing a single entry where the key is
		// `@id` and the value is the result of IRI expanding `value` using `true` for
		// document relative.
		Some(Type::Vocab) if value.is_string() => {
			let mut node = Node::new();

			let ty = expand_iri(
				&mut env,
				active_context,
				Nullable::Some(value.as_str().unwrap().into()),
				true,
				Some(vocab_policy),
			)?;

			if let Some(ty) = ty {
				let id = node_id_of_term(ty);
				node.id = id;
			}

			Ok(Object::node(node).into())
		}

		_ => {
			// Otherwise, initialize `result` to a map with an `@value` entry whose value is set to
			// `value`.
			let result: Literal = match value {
				LiteralValue::Given(v) => match v {
					GivenLiteralValue::Boolean(b) => Literal::Boolean(b),
					GivenLiteralValue::Number(n) => Literal::Number(unsafe {
						json_syntax::NumberBuf::new_unchecked(n.as_bytes().into())
					}),
					GivenLiteralValue::String(s) => Literal::String(s.into()),
				},
				LiteralValue::Inferred(s) => Literal::String(s),
			};

			// If `active_property` has a type mapping in active context, other than `@id`,
			// `@vocab`, or `@none`, add `@type` to `result` and set its value to the value
			// associated with the type mapping.
			let mut ty = None;
			match active_property_type {
				None | Some(Type::Id) | Some(Type::Vocab) | Some(Type::None) => {
					// Otherwise, if value is a string:
					if let Literal::String(s) = result {
						// Initialize `language` to the language mapping for
						// `active_property` in `active_context`, if any, otherwise to the
						// default language of `active_context`.
						let language =
							if let Some(active_property_definition) = active_property_definition {
								if let Some(language) = active_property_definition.language() {
									language.cloned().option()
								} else {
									active_context
										.default_language()
										.map(LenientLangTag::to_owned)
								}
							} else {
								active_context
									.default_language()
									.map(LenientLangTag::to_owned)
							};

						// Initialize `direction` to the direction mapping for
						// `active_property` in `active_context`, if any, otherwise to the
						// default base direction of `active_context`.
						let direction =
							if let Some(active_property_definition) = active_property_definition {
								if let Some(direction) = active_property_definition.direction() {
									direction.option()
								} else {
									active_context.default_base_direction()
								}
							} else {
								active_context.default_base_direction()
							};

						// If `language` is not null, add `@language` to result with the value
						// `language`.
						// If `direction` is not null, add `@direction` to result with the
						// value `direction`.
						return match LangString::new(s, language, direction) {
							Ok(lang_str) => Ok(Object::Value(Value::LangString(lang_str)).into()),
							Err(s) => {
								Ok(Object::Value(Value::Literal(Literal::String(s), None)).into())
							}
						};
					}
				}

				Some(t) => {
					if let Ok(t) = t.into_iri() {
						ty = Some(t)
					} else {
						return Err(LiteralExpansionError::InvalidTypeValue);
					}
				}
			}

			Ok(Object::Value(Value::Literal(result, ty)).into())
		}
	}
}
