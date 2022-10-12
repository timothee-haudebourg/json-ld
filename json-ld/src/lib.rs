//! This crate is a Rust implementation of the
//! [JSON-LD](https://www.w3.org/TR/json-ld/)
//! data interchange format.
//!
//! [Linked Data (LD)](https://www.w3.org/standards/semanticweb/data)
//! is a [World Wide Web Consortium (W3C)](https://www.w3.org/)
//! initiative built upon standard Web technologies to create an
//! interrelated network of datasets across the Web.
//! The [JavaScript Object Notation (JSON)](https://tools.ietf.org/html/rfc7159) is
//! a widely used, simple, unstructured data serialization format to describe
//! data objects in a human readable way.
//! JSON-LD brings these two technologies together, adding semantics to JSON
//! to create a lightweight data serialization format that can organize data and
//! help Web applications to inter-operate at a large scale.
//!
//! This crate aims to provide a set of types to build and process expanded
//! JSON-LD documents.
//! It can expand, compact and flatten JSON-LD documents backed by various
//! JSON implementations thanks to the [`generic-json`] crate.
use context_processing::{ProcessMeta, Processed};
use expansion::ExpansionResult;
pub use json_ld_compaction as compaction;
pub use json_ld_context_processing as context_processing;
pub use json_ld_core::*;
pub use json_ld_expansion as expansion;
pub use json_ld_syntax as syntax;

pub use compaction::Compact;
pub use context_processing::Process;
pub use expansion::Expand;

use futures::{future::BoxFuture, FutureExt};
use locspan::Meta;
use rdf_types::{vocabulary, VocabularyMut};
use std::hash::Hash;

#[derive(Clone)]
pub struct Options<I, C, M> {
	/// The base IRI to use when expanding or compacting the document.
	///
	/// If set, this overrides the input document's IRI.
	base: Option<I>,

	/// If set to true, the JSON-LD processor replaces arrays with just one element with that element during compaction.
	///
	/// If set to false, all arrays will remain arrays even if they have just one element.
	///
	/// Defaults to `true`.
	compact_arrays: bool,

	/// Determines if IRIs are compacted relative to the base option or document
	/// location when compacting.
	///
	/// Defaults to `true`.
	compact_to_relative: bool,

	/// A context that is used to initialize the active context when expanding a document.
	expand_context: Option<Meta<C, M>>,

	/// If set to `true`, certain algorithm processing steps where indicated are
	/// ordered lexicographically.
	///
	/// If `false`, order is not considered in processing.
	///
	/// Defaults to `false`.
	ordered: bool,

	/// Sets the processing mode.
	///
	/// Defaults to `ProcessingMode::JsonLd1_1`.
	processing_mode: ProcessingMode,
}

impl<I, C, M> Options<I, C, M> {
	pub fn unordered(self) -> Self {
		Self {
			ordered: false,
			..self
		}
	}

	pub fn context_processing_options(&self) -> context_processing::Options {
		context_processing::Options {
			processing_mode: self.processing_mode,
			..Default::default()
		}
	}

	pub fn expansion_options(&self) -> expansion::Options {
		expansion::Options {
			processing_mode: self.processing_mode,
			ordered: self.ordered,
			..Default::default()
		}
	}

	pub fn compaction_options(&self) -> compaction::Options {
		compaction::Options {
			processing_mode: self.processing_mode,
			compact_to_relative: self.compact_to_relative,
			compact_arrays: self.compact_arrays,
			ordered: self.ordered,
		}
	}
}

impl<I, C, M> Default for Options<I, C, M> {
	fn default() -> Self {
		Self {
			base: None,
			compact_arrays: true,
			compact_to_relative: true,
			expand_context: None,
			ordered: false,
			processing_mode: ProcessingMode::JsonLd1_1,
		}
	}
}

pub enum CompactError<M, E> {
	Expansion(expansion::Error<M, E>),
	ContextProcessing(context_processing::Error<E>),
	Compaction(compaction::Error<E>),
}

pub type MetaCompactError<M, E> = Meta<CompactError<M, E>, M>;

impl<M, E> From<expansion::Error<M, E>> for CompactError<M, E> {
	fn from(e: expansion::Error<M, E>) -> Self {
		Self::Expansion(e)
	}
}

impl<M, E> From<context_processing::Error<E>> for CompactError<M, E> {
	fn from(e: context_processing::Error<E>) -> Self {
		Self::ContextProcessing(e)
	}
}

impl<M, E> From<compaction::Error<E>> for CompactError<M, E> {
	fn from(e: compaction::Error<E>) -> Self {
		Self::Compaction(e)
	}
}

pub trait JsonLdProcessor<I, M> {
	fn expand_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, C, M>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<ExpansionResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<I, B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Context: Into<C>,
		L::ContextError: Send;

	fn expand_with<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, C, M>,
	) -> BoxFuture<ExpansionResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<I, B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		self.expand_full(vocabulary, loader, options, ())
	}

	fn expand<'a, B, C, L>(
		&'a self,
		loader: &'a mut L,
		options: Options<I, C, M>,
	) -> BoxFuture<ExpansionResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		(): Send + Sync + VocabularyMut<I, B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		self.expand_with(vocabulary::no_vocabulary_mut(), loader, options)
	}

	fn compact_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: Meta<C, M>,
		loader: &'a mut L,
		options: Options<I, C, M>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, MetaCompactError<M, L::ContextError>>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<I, B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Context: Into<C>,
		L::ContextError: Send;

	fn compact_with<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: Meta<C, M>,
		loader: &'a mut L,
		options: Options<I, C, M>,
	) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, MetaCompactError<M, L::ContextError>>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<I, B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		self.compact_full(vocabulary, context, loader, options, ())
	}

	fn compact<'a, B, C, L>(
		&'a self,
		context: Meta<C, M>,
		loader: &'a mut L,
		options: Options<I, C, M>,
	) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, MetaCompactError<M, L::ContextError>>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		(): Send + Sync + VocabularyMut<I, B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		self.compact_with(vocabulary::no_vocabulary_mut(), context, loader, options)
	}
}

impl<I, M> JsonLdProcessor<I, M> for RemoteDocument<I, json_syntax::Value<M>, M> {
	fn expand_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		mut options: Options<I, C, M>,
		mut warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<ExpansionResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<I, B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		async move {
			let active_context = Context::new(options.base.clone().or_else(|| self.url().cloned()));

			let active_context = match options.expand_context.take() {
				Some(expand_context) => expand_context
					.process_full(
						vocabulary,
						&active_context,
						loader,
						active_context.original_base_url().cloned(),
						options.context_processing_options(),
						&mut warnings,
					)
					.await
					.map_err(Meta::cast)?
					.into_processed(),
				None => active_context,
			};

			// TODO remote contextUrl

			self.document()
				.expand_full(
					vocabulary,
					active_context,
					self.url().or_else(|| options.base.as_ref()),
					loader,
					options.expansion_options(),
					warnings,
				)
				.await
		}
		.boxed()
	}

	fn compact_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: Meta<C, M>,
		loader: &'a mut L,
		options: Options<I, C, M>,
		mut warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, MetaCompactError<M, L::ContextError>>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<I, B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		async move {
			let expanded_input = JsonLdProcessor::expand_full(
				self,
				vocabulary,
				loader,
				options.clone().unordered(),
				&mut warnings,
			)
			.await
			.map_err(Meta::cast)?;

			let context_base = self.url().or_else(|| options.base.as_ref());

			let mut active_context: Processed<I, B, C, M> = context
				.process_full(
					vocabulary,
					&Context::new(None),
					loader,
					context_base.cloned(),
					options.context_processing_options(),
					warnings,
				)
				.await
				.map_err(Meta::cast)?;

			let base = match options.base.as_ref() {
				Some(base) => Some(base),
				None => {
					if options.compact_to_relative {
						self.url()
					} else {
						None
					}
				}
			};
			active_context.set_base_iri(base.cloned());

			expanded_input
				.compact_full(
					vocabulary,
					active_context.as_ref(),
					loader,
					options.compaction_options(),
				)
				.await
				.map_err(Meta::cast)
		}
		.boxed()
	}
}
