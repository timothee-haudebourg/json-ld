use crate::{
	context::{
		self,
		inverse::{Inversible, LangSelection, TypeSelection},
		Loader, Local,
	},
	object,
	syntax::{ContainerType, Keyword, Term},
	util::AsJson,
	ContextMut, Error, Id, Indexed, Object, ProcessingMode, Value,
};
use futures::future::{BoxFuture, FutureExt};
use json::JsonValue;
use std::collections::HashSet;

mod iri;
mod node;
mod property;
mod value;

pub(crate) use iri::*;
use node::*;
use property::*;
use value::*;

#[derive(Clone, Copy)]
pub struct Options {
	pub processing_mode: ProcessingMode,
	pub compact_to_relative: bool,
	pub compact_arrays: bool,
	pub ordered: bool,
}

impl From<Options> for context::ProcessingOptions {
	fn from(options: Options) -> context::ProcessingOptions {
		context::ProcessingOptions {
			processing_mode: options.processing_mode,
			..Default::default()
		}
	}
}

impl From<crate::expansion::Options> for Options {
	fn from(options: crate::expansion::Options) -> Options {
		Options {
			processing_mode: options.processing_mode,
			ordered: options.ordered,
			..Options::default()
		}
	}
}

impl Default for Options {
	fn default() -> Options {
		Options {
			processing_mode: ProcessingMode::default(),
			compact_to_relative: true,
			compact_arrays: true,
			ordered: false,
		}
	}
}

pub trait Compact<T: Id> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(
		&'a self,
		active_context: Inversible<T, &'a C>,
		type_scoped_context: Inversible<T, &'a C>,
		active_property: Option<&'a str>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, Result<JsonValue, Error>>
	where
		T: 'a,
		C: Sync + Send,
		C::LocalContext: Send + Sync + From<L::Output>,
		L: Sync + Send;

	fn compact<'a, C: ContextMut<T>, L: Loader>(
		&'a self,
		active_context: Inversible<T, &'a C>,
		loader: &'a mut L,
	) -> BoxFuture<'a, Result<JsonValue, Error>>
	where
		Self: Sync,
		T: 'a + Sync + Send,
		C: Sync + Send,
		C::LocalContext: Send + Sync + From<L::Output>,
		L: Sync + Send,
	{
		async move {
			self.compact_with(
				active_context.clone(),
				active_context,
				None,
				loader,
				Options::default(),
			)
			.await
		}
		.boxed()
	}
}

enum TypeLangValue<'a, T: Id> {
	Type(TypeSelection<T>),
	Lang(LangSelection<'a>),
}

pub trait CompactIndexed<T: Id> {
	fn compact_indexed_with<'a, C: ContextMut<T>, L: Loader>(
		&'a self,
		index: Option<&'a str>,
		active_context: Inversible<T, &'a C>,
		type_scoped_context: Inversible<T, &'a C>,
		active_property: Option<&'a str>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, Result<JsonValue, Error>>
	where
		T: 'a,
		C: Sync + Send,
		C::LocalContext: Send + Sync + From<L::Output>,
		L: Sync + Send;
}

impl<T: Sync + Send + Id, V: Sync + Send + CompactIndexed<T>> Compact<T> for Indexed<V> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(
		&'a self,
		active_context: Inversible<T, &'a C>,
		type_scoped_context: Inversible<T, &'a C>,
		active_property: Option<&'a str>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, Result<JsonValue, Error>>
	where
		T: 'a,
		C: Sync + Send,
		C::LocalContext: Send + Sync + From<L::Output>,
		L: Sync + Send,
	{
		self.inner().compact_indexed_with(
			self.index(),
			active_context,
			type_scoped_context,
			active_property,
			loader,
			options,
		)
	}
}

impl<T: Sync + Send + Id, N: object::Any<T> + Sync + Send> CompactIndexed<T> for N {
	fn compact_indexed_with<'a, C: ContextMut<T>, L: Loader>(
		&'a self,
		index: Option<&'a str>,
		active_context: Inversible<T, &'a C>,
		type_scoped_context: Inversible<T, &'a C>,
		active_property: Option<&'a str>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, Result<JsonValue, Error>>
	where
		T: 'a,
		C: Sync + Send,
		C::LocalContext: Send + Sync + From<L::Output>,
		L: Sync + Send,
	{
		match self.as_ref() {
			object::Ref::Value(value) => async move {
				compact_indexed_value_with(
					value,
					index,
					active_context,
					active_property,
					loader,
					options,
				)
				.await
			}
			.boxed(),
			object::Ref::Node(node) => async move {
				compact_indexed_node_with(
					node,
					index,
					active_context,
					type_scoped_context,
					active_property,
					loader,
					options,
				)
				.await
			}
			.boxed(),
			object::Ref::List(list) => async move {
				let mut active_context = active_context;
				// If active context has a previous context, the active context is not propagated.
				// If element does not contain an @value entry, and element does not consist of
				// a single @id entry, set active context to previous context from active context,
				// as the scope of a term-scoped context does not apply when processing new node objects.
				if let Some(previous_context) = active_context.previous_context() {
					active_context = Inversible::new(previous_context)
				}

				// If the term definition for active property in active context has a local context:
				// FIXME https://github.com/w3c/json-ld-api/issues/502
				//       Seems that the term definition should be looked up in `type_scoped_context`.
				let mut active_context = active_context.into_borrowed();
				let mut list_container = false;
				if let Some(active_property) = active_property {
					if let Some(active_property_definition) =
						type_scoped_context.get(active_property)
					{
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

						list_container = active_property_definition
							.container
							.contains(ContainerType::List);
					}
				}

				if list_container {
					compact_collection_with(
						list.iter(),
						active_context.as_ref(),
						active_context.as_ref(),
						active_property,
						loader,
						options,
					)
					.await
				} else {
					let mut result = json::object::Object::new();
					compact_property(
						&mut result,
						Term::Keyword(Keyword::List),
						list,
						active_context.as_ref(),
						loader,
						false,
						options,
					)
					.await?;

					// If expanded property is @index and active property has a container mapping in
					// active context that includes @index,
					if let Some(index) = index {
						let mut index_container = false;
						if let Some(active_property) = active_property {
							if let Some(active_property_definition) =
								active_context.get(active_property)
							{
								if active_property_definition
									.container
									.contains(ContainerType::Index)
								{
									// then the compacted result will be inside of an @index container,
									// drop the @index entry by continuing to the next expanded property.
									index_container = true;
								}
							}
						}

						if !index_container {
							// Initialize alias by IRI compacting expanded property.
							let alias = compact_iri(
								active_context.as_ref(),
								&Term::Keyword(Keyword::Index),
								true,
								false,
								options,
							)?;

							// Add an entry alias to result whose value is set to expanded value and continue with the next expanded property.
							result.insert(alias.as_str().unwrap(), index.as_json());
						}
					}

					Ok(JsonValue::Object(result))
				}
			}
			.boxed(),
		}
	}
}

/// Default value of `as_array` is false.
fn add_value(map: &mut json::object::Object, key: &str, value: JsonValue, as_array: bool) {
	match map.get(key) {
		Some(JsonValue::Array(_)) => (),
		Some(original_value) => {
			let value = original_value.clone();
			map.insert(key, JsonValue::Array(vec![value]))
		}
		None if as_array => map.insert(key, JsonValue::Array(Vec::new())),
		None => (),
	}

	match value {
		JsonValue::Array(values) => {
			for value in values {
				add_value(map, key, value, false)
			}
		}
		value => match map.get_mut(key) {
			Some(JsonValue::Array(values)) => values.push(value),
			Some(_) => unreachable!(),
			None => map.insert(key, value),
		},
	}
}

/// Get the `@value` field of a value object.
fn value_value<T: Id>(value: &Value<T>) -> JsonValue {
	use crate::object::value::Literal;
	match value {
		Value::Literal(lit, _ty) => match lit {
			Literal::Null => JsonValue::Null,
			Literal::Boolean(b) => b.as_json(),
			Literal::Number(n) => JsonValue::Number(*n),
			Literal::String(s) => s.as_json(),
		},
		Value::LangString(str) => str.as_str().into(),
		Value::Json(json) => json.clone(),
	}
}

fn compact_collection_with<
	'a,
	T: 'a + Sync + Send + Id,
	O: 'a + Send + Iterator<Item = &'a Indexed<Object<T>>>,
	C: ContextMut<T>,
	L: Loader,
>(
	items: O,
	active_context: Inversible<T, &'a C>,
	type_scoped_context: Inversible<T, &'a C>,
	active_property: Option<&'a str>,
	loader: &'a mut L,
	options: Options,
) -> BoxFuture<'a, Result<JsonValue, Error>>
where
	C: Sync + Send,
	C::LocalContext: Send + Sync + From<L::Output>,
	L: Sync + Send,
{
	async move {
		let mut result = Vec::new();

		for item in items {
			match item
				.compact_with(
					active_context.clone(),
					type_scoped_context.clone(),
					active_property,
					loader,
					options,
				)
				.await?
			{
				JsonValue::Null => (),
				compacted_item => result.push(compacted_item),
			}
		}

		let mut list_or_set = false;
		if let Some(active_property) = active_property {
			if let Some(active_property_definition) = active_context.get(active_property) {
				list_or_set = active_property_definition
					.container
					.contains(ContainerType::List)
					|| active_property_definition
						.container
						.contains(ContainerType::Set);
			}
		}

		if result.is_empty()
			|| result.len() > 1
			|| !options.compact_arrays
			|| active_property == Some("@graph")
			|| active_property == Some("@set")
			|| list_or_set
		{
			return Ok(JsonValue::Array(result));
		}

		Ok(result.into_iter().next().unwrap())
	}
	.boxed()
}

impl<T: Sync + Send + Id> Compact<T> for HashSet<Indexed<Object<T>>> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(
		&'a self,
		active_context: Inversible<T, &'a C>,
		type_scoped_context: Inversible<T, &'a C>,
		active_property: Option<&'a str>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, Result<JsonValue, Error>>
	where
		T: 'a,
		C: Sync + Send,
		C::LocalContext: Send + Sync + From<L::Output>,
		L: Sync + Send,
	{
		compact_collection_with(
			self.iter(),
			active_context,
			type_scoped_context,
			active_property,
			loader,
			options,
		)
	}
}
