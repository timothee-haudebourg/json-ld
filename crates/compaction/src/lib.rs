//! This library implements the [JSON-LD compaction algorithm](https://www.w3.org/TR/json-ld-api/#compaction-algorithms)
//! for the [`json-ld` crate](https://crates.io/crates/json-ld).
//!
//! # Usage
//!
//! The compaction algorithm is provided by the [`Compact`] trait.
use indexmap::IndexSet;
use json_ld_context_processing::{Options as ProcessingOptions, Process};
use json_ld_core::{
	context::inverse::{LangSelection, TypeSelection},
	object::Any,
	Context, Indexed, Loader, ProcessingMode, Term, Value,
};
use json_ld_syntax::{ContainerKind, ErrorCode, Keyword};
use json_syntax::object::Entry;
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("IRI confused with prefix")]
	IriConfusedWithPrefix,

	#[error("Invalid `@nest` value")]
	InvalidNestValue,

	#[error("Context processing failed: {0}")]
	ContextProcessing(json_ld_context_processing::Error),
}

impl Error {
	pub fn code(&self) -> ErrorCode {
		match self {
			Self::IriConfusedWithPrefix => ErrorCode::IriConfusedWithPrefix,
			Self::InvalidNestValue => ErrorCode::InvalidNestValue,
			Self::ContextProcessing(e) => e.code(),
		}
	}
}

impl From<json_ld_context_processing::Error> for Error {
	fn from(e: json_ld_context_processing::Error) -> Self {
		Self::ContextProcessing(e)
	}
}

impl From<IriConfusedWithPrefix> for Error {
	fn from(_: IriConfusedWithPrefix) -> Self {
		Self::IriConfusedWithPrefix
	}
}

pub type CompactFragmentResult = Result<json_syntax::Value, Error>;

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

pub trait CompactFragment<I, B> {
	#[allow(async_fn_in_trait)]
	async fn compact_fragment_full<'a, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		active_context: &'a Context<I, B>,
		type_scoped_context: &'a Context<I, B>,
		active_property: Option<&'a str>,
		loader: &'a mut L,
		options: Options,
	) -> CompactFragmentResult
	where
		N: VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
		L: Loader;

	#[allow(async_fn_in_trait)]
	#[inline(always)]
	async fn compact_fragment_with<'a, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		active_context: &'a Context<I, B>,
		loader: &'a mut L,
	) -> CompactFragmentResult
	where
		N: VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
		L: Loader,
	{
		self.compact_fragment_full(
			vocabulary,
			active_context,
			active_context,
			None,
			loader,
			Options::default(),
		)
		.await
	}

	#[allow(async_fn_in_trait)]
	#[inline(always)]
	async fn compact_fragment<'a, L>(
		&'a self,
		active_context: &'a Context<I, B>,
		loader: &'a mut L,
	) -> CompactFragmentResult
	where
		(): VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
		L: Loader,
	{
		self.compact_fragment_full(
			vocabulary::no_vocabulary_mut(),
			active_context,
			active_context,
			None,
			loader,
			Options::default(),
		)
		.await
	}
}

enum TypeLangValue<'a, I> {
	Type(TypeSelection<I>),
	Lang(LangSelection<'a>),
}

/// Type that can be compacted with an index.
pub trait CompactIndexedFragment<I, B> {
	#[allow(async_fn_in_trait)]
	#[allow(clippy::too_many_arguments)]
	async fn compact_indexed_fragment<'a, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		index: Option<&'a str>,
		active_context: &'a Context<I, B>,
		type_scoped_context: &'a Context<I, B>,
		active_property: Option<&'a str>,
		loader: &'a mut L,
		options: Options,
	) -> CompactFragmentResult
	where
		N: VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
		L: Loader;
}

impl<I, B, T: CompactIndexedFragment<I, B>> CompactFragment<I, B> for Indexed<T> {
	async fn compact_fragment_full<'a, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		active_context: &'a Context<I, B>,
		type_scoped_context: &'a Context<I, B>,
		active_property: Option<&'a str>,
		loader: &'a mut L,
		options: Options,
	) -> CompactFragmentResult
	where
		N: VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
		L: Loader,
	{
		self.inner()
			.compact_indexed_fragment(
				vocabulary,
				self.index(),
				active_context,
				type_scoped_context,
				active_property,
				loader,
				options,
			)
			.await
	}
}

impl<I, B, T: Any<I, B>> CompactIndexedFragment<I, B> for T {
	async fn compact_indexed_fragment<'a, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		index: Option<&'a str>,
		active_context: &'a Context<I, B>,
		type_scoped_context: &'a Context<I, B>,
		active_property: Option<&'a str>,
		loader: &'a mut L,
		options: Options,
	) -> CompactFragmentResult
	where
		N: VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
		L: Loader,
	{
		use json_ld_core::object::Ref;
		match self.as_ref() {
			Ref::Value(value) => {
				compact_indexed_value_with(
					vocabulary,
					value,
					index,
					active_context,
					active_property,
					loader,
					options,
				)
				.await
			}
			Ref::Node(node) => {
				compact_indexed_node_with(
					vocabulary,
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
			Ref::List(list) => {
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
						type_scoped_context.get(active_property)
					{
						if let Some(local_context) = active_property_definition.context() {
							active_context = Mown::Owned(
								local_context
									.process_with(
										vocabulary,
										active_context.as_ref(),
										loader,
										active_property_definition.base_url().cloned(),
										ProcessingOptions::from(options).with_override(),
									)
									.await?
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
						list.iter(),
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
								&Term::Keyword(Keyword::Index),
								true,
								false,
								options,
							)?;

							// Add an entry alias to result whose value is set to expanded value and continue with the next expanded property.
							result.insert(alias.unwrap(), json_syntax::Value::String(index.into()));
						}
					}

					Ok(json_syntax::Value::Object(result))
				}
			}
		}
	}
}

/// Default value of `as_array` is false.
fn add_value(map: &mut json_syntax::Object, key: &str, value: json_syntax::Value, as_array: bool) {
	match map
		.get_unique(key)
		.ok()
		.unwrap()
		.map(|entry| entry.is_array())
	{
		Some(false) => {
			let Entry { key, value } = map.remove_unique(key).ok().unwrap().unwrap();
			map.insert(key, json_syntax::Value::Array(vec![value]));
		}
		None if as_array => {
			map.insert(key.into(), json_syntax::Value::Array(Vec::new()));
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

			map.insert(key.into(), value);
		}
	}
}

/// Get the `@value` field of a value object.
fn value_value<I>(value: &Value<I>) -> json_syntax::Value {
	use json_ld_core::object::Literal;
	match value {
		Value::Literal(lit, _ty) => match lit {
			Literal::Null => json_syntax::Value::Null,
			Literal::Boolean(b) => json_syntax::Value::Boolean(*b),
			Literal::Number(n) => json_syntax::Value::Number(n.clone()),
			Literal::String(s) => json_syntax::Value::String(s.as_str().into()),
		},
		Value::LangString(s) => json_syntax::Value::String(s.as_str().into()),
		Value::Json(json) => json.clone(),
	}
}

async fn compact_collection_with<'a, N, L, O, T>(
	vocabulary: &'a mut N,
	items: O,
	active_context: &'a Context<N::Iri, N::BlankId>,
	type_scoped_context: &'a Context<N::Iri, N::BlankId>,
	active_property: Option<&'a str>,
	loader: &'a mut L,
	options: Options,
) -> CompactFragmentResult
where
	N: VocabularyMut,
	N::Iri: Clone + Hash + Eq,
	N::BlankId: Clone + Hash + Eq,
	T: 'a + CompactFragment<N::Iri, N::BlankId>,
	O: 'a + Iterator<Item = &'a T>,
	L: Loader,
{
	let mut result = Vec::new();

	for item in items {
		let compacted_item = Box::pin(item.compact_fragment_full(
			vocabulary,
			active_context,
			type_scoped_context,
			active_property,
			loader,
			options,
		))
		.await?;

		if !compacted_item.is_null() {
			result.push(compacted_item)
		}
	}

	let mut list_or_set = false;
	if let Some(active_property) = active_property {
		if let Some(active_property_definition) = active_context.get(active_property) {
			list_or_set = active_property_definition
				.container()
				.contains(ContainerKind::List)
				|| active_property_definition
					.container()
					.contains(ContainerKind::Set);
		}
	}

	if result.is_empty()
		|| result.len() > 1
		|| !options.compact_arrays
		|| active_property == Some("@graph")
		|| active_property == Some("@set")
		|| list_or_set
	{
		return Ok(json_syntax::Value::Array(result.into_iter().collect()));
	}

	Ok(result.into_iter().next().unwrap())
}

impl<T: CompactFragment<I, B>, I, B> CompactFragment<I, B> for IndexSet<T> {
	async fn compact_fragment_full<'a, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		active_context: &'a Context<I, B>,
		type_scoped_context: &'a Context<I, B>,
		active_property: Option<&'a str>,
		loader: &'a mut L,
		options: Options,
	) -> CompactFragmentResult
	where
		N: VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
		L: Loader,
	{
		compact_collection_with(
			vocabulary,
			self.iter(),
			active_context,
			type_scoped_context,
			active_property,
			loader,
			options,
		)
		.await
	}
}

impl<T: CompactFragment<I, B>, I, B> CompactFragment<I, B> for Vec<T> {
	async fn compact_fragment_full<'a, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		active_context: &'a Context<I, B>,
		type_scoped_context: &'a Context<I, B>,
		active_property: Option<&'a str>,
		loader: &'a mut L,
		options: Options,
	) -> CompactFragmentResult
	where
		N: VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
		L: Loader,
	{
		compact_collection_with(
			vocabulary,
			self.iter(),
			active_context,
			type_scoped_context,
			active_property,
			loader,
			options,
		)
		.await
	}
}

impl<T: CompactFragment<I, B> + Send + Sync, I, B> CompactFragment<I, B> for [T] {
	async fn compact_fragment_full<'a, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		active_context: &'a Context<I, B>,
		type_scoped_context: &'a Context<I, B>,
		active_property: Option<&'a str>,
		loader: &'a mut L,
		options: Options,
	) -> CompactFragmentResult
	where
		N: VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
		L: Loader,
	{
		compact_collection_with(
			vocabulary,
			self.iter(),
			active_context,
			type_scoped_context,
			active_property,
			loader,
			options,
		)
		.await
	}
}
