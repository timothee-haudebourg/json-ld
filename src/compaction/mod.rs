//! Compaction algorithm and related types.
use crate::{
	context::{
		self,
		inverse::{Inversible, LangSelection, TypeSelection},
		Loader, Local,
	},
	object,
	syntax::{ContainerType, Keyword, Term},
	util::{AsAnyJson, AsJson, JsonFrom},
	ContextMut, Error, Id, Indexed, Object, ProcessingMode, Value,
};
use futures::future::{BoxFuture, FutureExt};
use generic_json::{JsonBuild, JsonClone, JsonHash, JsonMut, JsonSendSync};
use std::collections::HashSet;

/// JSON type that can be used by the compaction algorithm.
pub trait JsonSrc = JsonClone + JsonHash + JsonSendSync;

mod iri;
mod node;
mod property;
mod value;

pub(crate) use iri::*;
use node::*;
use property::*;
use value::*;

fn optional_string<K: JsonBuild>(s: Option<String>, meta: K::MetaData) -> K {
	match s {
		Some(s) => K::string(s.as_str().into(), meta),
		None => K::null(meta),
	}
}

/// Compaction options.
#[derive(Clone, Copy)]
pub struct Options {
	/// JSON-LD processing mode.
	pub processing_mode: ProcessingMode,

	/// Determines if IRIs are compacted relative to the provided base IRI or document location when compacting.
	pub compact_to_relative: bool,

	/// If set to `true`, arrays with just one element are replaced with that element during compaction.
	/// If set to `false`, all arrays will remain arrays even if they have just one element.
	pub compact_arrays: bool,

	/// If set to `true`, properties are processed by lexical order.
	/// If `false`, order is not considered in processing.
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

/// Type that can be compacted.
pub trait Compact<J: JsonSrc, T: Id> {
	/// Compact a JSON-LD document into a `K` JSON value with the provided
	/// type scoped context, active property and options.
	///
	/// Unless you know what you are doing, you will probably prefer
	/// to use the [`compact`](Compact::compact) and [`compact_with`](Compact::compact_with) functions.
	fn compact_full<'a, K: JsonFrom<J>, C: ContextMut<T>, L: Loader, M>(
		&'a self,
		active_context: Inversible<T, &'a C>,
		type_scoped_context: Inversible<T, &'a C>,
		active_property: Option<&'a str>,
		loader: &'a mut L,
		options: Options,
		meta: M,
	) -> BoxFuture<'a, Result<K, Error>>
	where
		J: 'a,
		T: 'a,
		C: Sync + Send,
		C::LocalContext: Send + Sync + From<L::Output>,
		L: Sync + Send,
		M: 'a + Send + Sync + Clone + Fn(Option<&J::MetaData>) -> K::MetaData;

	/// Compact a JSON-LD document into a `K` JSON value with the provided options.
	///
	/// This calls [`compact_full`](Compact::compact_full) with `active_context`
	/// as type scoped context.
	#[inline(always)]
	fn compact_with<'a, K: JsonFrom<J>, C: ContextMut<T>, L: Loader, M>(
		&'a self,
		active_context: Inversible<T, &'a C>,
		loader: &'a mut L,
		options: Options,
		meta: M,
	) -> BoxFuture<'a, Result<K, Error>>
	where
		Self: Sync,
		T: 'a + Sync + Send,
		C: Sync + Send,
		C::LocalContext: Send + Sync + From<L::Output>,
		L: Sync + Send,
		M: 'a + Send + Sync + Clone + Fn(Option<&J::MetaData>) -> K::MetaData,
	{
		async move {
			self.compact_full(
				active_context.clone(),
				active_context,
				None,
				loader,
				options,
				meta,
			)
			.await
		}
		.boxed()
	}

	/// Compact a JSON-LD document into a `K` JSON value with the default options.
	#[inline(always)]
	fn compact<'a, K: JsonFrom<J>, C: ContextMut<T>, L: Loader, M>(
		&'a self,
		active_context: Inversible<T, &'a C>,
		loader: &'a mut L,
		meta: M,
	) -> BoxFuture<'a, Result<K, Error>>
	where
		Self: Sync,
		T: 'a + Sync + Send,
		C: Sync + Send,
		C::LocalContext: Send + Sync + From<L::Output>,
		L: Sync + Send,
		M: 'a + Send + Sync + Clone + Fn(Option<&J::MetaData>) -> K::MetaData,
	{
		self.compact_with(active_context, loader, Options::default(), meta)
	}
}

enum TypeLangValue<'a, T: Id> {
	Type(TypeSelection<T>),
	Lang(LangSelection<'a>),
}

/// Type that can be compacted with an index.
pub trait CompactIndexed<J: JsonSrc, T: Id> {
	/// Compact with the given optional index.
	fn compact_indexed<'a, K: JsonFrom<J>, C: ContextMut<T>, L: Loader, M>(
		&'a self,
		index: Option<&'a str>,
		active_context: Inversible<T, &'a C>,
		type_scoped_context: Inversible<T, &'a C>,
		active_property: Option<&'a str>,
		loader: &'a mut L,
		options: Options,
		meta: M,
	) -> BoxFuture<'a, Result<K, Error>>
	where
		J: 'a,
		T: 'a,
		C: Sync + Send,
		C::LocalContext: Send + Sync + From<L::Output>,
		L: Sync + Send,
		M: 'a + Send + Sync + Clone + Fn(Option<&J::MetaData>) -> K::MetaData;
}

impl<J: JsonSrc, T: Sync + Send + Id, V: Sync + Send + CompactIndexed<J, T>> Compact<J, T>
	for Indexed<V>
{
	fn compact_full<'a, K: JsonFrom<J>, C: ContextMut<T>, L: Loader, M>(
		&'a self,
		active_context: Inversible<T, &'a C>,
		type_scoped_context: Inversible<T, &'a C>,
		active_property: Option<&'a str>,
		loader: &'a mut L,
		options: Options,
		meta: M,
	) -> BoxFuture<'a, Result<K, Error>>
	where
		J: 'a,
		T: 'a,
		C: Sync + Send,
		C::LocalContext: Send + Sync + From<L::Output>,
		L: Sync + Send,
		M: 'a + Send + Sync + Clone + Fn(Option<&J::MetaData>) -> K::MetaData,
	{
		self.inner().compact_indexed(
			self.index(),
			active_context,
			type_scoped_context,
			active_property,
			loader,
			options,
			meta,
		)
	}
}

impl<J: JsonSrc, T: Sync + Send + Id, N: object::Any<J, T> + Sync + Send> CompactIndexed<J, T>
	for N
{
	fn compact_indexed<'a, K: JsonFrom<J>, C: ContextMut<T>, L: Loader, M>(
		&'a self,
		index: Option<&'a str>,
		active_context: Inversible<T, &'a C>,
		type_scoped_context: Inversible<T, &'a C>,
		active_property: Option<&'a str>,
		loader: &'a mut L,
		options: Options,
		meta: M,
	) -> BoxFuture<'a, Result<K, Error>>
	where
		J: 'a,
		T: 'a,
		C: Sync + Send,
		C::LocalContext: Send + Sync + From<L::Output>,
		L: Sync + Send,
		M: 'a + Send + Sync + Clone + Fn(Option<&J::MetaData>) -> K::MetaData,
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
					meta,
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
					meta,
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
						meta,
					)
					.await
				} else {
					let mut result = K::Object::default();
					compact_property::<J, K, _, _, _, _, _, _>(
						&mut result,
						Term::Keyword(Keyword::List),
						list,
						active_context.as_ref(),
						loader,
						false,
						options,
						meta.clone(),
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
							let alias = compact_iri::<J, _, _>(
								active_context.as_ref(),
								&Term::Keyword(Keyword::Index),
								true,
								false,
								options,
							)?;

							// Add an entry alias to result whose value is set to expanded value and continue with the next expanded property.
							result.insert(
								K::new_key(alias.unwrap().as_str(), meta(None)),
								index.as_json_with(meta(None)),
							);
						}
					}

					Ok(K::object(result, meta(None)))
				}
			}
			.boxed(),
		}
	}
}

/// Default value of `as_array` is false.
fn add_value<K: JsonBuild + JsonMut>(
	map: &mut K::Object,
	key: &str,
	value: K,
	as_array: bool,
	meta: impl Clone + Fn() -> K::MetaData,
) {
	match map.get(key).map(|value| value.is_array()) {
		Some(false) => {
			let value = map.remove(key).unwrap();
			map.insert(
				K::new_key(key, meta()),
				K::array(Some(value).into_iter().collect(), meta()),
			);
		}
		None if as_array => {
			map.insert(K::new_key(key, meta()), K::empty_array(meta()));
		}
		_ => (),
	}

	match value.into_parts() {
		(generic_json::Value::Array(values), _) => {
			for value in values {
				add_value(map, key, value, false, meta.clone())
			}
		}
		(value, metadata) => {
			if let Some(mut array) = map.get_mut(key) {
				array
					.as_array_mut()
					.unwrap()
					.push_back(K::new(value, metadata));
				return;
			}

			map.insert(K::new_key(key, meta()), K::new(value, metadata));
		}
	}
}

/// Get the `@value` field of a value object.
fn value_value<J: JsonClone, K: JsonFrom<J>, T: Id, M>(value: &Value<J, T>, meta: M) -> K
where
	M: Clone + Fn(Option<&J::MetaData>) -> K::MetaData,
{
	use crate::object::value::Literal;
	match value {
		Value::Literal(lit, _ty) => match lit {
			Literal::Null => K::null(meta(None)),
			Literal::Boolean(b) => b.as_json_with(meta(None)),
			Literal::Number(n) => K::number(n.clone().into(), meta(None)),
			Literal::String(s) => s.as_json_with(meta(None)),
		},
		Value::LangString(str) => K::string(str.as_str().into(), meta(None)),
		Value::Json(json) => json.as_json_with(meta),
	}
}

fn compact_collection_with<
	'a,
	J: 'a + JsonSrc,
	K: JsonFrom<J>,
	T: 'a + Sync + Send + Id,
	O: 'a + Send + Iterator<Item = &'a Indexed<Object<J, T>>>,
	C: ContextMut<T>,
	L: Loader,
	M: 'a,
>(
	items: O,
	active_context: Inversible<T, &'a C>,
	type_scoped_context: Inversible<T, &'a C>,
	active_property: Option<&'a str>,
	loader: &'a mut L,
	options: Options,
	meta: M,
) -> BoxFuture<'a, Result<K, Error>>
where
	C: Sync + Send,
	C::LocalContext: Send + Sync + From<L::Output>,
	L: Sync + Send,
	M: Send + Sync + Clone + Fn(Option<&J::MetaData>) -> K::MetaData,
{
	async move {
		let mut result = Vec::new();

		for item in items {
			let compacted_item: K = item
				.compact_full(
					active_context.clone(),
					type_scoped_context.clone(),
					active_property,
					loader,
					options,
					meta.clone(),
				)
				.await?;

			if !compacted_item.is_null() {
				result.push(compacted_item)
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
			return Ok(K::array(result.into_iter().collect(), meta(None)));
		}

		Ok(result.into_iter().next().unwrap())
	}
	.boxed()
}

impl<J: JsonSrc, T: Sync + Send + Id> Compact<J, T> for HashSet<Indexed<Object<J, T>>> {
	fn compact_full<'a, K: JsonFrom<J>, C: ContextMut<T>, L: Loader, M>(
		&'a self,
		active_context: Inversible<T, &'a C>,
		type_scoped_context: Inversible<T, &'a C>,
		active_property: Option<&'a str>,
		loader: &'a mut L,
		options: Options,
		meta: M,
	) -> BoxFuture<'a, Result<K, Error>>
	where
		T: 'a,
		C: Sync + Send,
		C::LocalContext: Send + Sync + From<L::Output>,
		L: Sync + Send,
		M: 'a + Send + Sync + Clone + Fn(Option<&J::MetaData>) -> K::MetaData,
	{
		compact_collection_with(
			self.iter(),
			active_context,
			type_scoped_context,
			active_property,
			loader,
			options,
			meta,
		)
	}
}
