use std::collections::HashSet;
use json::JsonValue;
use crate::{Error, Keyword, Direction, Id, Key, Object, Node, Value, Literal, ActiveContext};
use super::expand_iri;

fn clone_default_language<T: Id, C: ActiveContext<T>>(active_context: &C) -> Option<String> {
	match active_context.default_language() {
		Some(lang) => Some(lang.to_string()),
		None => None
	}
}

fn clone_default_base_direction<T: Id, C: ActiveContext<T>>(active_context: &C) -> Option<Direction> {
	match active_context.default_base_direction() {
		Some(dir) => Some(dir),
		None => None
	}
}

/// https://www.w3.org/TR/json-ld11-api/#value-expansion
pub fn expand_literal<T: Id, C: ActiveContext<T>>(active_context: &C, active_property: Option<&str>, value: &JsonValue) -> Result<Object<T>, Error> {
	let active_property_definition = active_context.get_opt(active_property);

	let active_property_type = if let Some(active_property_definition) = active_property_definition {
		active_property_definition.typ.clone()
	} else {
		None
	};

	match active_property_type {
		// If the `active_property` has a type mapping in `active_context` that is `@id`, and the
		// `value` is a string, return a new map containing a single entry where the key is `@id` and
		// the value is the result IRI expanding `value` using `true` for `document_relative` and
		// `false` for vocab.
		Some(Key::Keyword(Keyword::Id)) if value.is_string() => {
			let mut node = Node::new();
			node.id = Some(expand_iri(active_context, value.as_str().unwrap(), true, false));
			Ok(node.into())
		},

		// If `active_property` has a type mapping in active context that is `@vocab`, and the
		// value is a string, return a new map containing a single entry where the key is
		// `@id` and the value is the result of IRI expanding `value` using `true` for
		// document relative.
		Some(Key::Keyword(Keyword::Vocab)) if value.is_string() => {
			let mut node = Node::new();
			node.id = Some(expand_iri(active_context, value.as_str().unwrap(), true, true));
			Ok(node.into())
		},

		_ => {
			// Otherwise, initialize `result` to a map with an `@value` entry whose value is set to
			// `value`.
			let mut result = match value {
				JsonValue::Null => Literal::Null,
				JsonValue::Boolean(b) => Literal::Boolean(*b),
				JsonValue::Number(n) => Literal::Number(*n),
				JsonValue::Short(_) | JsonValue::String(_) => Literal::String {
					data: value.as_str().unwrap().to_string(),
					language: None,
					direction: None
				},
				_ => panic!("expand_value must be called with a literal JSON value")
			};

			let mut types = HashSet::new();

			// If `active_property` has a type mapping in active context, other than `@id`,
			// `@vocab`, or `@none`, add `@type` to `result` and set its value to the value
			// associated with the type mapping.
			match active_property_type {
				None | Some(Key::Keyword(Keyword::Id)) | Some(Key::Keyword(Keyword::Vocab)) | Some(Key::Keyword(Keyword::None)) => {
					// Otherwise, if value is a string:
					if let Literal::String { ref mut language, ref mut direction, .. } = &mut result {
						// Initialize `language` to the language mapping for
						// `active_property` in `active_context`, if any, otherwise to the
						// default language of `active_context`.
						*language = if let Some(active_property_definition) = active_property_definition {
							if let Some(language) = &active_property_definition.language {
								language.clone()
							} else {
								clone_default_language(active_context)
							}
						} else {
							clone_default_language(active_context)
						};

						// Initialize `direction` to the direction mapping for
						// `active_property` in `active_context`, if any, otherwise to the
						// default base direction of `active_context`.
						*direction = if let Some(active_property_definition) = active_property_definition {
							if let Some(direction) = &active_property_definition.direction {
								direction.clone()
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
						// Done.
					}
				},

				Some(typ) => {
					types.insert(typ);
				}
			}

			Ok(Value::Literal(result, types).into())
		}
	}
}
