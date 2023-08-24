//! This library implements the [JSON-LD compaction algorithm](https://www.w3.org/TR/json-ld-api/#compaction-algorithms)
//! for the [`json-ld` crate](https://crates.io/crates/json-ld).
//!
//! # Usage
//!
//! The compaction algorithm is provided by the [`Compact`] trait.
use futures::future::{BoxFuture, FutureExt};
use indexmap::IndexSet;
use json_ld_context_processing::{ContextLoader, Options as ProcessingOptions, Process};
use json_ld_core::{
	context::inverse::{LangSelection, TypeSelection},
	object::Any,
	Context, Indexed, Loader, ProcessingMode, Term, Value,
};
use json_ld_syntax::{ContainerKind, ErrorCode, Keyword};
use json_syntax::object::Entry;
use locspan::{Meta, Stripped};
use mown::Mown;
use rdf_types::{vocabulary, VocabularyMut};
use std::hash::Hash;

mod document;
mod iri;
mod node;
mod property;
mod value;

pub use document::*;
pub(crate) use iri::*;
use node::*;
use property::*;
use value::*;

pub type MetaError<M, E> = Meta<Error<E>, M>;

#[derive(Debug, thiserror::Error)]
pub enum Error<E> {
	#[error("IRI confused with prefix")]
	IriConfusedWithPrefix,

	#[error("Invalid `@nest` value")]
	InvalidNestValue,

	#[error("Context processing failed: {0}")]
	ContextProcessing(json_ld_context_processing::Error<E>),
}

impl<E> Error<E> {
	pub fn code(&self) -> ErrorCode {
		match self {
			Self::IriConfusedWithPrefix => ErrorCode::IriConfusedWithPrefix,
			Self::InvalidNestValue => ErrorCode::InvalidNestValue,
			Self::ContextProcessing(e) => e.code(),
		}
	}
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

pub type CompactFragmentResult<I, M, L> =
	Result<json_syntax::MetaValue<M>, MetaError<M, <L as ContextLoader<I, M>>::ContextError>>;

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

impl Options {
	pub fn unordered(self) -> Self {
		Self {
			ordered: false,
			..self
		}
	}
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

pub trait CompactFragmentMeta<I, B, M> {
	fn compact_fragment_full_meta<'a, N, L: Loader<I, M> + ContextLoader<I, M>>(
		&'a self,
		meta: &'a M,
		vocabulary: &'a mut N,
		active_context: &'a Context<I, B, M>,
		type_scoped_context: &'a Context<I, B, M>,
		active_property: Option<Meta<&'a str, &'a M>>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, CompactFragmentResult<I, M, L>>
	where
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		L: Send + Sync;
}

pub trait CompactFragment<I, B, M> {
	fn compact_fragment_full<'a, N, L: Loader<I, M> + ContextLoader<I, M>>(
		&'a self,
		vocabulary: &'a mut N,
		active_context: &'a Context<I, B, M>,
		type_scoped_context: &'a Context<I, B, M>,
		active_property: Option<Meta<&'a str, &'a M>>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, CompactFragmentResult<I, M, L>>
	where
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		L: Send + Sync;

	#[inline(always)]
	fn compact_fragment_with<'a, N, L: Loader<I, M> + ContextLoader<I, M>>(
		&'a self,
		vocabulary: &'a mut N,
		active_context: &'a Context<I, B, M>,
		loader: &'a mut L,
	) -> BoxFuture<'a, CompactFragmentResult<I, M, L>>
	where
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		L: Send + Sync,
	{
		self.compact_fragment_full(
			vocabulary,
			active_context,
			active_context,
			None,
			loader,
			Options::default(),
		)
	}

	#[inline(always)]
	fn compact_fragment<'a, L: Loader<I, M> + ContextLoader<I, M>>(
		&'a self,
		active_context: &'a Context<I, B, M>,
		loader: &'a mut L,
	) -> BoxFuture<'a, CompactFragmentResult<I, M, L>>
	where
		(): VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		L: Send + Sync,
	{
		self.compact_fragment_full(
			vocabulary::no_vocabulary_mut(),
			active_context,
			active_context,
			None,
			loader,
			Options::default(),
		)
	}
}

impl<T: CompactFragmentMeta<I, B, M>, I, B, M> CompactFragment<I, B, M> for Meta<T, M> {
	fn compact_fragment_full<'a, N, L: Loader<I, M> + ContextLoader<I, M>>(
		&'a self,
		vocabulary: &'a mut N,
		active_context: &'a Context<I, B, M>,
		type_scoped_context: &'a Context<I, B, M>,
		active_property: Option<Meta<&'a str, &'a M>>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, CompactFragmentResult<I, M, L>>
	where
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		L: Send + Sync,
	{
		self.0.compact_fragment_full_meta(
			&self.1,
			vocabulary,
			active_context,
			type_scoped_context,
			active_property,
			loader,
			options,
		)
	}
}

enum TypeLangValue<'a, I> {
	Type(TypeSelection<I>),
	Lang(LangSelection<'a>),
}

/// Type that can be compacted with an index.
pub trait CompactIndexedFragment<I, B, M> {
	fn compact_indexed_fragment<'a, N, L: Loader<I, M> + ContextLoader<I, M>>(
		&'a self,
		vocabulary: &'a mut N,
		meta: &'a M,
		index: Option<&'a json_ld_syntax::Entry<String, M>>,
		active_context: &'a Context<I, B, M>,
		type_scoped_context: &'a Context<I, B, M>,
		active_property: Option<Meta<&'a str, &'a M>>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, CompactFragmentResult<I, M, L>>
	where
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		L: Send + Sync;
}

impl<I, B, M, T: CompactIndexedFragment<I, B, M>> CompactFragmentMeta<I, B, M> for Indexed<T, M> {
	fn compact_fragment_full_meta<'a, N, L: Loader<I, M> + ContextLoader<I, M>>(
		&'a self,
		meta: &'a M,
		vocabulary: &'a mut N,
		active_context: &'a Context<I, B, M>,
		type_scoped_context: &'a Context<I, B, M>,
		active_property: Option<Meta<&'a str, &'a M>>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, CompactFragmentResult<I, M, L>>
	where
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		L: Send + Sync,
	{
		self.inner().compact_indexed_fragment(
			vocabulary,
			meta,
			self.index_entry(),
			active_context,
			type_scoped_context,
			active_property,
			loader,
			options,
		)
	}
}

impl<I, B, M, T: Any<I, B, M>> CompactIndexedFragment<I, B, M> for T {
	fn compact_indexed_fragment<'a, N, L: Loader<I, M> + ContextLoader<I, M>>(
		&'a self,
		vocabulary: &'a mut N,
		meta: &'a M,
		index: Option<&'a json_ld_syntax::Entry<String, M>>,
		active_context: &'a Context<I, B, M>,
		type_scoped_context: &'a Context<I, B, M>,
		active_property: Option<Meta<&'a str, &'a M>>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, CompactFragmentResult<I, M, L>>
	where
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		L: Send + Sync,
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
					options,
				)
				.await
			}
			.boxed(),
			Ref::Node(node) => async move {
				compact_indexed_node_with(
					vocabulary,
					Meta(node, meta),
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
						if let Some(local_context) = active_property_definition.context() {
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

						list_container = active_property_definition
							.container()
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
						options,
					)
					.await
				} else {
					let mut result = json_syntax::Object::default();
					compact_property(
						vocabulary,
						&mut result,
						Meta(
							Term::Keyword(Keyword::List),
							list.entry().key_metadata.clone(),
						),
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
						if let Some(Meta(active_property, _)) = active_property {
							if let Some(active_property_definition) =
								active_context.get(active_property)
							{
								if active_property_definition
									.container()
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
							)
							.map_err(Meta::cast)?;

							// Add an entry alias to result whose value is set to expanded value and continue with the next expanded property.
							result.insert(
								alias.unwrap(),
								Meta(
									json_syntax::Value::String(index.value.as_str().into()),
									index.value.metadata().clone(),
								),
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
	as_array: bool,
) {
	match map
		.get_unique(key)
		.ok()
		.unwrap()
		.map(|entry| entry.value().is_array())
	{
		Some(false) => {
			let Entry {
				key,
				value: Meta(value, meta),
			} = map.remove_unique(key).ok().unwrap().unwrap();
			map.insert(
				key,
				Meta(
					json_syntax::Value::Array(vec![Meta(value, meta.clone())]),
					meta,
				),
			);
		}
		None if as_array => {
			map.insert(
				Meta(key.into(), key_metadata.clone()),
				Meta(
					json_syntax::Value::Array(Vec::new()),
					value.metadata().clone(),
				),
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
			if let Some(array) = map.get_unique_mut(key).ok().unwrap() {
				array.as_array_mut().unwrap().push(value);
				return;
			}

			map.insert(Meta(key.into(), key_metadata.clone()), value);
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

fn compact_collection_with<'a, T, O, I, B, M, N, L>(
	vocabulary: &'a mut N,
	Meta(items, meta): Meta<O, &'a M>,
	active_context: &'a Context<I, B, M>,
	type_scoped_context: &'a Context<I, B, M>,
	active_property: Option<Meta<&'a str, &'a M>>,
	loader: &'a mut L,
	options: Options,
) -> BoxFuture<'a, CompactFragmentResult<I, M, L>>
where
	T: 'a + CompactFragment<I, B, M> + Send + Sync,
	O: 'a + Iterator<Item = &'a T> + Send,
	N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
	I: Clone + Hash + Eq + Send + Sync,
	B: Clone + Hash + Eq + Send + Sync,
	M: Clone + Send + Sync,
	L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
{
	async move {
		let mut result = Vec::new();

		for item in items {
			let compacted_item = item
				.compact_fragment_full(
					vocabulary,
					active_context,
					type_scoped_context,
					active_property,
					loader,
					options,
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
					.container()
					.contains(ContainerKind::List)
					|| active_property_definition
						.container()
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
			return Ok(Meta(
				json_syntax::Value::Array(result.into_iter().collect()),
				meta.clone(),
			));
		}

		Ok(result.into_iter().next().unwrap())
	}
	.boxed()
}

impl<T: CompactFragment<I, B, M> + Send + Sync, I, B, M> CompactFragmentMeta<I, B, M>
	for IndexSet<T>
{
	fn compact_fragment_full_meta<'a, N, L: Loader<I, M> + ContextLoader<I, M>>(
		&'a self,
		meta: &'a M,
		vocabulary: &'a mut N,
		active_context: &'a Context<I, B, M>,
		type_scoped_context: &'a Context<I, B, M>,
		active_property: Option<Meta<&'a str, &'a M>>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, CompactFragmentResult<I, M, L>>
	where
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		L: Send + Sync,
	{
		compact_collection_with(
			vocabulary,
			Meta(self.iter(), meta),
			active_context,
			type_scoped_context,
			active_property,
			loader,
			options,
		)
	}
}

impl<T: CompactFragment<I, B, M> + Send + Sync, I, B, M> CompactFragmentMeta<I, B, M> for Vec<T> {
	fn compact_fragment_full_meta<'a, N, L: Loader<I, M> + ContextLoader<I, M>>(
		&'a self,
		meta: &'a M,
		vocabulary: &'a mut N,
		active_context: &'a Context<I, B, M>,
		type_scoped_context: &'a Context<I, B, M>,
		active_property: Option<Meta<&'a str, &'a M>>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, CompactFragmentResult<I, M, L>>
	where
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		L: Send + Sync,
	{
		compact_collection_with(
			vocabulary,
			Meta(self.iter(), meta),
			active_context,
			type_scoped_context,
			active_property,
			loader,
			options,
		)
	}
}

impl<T: CompactFragment<I, B, M>, I, B, M> CompactFragment<I, B, M> for Stripped<T> {
	fn compact_fragment_full<'a, N, L: Loader<I, M> + ContextLoader<I, M>>(
		&'a self,
		vocabulary: &'a mut N,
		active_context: &'a Context<I, B, M>,
		type_scoped_context: &'a Context<I, B, M>,
		active_property: Option<Meta<&'a str, &'a M>>,
		loader: &'a mut L,
		options: Options,
	) -> BoxFuture<'a, CompactFragmentResult<I, M, L>>
	where
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		L: Send + Sync,
	{
		self.0.compact_fragment_full(
			vocabulary,
			active_context,
			type_scoped_context,
			active_property,
			loader,
			options,
		)
	}
}
