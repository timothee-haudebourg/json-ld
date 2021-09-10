use super::{compact_iri, Options};
use crate::{
	context::{self, Inversible, Loader, Local},
	syntax::{Container, ContainerType, Keyword, Term, Type},
	util::AsJson,
	ContextMut, Error, Id, Reference, Value,
};
use json::JsonValue;

/// Compact the given indexed value.
pub async fn compact_indexed_value_with<T: Sync + Send + Id, C: ContextMut<T>, L: Loader>(
	value: &Value<T>,
	index: Option<&str>,
	active_context: Inversible<T, &C>,
	active_property: Option<&str>,
	loader: &mut L,
	options: Options,
) -> Result<JsonValue, Error>
where
	C: Sync + Send,
	C::LocalContext: Send + Sync + From<L::Output>,
	L: Sync + Send,
{
	// If the term definition for active property in active context has a local context:
	let mut active_context = active_context.into_borrowed();
	if let Some(active_property) = active_property {
		if let Some(active_property_definition) = active_context.get(active_property) {
			if let Some(local_context) = &active_property_definition.context {
				active_context = Inversible::new(
					local_context
						.process_with(
							*active_context.as_ref(),
							loader,
							active_property_definition.base_url(),
							context::ProcessingOptions::from(options).with_override(),
						)
						.await?
						.into_inner(),
				)
				.into_owned()
			}
		}
	}

	// If element has an @value or @id entry and the result of using the Value Compaction algorithm,
	// passing active context, active property, and element as value is a scalar,
	// or the term definition for active property has a type mapping of @json,
	// return that result.

	// Here starts the Value Compaction Algorithm.

	// Initialize result to a copy of value.
	let mut result = json::object::Object::new();

	// If the active context has a null inverse context,
	// set inverse context in active context to the result of calling the
	// Inverse Context Creation algorithm using active context.
	// NOTE never null here (FIXME is that true?)

	// Initialize inverse context to the value of inverse context in active context.
	// DONE

	let active_property_definition = match active_property {
		Some(active_property) => active_context.get(active_property),
		None => None,
	};

	// Initialize language to the language mapping for active property in active context,
	// if any, otherwise to the default language of active context.
	let language = match active_property_definition {
		Some(def) => match def.language.as_ref() {
			Some(lang) => lang.as_ref().map(|l| l.as_ref()).option(),
			None => active_context.default_language(),
		},
		None => active_context.default_language(),
	};

	// Initialize direction to the direction mapping for active property in active context,
	// if any, otherwise to the default base direction of active context.
	let direction = match active_property_definition {
		Some(def) => match def.direction {
			Some(dir) => dir.option(),
			None => active_context.default_base_direction(),
		},
		None => active_context.default_base_direction(),
	};

	// If value has an @id entry and has no other entries other than @index:
	// NOTE not possible here

	// Otherwise, if value has an @type entry whose value matches the type mapping of
	// active property, set result to the value associated with the @value entry of value.
	let type_mapping: Option<Type<&T>> = match active_property_definition {
		Some(def) => def.typ.as_ref().map(|t| t.into()),
		None => None,
	};

	let container_mapping = match active_property_definition {
		Some(def) => def.container,
		None => Container::None,
	};

	let remove_index =
		(index.is_some() && container_mapping.contains(ContainerType::Index)) || index.is_none();

	match value {
		Value::Literal(lit, ty) => {
			use crate::object::value::Literal;
			if ty.as_ref().map(|t| Type::Ref(t)) == type_mapping && remove_index {
				match lit {
					Literal::Null => return Ok(JsonValue::Null),
					Literal::Boolean(b) => return Ok(b.as_json()),
					Literal::Number(n) => return Ok(JsonValue::Number(n.clone())),
					Literal::String(s) => {
						if ty.is_some() || (language.is_none() && direction.is_none()) {
							return Ok(s.as_json());
						} else {
							let compact_key = compact_iri(
								active_context.as_ref(),
								&Term::Keyword(Keyword::Value),
								true,
								false,
								options,
							)?;
							result.insert(compact_key.as_str().unwrap(), s.as_json())
						}
					}
				}
			} else {
				let compact_key = compact_iri(
					active_context.as_ref(),
					&Term::Keyword(Keyword::Value),
					true,
					false,
					options,
				)?;
				match lit {
					Literal::Null => result.insert(compact_key.as_str().unwrap(), JsonValue::Null),
					Literal::Boolean(b) => {
						result.insert(compact_key.as_str().unwrap(), b.as_json())
					}
					Literal::Number(n) => {
						result.insert(compact_key.as_str().unwrap(), JsonValue::Number(n.clone()))
					}
					Literal::String(s) => result.insert(compact_key.as_str().unwrap(), s.as_json()),
				}

				if let Some(ty) = ty {
					let compact_key = compact_iri(
						active_context.as_ref(),
						&Term::Keyword(Keyword::Type),
						true,
						false,
						options,
					)?;
					let compact_ty = compact_iri(
						active_context.as_ref(),
						&Term::Ref(Reference::Id(ty.clone())),
						true,
						false,
						options,
					)?;
					result.insert(compact_key.as_str().unwrap(), compact_ty)
				}
			}
		}
		Value::LangString(ls) => {
			let ls_language = ls.language(); //.map(|l| Nullable::Some(l));
			let ls_direction = ls.direction(); //.map(|d| Nullable::Some(d));

			if remove_index
			&& (ls_language.is_none() || language == ls_language) // || (ls.language().is_none() && language.is_none()))
			&& (ls_direction.is_none() || direction == ls_direction)
			{
				// || (ls.direction().is_none() && direction.is_none())) {
				return Ok(ls.as_str().as_json());
			} else {
				let compact_key = compact_iri(
					active_context.as_ref(),
					&Term::Keyword(Keyword::Value),
					true,
					false,
					options,
				)?;
				result.insert(compact_key.as_str().unwrap(), ls.as_str().into());

				if let Some(language) = ls.language() {
					let compact_key = compact_iri(
						active_context.as_ref(),
						&Term::Keyword(Keyword::Language),
						true,
						false,
						options,
					)?;
					result.insert(compact_key.as_str().unwrap(), language.as_json());
				}

				if let Some(direction) = ls.direction() {
					let compact_key = compact_iri(
						active_context.as_ref(),
						&Term::Keyword(Keyword::Direction),
						true,
						false,
						options,
					)?;
					result.insert(compact_key.as_str().unwrap(), direction.as_json());
				}
			}
		}
		Value::Json(value) => {
			if type_mapping == Some(Type::Json) && remove_index {
				return Ok(value.clone());
			} else {
				let compact_key = compact_iri(
					active_context.as_ref(),
					&Term::Keyword(Keyword::Value),
					true,
					false,
					options,
				)?;
				result.insert(compact_key.as_str().unwrap(), value.clone());

				let compact_key = compact_iri(
					active_context.as_ref(),
					&Term::Keyword(Keyword::Type),
					true,
					false,
					options,
				)?;
				let compact_ty = compact_iri(
					active_context.as_ref(),
					&Term::Keyword(Keyword::Json),
					true,
					false,
					options,
				)?;
				result.insert(compact_key.as_str().unwrap(), compact_ty);
			}
		}
	}

	if !remove_index {
		if let Some(index) = index {
			let compact_key = compact_iri(
				active_context.as_ref(),
				&Term::Keyword(Keyword::Index),
				true,
				false,
				options,
			)?;
			result.insert(compact_key.as_str().unwrap(), index.as_json())
		}
	}

	Ok(JsonValue::Object(result))
}
