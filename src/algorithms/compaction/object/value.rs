use mown::Mown;

use crate::{
	algorithms::{
		context_processing::ContextProcessingOptions, ProcessingEnvironment,
		ProcessingEnvironmentRefMut,
	},
	object::LiteralValue,
	syntax::{Container, ContainerItem, Keyword},
	Error, Id, Term, Type, ValueObject,
};

use super::Compactor;

impl<'a> Compactor<'a> {
	/// Compact the given indexed value.
	pub async fn compact_indexed_value_with(
		&self,
		env: &mut impl ProcessingEnvironment,
		value: &ValueObject,
		index: Option<&str>,
		// active_property: Option<&str>,
	) -> Result<json_syntax::Value, Error> {
		// If the term definition for active property in active context has a local context:
		let mut active_context = Mown::Borrowed(self.active_context);
		if let Some(active_property) = self.active_property {
			if let Some(active_property_definition) = active_context.get(active_property) {
				if let Some(local_context) = active_property_definition.context() {
					active_context = Mown::Owned(
						local_context
							.process_with(
								ProcessingEnvironmentRefMut(env),
								active_property_definition.base_url(),
								active_context.as_ref(),
								ContextProcessingOptions::from(self.options).with_override(),
							)
							.await?,
					)
				}
			}
		}

		// If element has an @value or @id entry and the result of using the Value Compaction algorithm,
		// passing active context, active property, and element as value is a scalar,
		// or the term definition for active property has a type mapping of @json,
		// return that result.

		// Here starts the Value Compaction Algorithm.

		// Initialize result to a copy of value.
		let mut result = json_syntax::Object::default();

		// If the active context has a null inverse context,
		// set inverse context in active context to the result of calling the
		// Inverse Context Creation algorithm using active context.
		// NOTE never null here (FIXME is that true?)

		// Initialize inverse context to the value of inverse context in active context.
		// DONE

		let active_property_definition = match self.active_property {
			Some(active_property) => active_context.get(active_property),
			None => None,
		};

		// Initialize language to the language mapping for active property in active context,
		// if any, otherwise to the default language of active context.
		let language = match active_property_definition {
			Some(def) => match def.language() {
				Some(lang) => lang.as_ref().map(|l| l.as_lenient_lang_tag_ref()).option(),
				None => active_context.default_language(),
			},
			None => active_context.default_language(),
		};

		// Initialize direction to the direction mapping for active property in active context,
		// if any, otherwise to the default base direction of active context.
		let direction = match active_property_definition {
			Some(def) => match def.direction() {
				Some(dir) => dir.option(),
				None => active_context.default_base_direction(),
			},
			None => active_context.default_base_direction(),
		};

		// If value has an @id entry and has no other entries other than @index:
		// NOTE not possible here

		// Otherwise, if value has an @type entry whose value matches the type mapping of
		// active property, set result to the value associated with the @value entry of value.
		let type_mapping: Option<Type> = match active_property_definition {
			Some(def) => def.typ().cloned(),
			None => None,
		};

		let container_mapping = match active_property_definition {
			Some(def) => def.container(),
			None => Container::Null,
		};

		let remove_index = (index.is_some() && container_mapping.contains(ContainerItem::Index))
			|| index.is_none();

		match value {
			ValueObject::Literal(lit, ty) => {
				if ty.clone().map(Type::Iri) == type_mapping && remove_index {
					match lit {
						LiteralValue::Null => return Ok(json_syntax::Value::Null),
						LiteralValue::Boolean(b) => return Ok(json_syntax::Value::Boolean(*b)),
						LiteralValue::Number(n) => {
							return Ok(json_syntax::Value::Number(n.clone()))
						}
						LiteralValue::String(s) => {
							if ty.is_some() || (language.is_none() && direction.is_none()) {
								return Ok(json_syntax::Value::String(s.as_str().into()));
							} else {
								let compact_key = self
									.with_active_context(&active_context)
									.compact_key(&Term::Keyword(Keyword::Value), true, false)?;
								result.insert(
									compact_key.unwrap(),
									json_syntax::Value::String(s.as_str().into()),
								);
							}
						}
					}
				} else {
					let compact_key = self.with_active_context(&active_context).compact_key(
						&Term::Keyword(Keyword::Value),
						true,
						false,
					)?;
					match lit {
						LiteralValue::Null => {
							result.insert(compact_key.unwrap(), json_syntax::Value::Null);
						}
						LiteralValue::Boolean(b) => {
							result.insert(compact_key.unwrap(), json_syntax::Value::Boolean(*b));
						}
						LiteralValue::Number(n) => {
							result.insert(
								compact_key.unwrap(),
								json_syntax::Value::Number(n.clone()),
							);
						}
						LiteralValue::String(s) => {
							result.insert(
								compact_key.unwrap(),
								json_syntax::Value::String(s.as_str().into()),
							);
						}
					}

					if let Some(ty) = ty {
						let compact_key = self.with_active_context(&active_context).compact_key(
							&Term::Keyword(Keyword::Type),
							true,
							false,
						)?;
						let compact_ty = self.with_active_context(&active_context).compact_iri(
							&Term::Id(Id::iri(ty.clone())),
							true,
							false,
						)?;
						result.insert(
							compact_key.unwrap(),
							match compact_ty {
								Some(s) => json_syntax::Value::String(s.into()),
								None => json_syntax::Value::Null,
							},
						);
					}
				}
			}
			ValueObject::LangString(ls) => {
				let ls_language = ls.language(); //.map(|l| Nullable::Some(l));
				let ls_direction = ls.direction(); //.map(|d| Nullable::Some(d));

				if remove_index
				&& (ls_language.is_none() || language == ls_language) // || (ls.language().is_none() && language.is_none()))
				&& (ls_direction.is_none() || direction == ls_direction)
				{
					// || (ls.direction().is_none() && direction.is_none())) {
					return Ok(json_syntax::Value::String(ls.as_str().into()));
				} else {
					let compact_key = self.with_active_context(&active_context).compact_key(
						&Term::Keyword(Keyword::Value),
						true,
						false,
					)?;
					result.insert(
						compact_key.unwrap(),
						json_syntax::Value::String(ls.as_str().into()),
					);

					if let Some(language) = ls.language() {
						let compact_key = self.with_active_context(&active_context).compact_key(
							&Term::Keyword(Keyword::Language),
							true,
							false,
						)?;
						result.insert(
							compact_key.unwrap(),
							json_syntax::Value::String(language.as_str().into()),
						);
					}

					if let Some(direction) = ls.direction() {
						let compact_key = self.with_active_context(&active_context).compact_key(
							&Term::Keyword(Keyword::Direction),
							true,
							false,
						)?;
						result.insert(
							compact_key.unwrap(),
							json_syntax::Value::String(direction.as_str().into()),
						);
					}
				}
			}
			ValueObject::Json(value) => {
				if type_mapping == Some(Type::Json) && remove_index {
					return Ok(value.clone());
				} else {
					let compact_key = self.with_active_context(&active_context).compact_key(
						&Term::Keyword(Keyword::Value),
						true,
						false,
					)?;
					result.insert(compact_key.unwrap(), value.clone());

					let compact_key = self.with_active_context(&active_context).compact_key(
						&Term::Keyword(Keyword::Type),
						true,
						false,
					)?;

					let compact_ty = self.with_active_context(&active_context).compact_iri(
						&Term::Keyword(Keyword::Json),
						true,
						false,
					)?;
					result.insert(
						compact_key.unwrap(),
						match compact_ty {
							Some(s) => json_syntax::Value::String(s.into()),
							None => json_syntax::Value::Null,
						},
					);
				}
			}
		}

		if !remove_index {
			if let Some(index) = index {
				let compact_key = self.with_active_context(&active_context).compact_key(
					&Term::Keyword(Keyword::Index),
					true,
					false,
				)?;
				result.insert(
					compact_key.unwrap(),
					json_syntax::Value::String(index.into()),
				);
			}
		}

		Ok(json_syntax::Value::Object(result))
	}
}

/// Default value of `as_array` is false.
pub fn add_value(
	map: &mut json_syntax::Object,
	key: &str,
	value: json_syntax::Value,
	as_array: bool,
) {
	match map
		.get_unique(key)
		.ok()
		.unwrap()
		.map(|entry| entry.is_array())
	{
		Some(false) => {
			let (key, value) = map.remove_unique(key).ok().unwrap().unwrap();
			map.insert(key, json_syntax::Value::Array(vec![value]));
		}
		None if as_array => {
			map.insert(key, json_syntax::Value::Array(Vec::new()));
		}
		_ => (),
	}

	match value {
		json_syntax::Value::Array(values) => {
			for value in values {
				add_value(map, key, value, false)
			}
		}
		value => {
			if let Some(array) = map.get_unique_mut(key).ok().unwrap() {
				array.as_array_mut().unwrap().push(value);
				return;
			}

			map.insert(key, value);
		}
	}
}

/// Get the `@value` field of a value object.
pub fn value_value(value: &ValueObject) -> json_syntax::Value {
	match value {
		ValueObject::Literal(lit, _ty) => match lit {
			LiteralValue::Null => json_syntax::Value::Null,
			LiteralValue::Boolean(b) => json_syntax::Value::Boolean(*b),
			LiteralValue::Number(n) => json_syntax::Value::Number(n.clone()),
			LiteralValue::String(s) => json_syntax::Value::String(s.as_str().into()),
		},
		ValueObject::LangString(s) => json_syntax::Value::String(s.as_str().into()),
		ValueObject::Json(json) => json.clone(),
	}
}
