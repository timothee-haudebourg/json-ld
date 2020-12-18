use std::collections::HashSet;
use langtag::LanguageTagBuf;
use json::JsonValue;
use crate::{
	Error,
	ErrorCode,
	Direction,
	LangString,
	Id,
	Indexed,
	object::*,
	Context,
	syntax::Type
};
use super::{
	expand_iri,
	node_id_of_term
};

fn clone_default_language<T: Id, C: Context<T>>(active_context: &C) -> Option<LanguageTagBuf> {
	match active_context.default_language() {
		Some(lang) => Some(lang.cloned()),
		None => None
	}
}

fn clone_default_base_direction<T: Id, C: Context<T>>(active_context: &C) -> Option<Direction> {
	match active_context.default_base_direction() {
		Some(dir) => Some(dir),
		None => None
	}
}

/// https://www.w3.org/TR/json-ld11-api/#value-expansion
pub fn expand_literal<T: Id, C: Context<T>>(active_context: &C, active_property: Option<&str>, value: &JsonValue) -> Result<Indexed<Object<T>>, Error> {
	let active_property_definition = active_context.get_opt(active_property);

	let active_property_type = if let Some(active_property_definition) = active_property_definition {
		active_property_definition.typ.clone()
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
			node.id = node_id_of_term(expand_iri(active_context, value.as_str().unwrap(), true, false));
			Ok(Object::Node(node).into())
		},

		// If `active_property` has a type mapping in active context that is `@vocab`, and the
		// value is a string, return a new map containing a single entry where the key is
		// `@id` and the value is the result of IRI expanding `value` using `true` for
		// document relative.
		Some(Type::Vocab) if value.is_string() => {
			let mut node = Node::new();
			node.id = node_id_of_term(expand_iri(active_context, value.as_str().unwrap(), true, true));
			Ok(Object::Node(node).into())
		},

		_ => {
			// Otherwise, initialize `result` to a map with an `@value` entry whose value is set to
			// `value`.
			let result = match value {
				JsonValue::Null => Literal::Null,
				JsonValue::Boolean(b) => Literal::Boolean(*b),
				JsonValue::Number(n) => Literal::Number(*n),
				JsonValue::Short(_) | JsonValue::String(_) => Literal::String(value.as_str().unwrap().to_string()),
				_ => panic!("expand_literal must be called with a literal JSON value")
			};

			// If `active_property` has a type mapping in active context, other than `@id`,
			// `@vocab`, or `@none`, add `@type` to `result` and set its value to the value
			// associated with the type mapping.
			let mut ty = None;
			match active_property_type {
				None | Some(Type::Id) | Some(Type::Vocab) | Some(Type::None) => {
					// Otherwise, if value is a string:
					if let Literal::String(str) = result {
						// Initialize `language` to the language mapping for
						// `active_property` in `active_context`, if any, otherwise to the
						// default language of `active_context`.
						let language = if let Some(active_property_definition) = active_property_definition {
							if let Some(language) = &active_property_definition.language {
								language.as_ref().cloned().option()
							} else {
								clone_default_language(active_context)
							}
						} else {
							clone_default_language(active_context)
						};

						// Initialize `direction` to the direction mapping for
						// `active_property` in `active_context`, if any, otherwise to the
						// default base direction of `active_context`.
						let direction = if let Some(active_property_definition) = active_property_definition {
							if let Some(direction) = &active_property_definition.direction {
								direction.clone().option()
							} else {
								clone_default_base_direction(active_context)
							}
						} else {
							clone_default_base_direction(active_context)
						};

						// If `language` is not null, add `@language` to result with the value
						// `language`.
						// If `direction` is not null, add `@direction` to result with the
						// value `direction`.
						return match LangString::new(str, language, direction) {
							Ok(lang_str) => Ok(Object::Value(Value::LangString(lang_str)).into()),
							Err(str) => Ok(Object::Value(Value::Literal(Literal::String(str), None)).into())
						}
					}
				},

				Some(t) => {
					if let Ok(t) = t.into_ref() {
						ty = Some(t)
					} else {
						return Err(ErrorCode::InvalidTypeValue.into())
					}
				}
			}

			Ok(Object::Value(Value::Literal(result, ty)).into())
		}
	}
}
