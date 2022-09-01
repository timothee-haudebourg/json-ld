//! Compaction algorithm and related types.
use futures::future::{BoxFuture, FutureExt};
use json_syntax::object::Entry;
use std::hash::Hash;
use locspan::Meta;
use json_ld_context_processing::{ContextLoader, Process, Options as ProcessingOptions};
use json_ld_core::{Term, Context, Loader, NamespaceMut, ProcessingMode, Value, Indexed, context::inverse::{TypeSelection, LangSelection}, object::Any};
use json_ld_syntax::{Keyword, ContainerKind};
use mown::Mown;

mod iri;
// mod node;
mod property;
mod value;

pub(crate) use iri::*;
// use node::*;
use property::*;
use value::*;

pub type MetaError<M, E> = Meta<Error<E>, M>;

pub enum Error<E> {
	IriConfusedWithPrefix,
	ContextProcessing(json_ld_context_processing::Error<E>)
}

impl<E> From<json_ld_context_processing::Error<E>> for Error<E> {
	fn from(e: json_ld_context_processing::Error<E>) -> Self {
		Self::ContextProcessing(e)
	}
}

impl<E> From<IriConfusedWithPrefix> for Error<E> {
	fn from(_: IriConfusedWithPrefix) -> Self {
		Self::IriConfusedWithPrefix
	}
}

// fn optional_string<K: JsonBuild>(s: Option<String>, meta: K::MetaData) -> K {
// 	match s {
// 		Some(s) => K::string(s.as_str().into(), meta),
// 		None => K::null(meta),
// 	}
// }

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

impl From<Options> for json_ld_context_processing::Options {
	fn from(options: Options) -> json_ld_context_processing::Options {
		json_ld_context_processing::Options {
			processing_mode: options.processing_mode,
			..Default::default()
		}
	}
}

impl From<json_ld_expansion::Options> for Options {
	fn from(options: json_ld_expansion::Options) -> Options {
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

pub trait Compact<I, B, M> {
	fn compact_full<'a, N, C, L: Loader<I, M> + ContextLoader<I, M>>(
		&'a self,
		vocabulary: &'a mut N,
		active_context: &'a Context<I, B, C>,
		type_scoped_context: &'a Context<I, B, C>,
		active_property: Option<Meta<&'a str, &'a M>>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, MetaError<M, L::ContextError>>>
	where
		N: Send + Sync + NamespaceMut<I, B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		C: Process<I, B, M>,
		L: Send + Sync,
		L::Output: Into<Value<M, C>>,
		L::Context: Into<C>;

	// /// Compact a JSON-LD document into a `K` JSON value with the provided options.
	// ///
	// /// This calls [`compact_full`](Compact::compact_full) with `active_context`
	// /// as type scoped context.
	// #[inline(always)]
	// fn compact_with<'a, K: JsonFrom<J>, C: ContextMut<T>, L: Loader, M>(
	// 	&'a self,
	// 	active_context: &'a Inversible<T, C>,
	// 	loader: &'a mut L,
	// 	options: Options,
	// 	meta: M,
	// ) -> BoxFuture<'a, Result<K, Error>>
	// where
	// 	Self: Sync,
	// 	T: 'a + Sync + Send,
	// 	C: Sync + Send,
	// 	C::LocalContext: Send + Sync + From<L::Output>,
	// 	L: Sync + Send,
	// 	M: 'a + Send + Sync + Clone + Fn(Option<&J::MetaData>) -> K::MetaData,
	// {
	// 	async move {
	// 		self.compact_full(active_context, active_context, None, loader, options, meta)
	// 			.await
	// 	}
	// 	.boxed()
	// }

	// /// Compact a JSON-LD document into a `K` JSON value with the default options.
	// #[inline(always)]
	// fn compact<'a, K: JsonFrom<J>, C: ContextMut<T>, L: Loader, M>(
	// 	&'a self,
	// 	active_context: &'a Inversible<T, C>,
	// 	loader: &'a mut L,
	// 	meta: M,
	// ) -> BoxFuture<'a, Result<K, Error>>
	// where
	// 	Self: Sync,
	// 	T: 'a + Sync + Send,
	// 	C: Sync + Send,
	// 	C::LocalContext: Send + Sync + From<L::Output>,
	// 	L: Sync + Send,
	// 	M: 'a + Send + Sync + Clone + Fn(Option<&J::MetaData>) -> K::MetaData,
	// {
	// 	self.compact_with(active_context, loader, Options::default(), meta)
	// }
}

enum TypeLangValue<'a, I> {
	Type(TypeSelection<I>),
	Lang(LangSelection<'a>),
}

/// Type that can be compacted with an index.
pub trait CompactIndexed<I, B, M> {
	fn compact_indexed<'a, N, C, L: Loader<I, M> + ContextLoader<I, M>>(
		&'a self,
		vocabulary: &'a mut N,
		meta: &'a M,
		index: Option<&'a json_ld_syntax::Entry<String, M>>,
		active_context: &'a Context<I, B, C>,
		type_scoped_context: &'a Context<I, B, C>,
		active_property: Option<Meta<&'a str, &'a M>>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, MetaError<M, L::ContextError>>>
	where
		N: Send + Sync + NamespaceMut<I, B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		C: Process<I, B, M>,
		L: Send + Sync,
		L::Output: Into<Value<M, C>>,
		L::Context: Into<C>;
}

impl<I, B, M, T: CompactIndexed<I, B, M>> Compact<I, B, M> for Meta<Indexed<T, M>, M> {
	fn compact_full<'a, N, C, L: Loader<I, M> + ContextLoader<I, M>>(
		&'a self,
		vocabulary: &'a mut N,
		active_context: &'a Context<I, B, C>,
		type_scoped_context: &'a Context<I, B, C>,
		active_property: Option<Meta<&'a str, &'a M>>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, MetaError<M, L::ContextError>>>
	where
		N: Send + Sync + NamespaceMut<I, B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		C: Process<I, B, M>,
		L: Send + Sync,
		L::Output: Into<Value<M, C>>,
		L::Context: Into<C>
	{
		let Meta(indexed, meta) = self;
		indexed.inner().compact_indexed(
			vocabulary,
			meta,
			indexed.index_entry(),
			active_context,
			type_scoped_context,
			active_property,
			loader,
			options
		)
	}
}

impl<I, B, M, T: Any<I, B, M>> CompactIndexed<I, B, M> for T {
	fn compact_indexed<'a, N, C, L: Loader<I, M> + ContextLoader<I, M>>(
		&'a self,
		vocabulary: &'a mut N,
		meta: &'a M,
		index: Option<&'a json_ld_syntax::Entry<String, M>>,
		active_context: &'a Context<I, B, C>,
		type_scoped_context: &'a Context<I, B, C>,
		active_property: Option<Meta<&'a str, &'a M>>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, MetaError<M, L::ContextError>>>
	where
		N: Send + Sync + NamespaceMut<I, B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		C: Process<I, B, M>,
		L: Send + Sync,
		L::Output: Into<Value<M, C>>,
		L::Context: Into<C>,
	{
		use json_ld_core::object::Ref;
		match self.as_ref() {
			Ref::Value(value) => async move {
				compact_indexed_value_with(
					vocabulary,
					Meta(value, meta),
					index,
					active_context,
					active_property,
					loader,
					options
				)
				.await
			}
			.boxed(),
			Ref::Node(node) => async move {
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
			Ref::List(list) => async move {
				let mut active_context = active_context;
				// If active context has a previous context, the active context is not propagated.
				// If element does not contain an @value entry, and element does not consist of
				// a single @id entry, set active context to previous context from active context,
				// as the scope of a term-scoped context does not apply when processing new node objects.
				if let Some(previous_context) = active_context.previous_context() {
					active_context = previous_context
				}

				// If the term definition for active property in active context has a local context:
				// FIXME https://github.com/w3c/json-ld-api/issues/502
				//       Seems that the term definition should be looked up in `type_scoped_context`.
				let mut active_context = Mown::Borrowed(active_context);
				let mut list_container = false;
				if let Some(active_property) = active_property {
					if let Some(active_property_definition) =
						type_scoped_context.get(active_property.0)
					{
						if let Some(local_context) = &active_property_definition.context {
							active_context = Mown::Owned(
								local_context
									.process_with(
										vocabulary,
										active_context.as_ref(),
										loader,
										active_property_definition.base_url().cloned(),
										ProcessingOptions::from(options).with_override(),
									)
									.await.map_err(Meta::cast)?
							)
						}

						list_container = active_property_definition
							.container
							.contains(ContainerKind::List);
					}
				}

				if list_container {
					compact_collection_with(
						vocabulary,
						Meta(list.iter(), meta),
						active_context.as_ref(),
						active_context.as_ref(),
						active_property,
						loader,
						options
					)
					.await
				} else {
					let mut result = json_syntax::Object::default();
					compact_property(
						vocabulary,
						&mut result,
						Term::Keyword(Keyword::List),
						list,
						active_context.as_ref(),
						loader,
						false,
						options
					)
					.await?;

					// If expanded property is @index and active property has a container mapping in
					// active context that includes @index,
					if let Some(index) = index {
						let mut index_container = false;
						if let Some(Meta(active_property, _)) = active_property {
							if let Some(active_property_definition) =
								active_context.get(active_property)
							{
								if active_property_definition
									.container
									.contains(ContainerKind::Index)
								{
									// then the compacted result will be inside of an @index container,
									// drop the @index entry by continuing to the next expanded property.
									index_container = true;
								}
							}
						}

						if !index_container {
							// Initialize alias by IRI compacting expanded property.
							let alias = compact_key(
								vocabulary,
								active_context.as_ref(),
								Meta(&Term::Keyword(Keyword::Index), &index.key_metadata),
								true,
								false,
								options,
							).map_err(Meta::cast)?;

							// Add an entry alias to result whose value is set to expanded value and continue with the next expanded property.
							result.insert(
								alias.unwrap(),
								Meta(json_syntax::Value::String(index.value.as_str().into()), index.value.metadata().clone())
							);
						}
					}

					Ok(Meta(json_syntax::Value::Object(result), meta.clone()))
				}
			}
			.boxed(),
		}
	}
}

/// Default value of `as_array` is false.
fn add_value<M: Clone>(
	map: &mut json_syntax::Object<M>,
	Meta(key, key_metadata): Meta<&str, &M>,
	value: json_syntax::MetaValue<M>,
	as_array: bool
) {
	match map.get_unique(key).ok().unwrap().map(|entry| entry.value().is_array()) {
		Some(false) => {
			let Entry { key, value: Meta(value, meta) } = map.remove_unique(key).ok().unwrap().unwrap();
			map.insert(
				key,
				Meta(json_syntax::Value::Array(vec![Meta(value, meta.clone())]), meta)
			);
		}
		None if as_array => {
			map.insert(
				Meta(key.into(), key_metadata.clone()),
				Meta(json_syntax::Value::Array(Vec::new()), value.metadata().clone())
			);
		}
		_ => (),
	}

	match value {
		Meta(json_syntax::Value::Array(values), _) => {
			for value in values {
				add_value(map, Meta(key, key_metadata), value, false)
			}
		}
		value => {
			if let Some(mut array) = map.get_unique_mut(key).ok().unwrap() {
				array
					.as_array_mut()
					.unwrap()
					.push(value);
				return;
			}

			map.insert(
				Meta(key.into(), key_metadata.clone()),
				value
			);
		}
	}
}

/// Get the `@value` field of a value object.
fn value_value<I, M: Clone>(value: &Value<I, M>, meta: &M) -> json_syntax::MetaValue<M> {
	use json_ld_core::object::Literal;
	match value {
		Value::Literal(lit, _ty) => match lit {
			Literal::Null => Meta(json_syntax::Value::Null, meta.clone()),
			Literal::Boolean(b) => Meta(json_syntax::Value::Boolean(*b), meta.clone()),
			Literal::Number(n) => Meta(json_syntax::Value::Number(n.clone()), meta.clone()),
			Literal::String(s) => Meta(json_syntax::Value::String(s.as_str().into()), meta.clone()),
		},
		Value::LangString(s) => Meta(json_syntax::Value::String(s.as_str().into()), meta.clone()),
		Value::Json(json) => json.clone(),
	}
}

fn compact_collection_with<'a, T, O, I, B, M, C, N, L>(
	vocabulary: &mut N,
	Meta(items, meta): Meta<O, &M>,
	active_context: &Context<I, B, C>,
	type_scoped_context: &Context<I, B, C>,
	active_property: Option<Meta<&str, &M>>,
	loader: &mut L,
	options: Options
) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, MetaError<M, L::ContextError>>>
where
	T: 'a + Compact<I, B, M> + Send + Sync,
	O: Iterator<Item = &'a T> + Send,
	N: Send + Sync + NamespaceMut<I, B>,
	I: Clone + Hash + Eq + Send + Sync,
	B: Clone + Hash + Eq + Send + Sync,
	M: Clone + Send + Sync,
	C: Process<I, B, M>,
	L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
	L::Output: Into<Value<M, C>>,
	L::Context: Into<C>
{
	async move {
		let mut result = Vec::new();

		for item in items {
			let compacted_item = item
				.compact_full(
					vocabulary,
					active_context,
					type_scoped_context,
					active_property,
					loader,
					options
				)
				.await?;

			if !compacted_item.is_null() {
				result.push(compacted_item)
			}
		}

		let mut list_or_set = false;
		if let Some(Meta(active_property, _)) = active_property {
			if let Some(active_property_definition) = active_context.get(active_property) {
				list_or_set = active_property_definition
					.container
					.contains(ContainerKind::List)
					|| active_property_definition
						.container
						.contains(ContainerKind::Set);
			}
		}

		let stripped_active_property = active_property.map(Meta::into_value);

		if result.is_empty()
			|| result.len() > 1
			|| !options.compact_arrays
			|| stripped_active_property == Some("@graph")
			|| stripped_active_property == Some("@set")
			|| list_or_set
		{
			return Ok(Meta(json_syntax::Value::Array(result.into_iter().collect()), meta.clone()));
		}

		Ok(result.into_iter().next().unwrap())
	}
	.boxed()
}

// impl<J: JsonSrc, T: Sync + Send + Id> Compact<J, T> for HashSet<Indexed<Object<J, T>>> {
// 	fn compact_full<'a, K: JsonFrom<J>, C: ContextMut<T>, L: Loader, M>(
// 		&'a self,
// 		active_context: &'a Inversible<T, C>,
// 		type_scoped_context: &'a Inversible<T, C>,
// 		active_property: Option<&'a str>,
// 		loader: &'a mut L,
// 		options: Options,
// 		meta: M,
// 	) -> BoxFuture<'a, Result<K, Error>>
// 	where
// 		T: 'a,
// 		C: Sync + Send,
// 		C::LocalContext: Send + Sync + From<L::Output>,
// 		L: Sync + Send,
// 		M: 'a + Send + Sync + Clone + Fn(Option<&J::MetaData>) -> K::MetaData,
// 	{
// 		compact_collection_with(
// 			self.iter(),
// 			active_context,
// 			type_scoped_context,
// 			active_property,
// 			loader,
// 			options,
// 			meta,
// 		)
// 	}
// }

// impl<J: JsonSrc, T: Sync + Send + Id> Compact<J, T> for Vec<Indexed<Object<J, T>>> {
// 	fn compact_full<'a, K: JsonFrom<J>, C: ContextMut<T>, L: Loader, M>(
// 		&'a self,
// 		active_context: &'a Inversible<T, C>,
// 		type_scoped_context: &'a Inversible<T, C>,
// 		active_property: Option<&'a str>,
// 		loader: &'a mut L,
// 		options: Options,
// 		meta: M,
// 	) -> BoxFuture<'a, Result<K, Error>>
// 	where
// 		T: 'a,
// 		C: Sync + Send,
// 		C::LocalContext: Send + Sync + From<L::Output>,
// 		L: Sync + Send,
// 		M: 'a + Send + Sync + Clone + Fn(Option<&J::MetaData>) -> K::MetaData,
// 	{
// 		compact_collection_with(
// 			self.iter(),
// 			active_context,
// 			type_scoped_context,
// 			active_property,
// 			loader,
// 			options,
// 			meta,
// 		)
// 	}
// }

// impl<J: JsonSrc, T: Sync + Send + Id> Compact<J, T> for Vec<Indexed<Node<J, T>>> {
// 	fn compact_full<'a, K: JsonFrom<J>, C: ContextMut<T>, L: Loader, M>(
// 		&'a self,
// 		active_context: &'a Inversible<T, C>,
// 		type_scoped_context: &'a Inversible<T, C>,
// 		active_property: Option<&'a str>,
// 		loader: &'a mut L,
// 		options: Options,
// 		meta: M,
// 	) -> BoxFuture<'a, Result<K, Error>>
// 	where
// 		T: 'a,
// 		C: Sync + Send,
// 		C::LocalContext: Send + Sync + From<L::Output>,
// 		L: Sync + Send,
// 		M: 'a + Send + Sync + Clone + Fn(Option<&J::MetaData>) -> K::MetaData,
// 	{
// 		compact_collection_with(
// 			self.iter(),
// 			active_context,
// 			type_scoped_context,
// 			active_property,
// 			loader,
// 			options,
// 			meta,
// 		)
// 	}
// }
