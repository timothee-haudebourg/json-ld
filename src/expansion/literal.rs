use super::{expand_iri, node_id_of_term, ActiveProperty};
use crate::{
	loader, object::*, syntax::Type, Context, Error, ErrorCode, Id, Indexed, LangString, Loc,
	Warning,
};
use generic_json::{Json, JsonClone, JsonHash, ValueRef};

pub enum LiteralValue<'a, J: Json> {
	Given(&'a J),
	Inferred(String, J::MetaData),
}

impl<'a, J: Json> LiteralValue<'a, J> {
	pub fn is_string(&self) -> bool {
		match self {
			Self::Given(v) => v.is_string(),
			Self::Inferred(_, _) => true,
		}
	}

	pub fn as_str(&self) -> Option<&str> {
		match self {
			Self::Given(v) => v.as_str(),
			Self::Inferred(s, _) => Some(s.as_str()),
		}
	}

	pub fn metadata(&self) -> &J::MetaData {
		match self {
			Self::Given(v) => v.metadata(),
			Self::Inferred(_, meta) => meta,
		}
	}
}

/// Expand a literal value.
/// See <https://www.w3.org/TR/json-ld11-api/#value-expansion>.
pub fn expand_literal<J: JsonHash + JsonClone, T: Id, C: Context<T>>(
	source: Option<loader::Id>,
	active_context: &C,
	active_property: ActiveProperty<J>,
	value: LiteralValue<J>,
	warnings: &mut Vec<Loc<Warning, J::MetaData>>,
) -> Result<Indexed<Object<J, T>>, Error> {
	let active_property_definition = active_context.get_opt(active_property.id());

	let active_property_type = if let Some(active_property_definition) = active_property_definition
	{
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
			node.id = node_id_of_term(expand_iri(
				source,
				active_context,
				value.as_str().unwrap(),
				value.metadata(),
				true,
				false,
				warnings,
			));
			Ok(Object::Node(node).into())
		}

		// If `active_property` has a type mapping in active context that is `@vocab`, and the
		// value is a string, return a new map containing a single entry where the key is
		// `@id` and the value is the result of IRI expanding `value` using `true` for
		// document relative.
		Some(Type::Vocab) if value.is_string() => {
			let mut node = Node::new();
			node.id = node_id_of_term(expand_iri(
				source,
				active_context,
				value.as_str().unwrap(),
				value.metadata(),
				true,
				true,
				warnings,
			));
			Ok(Object::Node(node).into())
		}

		_ => {
			// Otherwise, initialize `result` to a map with an `@value` entry whose value is set to
			// `value`.
			let result: Literal<J> = match value {
				LiteralValue::Given(v) => match v.as_value_ref() {
					ValueRef::Null => Literal::Null,
					ValueRef::Boolean(b) => Literal::Boolean(b),
					ValueRef::Number(n) => Literal::Number(n.clone()),
					ValueRef::String(s) => Literal::String(LiteralString::Expanded(s.clone())),
					_ => panic!("expand_literal must be called with a literal JSON value"),
				},
				LiteralValue::Inferred(s, _) => Literal::String(LiteralString::Inferred(s)),
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
						let language =
							if let Some(active_property_definition) = active_property_definition {
								if let Some(language) = &active_property_definition.language {
									language.as_ref().cloned().option()
								} else {
									active_context.default_language().map(|lang| lang.cloned())
								}
							} else {
								active_context.default_language().map(|lang| lang.cloned())
							};

						// Initialize `direction` to the direction mapping for
						// `active_property` in `active_context`, if any, otherwise to the
						// default base direction of `active_context`.
						let direction =
							if let Some(active_property_definition) = active_property_definition {
								if let Some(direction) = &active_property_definition.direction {
									(*direction).option()
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
						return match LangString::new(str, language, direction) {
							Ok(lang_str) => Ok(Object::Value(Value::LangString(lang_str)).into()),
							Err(str) => Ok(Object::Value(Value::Literal(
								Literal::String(str),
								None,
							))
							.into()),
						};
					}
				}

				Some(t) => {
					if let Ok(t) = t.into_ref() {
						ty = Some(t)
					} else {
						return Err(ErrorCode::InvalidTypeValue.into());
					}
				}
			}

			Ok(Object::Value(Value::Literal(result, ty)).into())
		}
	}
}
