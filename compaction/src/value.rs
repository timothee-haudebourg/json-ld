use crate::{compact_iri, compact_key, MetaError, Options};
use json_ld_context_processing::{Options as ProcessingOptions, Process, ProcessMeta};
use json_ld_core::{
	object, Container, ContainerKind, Context, ContextLoader, Loader, Reference, Term, Type, Value,
};
use json_ld_syntax::{Entry, Keyword};
use locspan::Meta;
use mown::Mown;
use rdf_types::VocabularyMut;
use std::hash::Hash;

/// Compact the given indexed value.
pub async fn compact_indexed_value_with<
	I,
	B,
	M,
	C: ProcessMeta<I, B, M>,
	N,
	L: Loader<I, M> + ContextLoader<I, M>,
>(
	vocabulary: &mut N,
	Meta(value, meta): Meta<&Value<I, M>, &M>,
	index: Option<&Entry<String, M>>,
	active_context: &Context<I, B, C, M>,
	active_property: Option<Meta<&str, &M>>,
	loader: &mut L,
	options: Options,
) -> Result<json_syntax::MetaValue<M>, MetaError<M, L::ContextError>>
where
	N: Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
	I: Clone + Hash + Eq + Send + Sync,
	B: Clone + Hash + Eq + Send + Sync,
	M: Clone + Send + Sync,
	L: Send + Sync,
	L::Context: Into<C>,
{
	// If the term definition for active property in active context has a local context:
	let mut active_context = Mown::Borrowed(active_context);
	if let Some(Meta(active_property, _)) = active_property {
		if let Some(active_property_definition) = active_context.get(active_property) {
			if let Some(local_context) = &active_property_definition.context {
				active_context = Mown::Owned(
					local_context
						.value
						.process_with(
							vocabulary,
							active_context.as_ref(),
							loader,
							active_property_definition.base_url().cloned(),
							ProcessingOptions::from(options).with_override(),
						)
						.await
						.map_err(Meta::cast)?
						.into_processed(),
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

	let active_property_definition = match active_property {
		Some(Meta(active_property, _)) => active_context.get(active_property),
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
	let type_mapping: Option<Type<I>> = match active_property_definition {
		Some(def) => def.typ.clone(),
		None => None,
	};

	let container_mapping = match active_property_definition {
		Some(def) => def.container,
		None => Container::None,
	};

	let remove_index =
		(index.is_some() && container_mapping.contains(ContainerKind::Index)) || index.is_none();

	match value {
		Value::Literal(lit, ty) => {
			use object::value::Literal;
			if ty.clone().map(Type::Ref) == type_mapping && remove_index {
				match lit {
					Literal::Null => return Ok(Meta(json_syntax::Value::Null, meta.clone())),
					Literal::Boolean(b) => {
						return Ok(Meta(json_syntax::Value::Boolean(*b), meta.clone()))
					}
					Literal::Number(n) => {
						return Ok(Meta(json_syntax::Value::Number(n.clone()), meta.clone()))
					}
					Literal::String(s) => {
						if ty.is_some() || (language.is_none() && direction.is_none()) {
							return Ok(Meta(
								json_syntax::Value::String(s.as_str().into()),
								meta.clone(),
							));
						} else {
							let compact_key = compact_key(
								vocabulary,
								active_context.as_ref(),
								Meta(&Term::Keyword(Keyword::Value), meta),
								true,
								false,
								options,
							)
							.map_err(Meta::cast)?;
							result.insert(
								compact_key.unwrap(),
								Meta(json_syntax::Value::String(s.as_str().into()), meta.clone()),
							);
						}
					}
				}
			} else {
				let compact_key = compact_key(
					vocabulary,
					active_context.as_ref(),
					Meta(&Term::Keyword(Keyword::Value), meta),
					true,
					false,
					options,
				)
				.map_err(Meta::cast)?;
				match lit {
					Literal::Null => {
						result.insert(
							compact_key.unwrap(),
							Meta(json_syntax::Value::Null, meta.clone()),
						);
					}
					Literal::Boolean(b) => {
						result.insert(
							compact_key.unwrap(),
							Meta(json_syntax::Value::Boolean(*b), meta.clone()),
						);
					}
					Literal::Number(n) => {
						result.insert(
							compact_key.unwrap(),
							Meta(json_syntax::Value::Number(n.clone()), meta.clone()),
						);
					}
					Literal::String(s) => {
						result.insert(
							compact_key.unwrap(),
							Meta(json_syntax::Value::String(s.as_str().into()), meta.clone()),
						);
					}
				}

				if let Some(ty) = ty {
					let compact_key = crate::compact_key(
						vocabulary,
						active_context.as_ref(),
						Meta(&Term::Keyword(Keyword::Type), meta),
						true,
						false,
						options,
					)
					.map_err(Meta::cast)?;
					let compact_ty = compact_iri(
						vocabulary,
						active_context.as_ref(),
						Meta(&Term::Ref(Reference::id(ty.clone())), meta),
						true,
						false,
						options,
					)
					.map_err(Meta::cast)?;
					result.insert(
						compact_key.unwrap(),
						match compact_ty {
							Some(Meta(s, meta)) => Meta(json_syntax::Value::String(s.into()), meta),
							None => Meta(json_syntax::Value::Null, meta.clone()),
						},
					);
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
				return Ok(Meta(
					json_syntax::Value::String(ls.as_str().into()),
					meta.clone(),
				));
			} else {
				let compact_key = compact_key(
					vocabulary,
					active_context.as_ref(),
					Meta(&Term::Keyword(Keyword::Value), meta),
					true,
					false,
					options,
				)
				.map_err(Meta::cast)?;
				result.insert(
					compact_key.unwrap(),
					Meta(json_syntax::Value::String(ls.as_str().into()), meta.clone()),
				);

				if let Some(language) = ls.language() {
					let compact_key = crate::compact_key(
						vocabulary,
						active_context.as_ref(),
						Meta(&Term::Keyword(Keyword::Language), meta),
						true,
						false,
						options,
					)
					.map_err(Meta::cast)?;
					result.insert(
						compact_key.unwrap(),
						Meta(
							json_syntax::Value::String(language.as_str().into()),
							meta.clone(),
						),
					);
				}

				if let Some(direction) = ls.direction() {
					let compact_key = crate::compact_key(
						vocabulary,
						active_context.as_ref(),
						Meta(&Term::Keyword(Keyword::Direction), meta),
						true,
						false,
						options,
					)
					.map_err(Meta::cast)?;
					result.insert(
						compact_key.unwrap(),
						Meta(
							json_syntax::Value::String(direction.as_str().into()),
							meta.clone(),
						),
					);
				}
			}
		}
		Value::Json(value) => {
			if type_mapping == Some(Type::Json) && remove_index {
				return Ok(value.clone());
			} else {
				let compact_key = compact_key(
					vocabulary,
					active_context.as_ref(),
					Meta(&Term::Keyword(Keyword::Value), meta),
					true,
					false,
					options,
				)
				.map_err(Meta::cast)?;
				result.insert(compact_key.unwrap(), value.clone());

				let compact_key = crate::compact_key(
					vocabulary,
					active_context.as_ref(),
					Meta(&Term::Keyword(Keyword::Type), meta),
					true,
					false,
					options,
				)
				.map_err(Meta::cast)?;

				let compact_ty = compact_iri(
					vocabulary,
					active_context.as_ref(),
					Meta(&Term::Keyword(Keyword::Json), meta),
					true,
					false,
					options,
				)
				.map_err(Meta::cast)?;
				result.insert(
					compact_key.unwrap(),
					match compact_ty {
						Some(Meta(s, meta)) => Meta(json_syntax::Value::String(s.into()), meta),
						None => Meta(json_syntax::Value::Null, meta.clone()),
					},
				);
			}
		}
	}

	if !remove_index {
		if let Some(index) = index {
			let compact_key = compact_key(
				vocabulary,
				active_context.as_ref(),
				Meta(&Term::Keyword(Keyword::Index), &index.key_metadata),
				true,
				false,
				options,
			)
			.map_err(Meta::cast)?;
			result.insert(
				compact_key.unwrap(),
				Meta(
					json_syntax::Value::String(index.value.as_str().into()),
					index.value.metadata().clone(),
				),
			);
		}
	}

	Ok(Meta(json_syntax::Value::Object(result), meta.clone()))
}
