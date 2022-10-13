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
pub use json_ld_compaction as compaction;
pub use json_ld_context_processing as context_processing;
pub use json_ld_core::*;
pub use json_ld_expansion as expansion;
pub use json_ld_syntax as syntax;

pub use compaction::Compact;
pub use context_processing::Process;
pub use expansion::Expand;

use futures::{future::BoxFuture, FutureExt};
use locspan::{Meta, Location};
use rdf_types::vocabulary::Index;
use rdf_types::{vocabulary, VocabularyMut};
use std::hash::Hash;
use std::fmt::{self, Pointer};

#[derive(Clone)]
pub struct Options<I = Index, M = Location<I>, C = json_ld_syntax::context::Value<M>> {
	/// The base IRI to use when expanding or compacting the document.
	///
	/// If set, this overrides the input document's IRI.
	pub base: Option<I>,

	/// If set to true, the JSON-LD processor replaces arrays with just one element with that element during compaction.
	///
	/// If set to false, all arrays will remain arrays even if they have just one element.
	///
	/// Defaults to `true`.
	pub compact_arrays: bool,

	/// Determines if IRIs are compacted relative to the base option or document
	/// location when compacting.
	///
	/// Defaults to `true`.
	pub compact_to_relative: bool,

	/// A context that is used to initialize the active context when expanding a document.
	pub expand_context: Option<RemoteDocumentReference<I, M, C>>,

	/// If set to `true`, certain algorithm processing steps where indicated are
	/// ordered lexicographically.
	///
	/// If `false`, order is not considered in processing.
	///
	/// Defaults to `false`.
	pub ordered: bool,

	/// Sets the processing mode.
	///
	/// Defaults to `ProcessingMode::JsonLd1_1`.
	pub processing_mode: ProcessingMode,
}

impl<I, M, C> Options<I, M, C> {
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

impl<I, M, C> Default for Options<I, M, C> {
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

pub enum ExpandError<M, E, C> {
	Expansion(Meta<expansion::Error<M, C>, M>),
	ContextProcessing(Meta<context_processing::Error<C>, M>),
	Loading(E),
	ContextLoading(C),
}

impl<M, E: fmt::Debug, C: fmt::Debug> fmt::Debug for ExpandError<M, E, C> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Expansion(e) => e.fmt(f),
			Self::ContextProcessing(e) => e.fmt(f),
			Self::Loading(e) => e.fmt(f),
			Self::ContextLoading(e) => e.fmt(f)
		}
	}
}

pub type ExpandResult<I, B, M, E, C> = Result<Meta<ExpandedDocument<I, B, M>, M>, ExpandError<M, E, C>>;

pub enum CompactError<M, E, C> {
	Expand(ExpandError<M, E, C>),
	ContextProcessing(Meta<context_processing::Error<C>, M>),
	Compaction(Meta<compaction::Error<C>, M>),
	Loading(E),
	ContextLoading(C)
}

impl<M, E: fmt::Debug, C: fmt::Debug> fmt::Debug for CompactError<M, E, C> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Expand(e) => e.fmt(f),
			Self::ContextProcessing(e) => e.fmt(f),
			Self::Compaction(e) => e.fmt(f),
			Self::Loading(e) => e.fmt(f),
			Self::ContextLoading(e) => e.fmt(f)
		}
	}
}

pub trait JsonLdProcessor<I, M> {
	fn expand_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, M, C>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<ExpandResult<I, B, M, L::Error, L::ContextError>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send;

	fn expand_with<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, M, C>,
	) -> BoxFuture<ExpandResult<I, B, M, L::Error, L::ContextError>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		self.expand_full(vocabulary, loader, options, ())
	}

	fn expand<'a, B, C, L>(
		&'a self,
		loader: &'a mut L,
		options: Options<I, M, C>,
	) -> BoxFuture<ExpandResult<I, B, M, L::Error, L::ContextError>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		(): Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		self.expand_with(vocabulary::no_vocabulary_mut(), loader, options)
	}

	fn compact_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: RemoteDocumentReference<I, M, C>,
		loader: &'a mut L,
		options: Options<I, M, C>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, CompactError<M, L::Error, L::ContextError>>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send;

	fn compact_with<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: RemoteDocumentReference<I, M, C>,
		loader: &'a mut L,
		options: Options<I, M, C>,
	) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, CompactError<M, L::Error, L::ContextError>>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		self.compact_full(vocabulary, context, loader, options, ())
	}

	fn compact<'a, B, C, L>(
		&'a self,
		context: RemoteDocumentReference<I, M, C>,
		loader: &'a mut L,
		options: Options<I, M, C>,
	) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, CompactError<M, L::Error, L::ContextError>>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		(): Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		self.compact_with(vocabulary::no_vocabulary_mut(), context, loader, options)
	}
}

impl<I, M> JsonLdProcessor<I, M> for RemoteDocument<I, M, json_syntax::Value<M>> {
	fn expand_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		mut options: Options<I, M, C>,
		mut warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<ExpandResult<I, B, M, L::Error, L::ContextError>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		async move {
			let active_context = Context::new(options.base.clone().or_else(|| self.url().cloned()));

			let active_context = match options.expand_context.take() {
				Some(expand_context) => expand_context
					.load_context_with(vocabulary, loader).await.map_err(ExpandError::ContextLoading)?.into_document()
					.process_full(
						vocabulary,
						&active_context,
						loader,
						active_context.original_base_url().cloned(),
						options.context_processing_options(),
						&mut warnings,
					)
					.await
					.map_err(ExpandError::ContextProcessing)?
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
				.await.map_err(ExpandError::Expansion)
		}
		.boxed()
	}

	fn compact_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: RemoteDocumentReference<I, M, C>,
		loader: &'a mut L,
		options: Options<I, M, C>,
		mut warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, CompactError<M, L::Error, L::ContextError>>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
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
			.map_err(CompactError::Expand)?;

			let context_base = self.url().or_else(|| options.base.as_ref());

			let context = context
			.load_context_with(vocabulary, loader).await.map_err(CompactError::ContextLoading)?.into_document();

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
				.map_err(CompactError::ContextProcessing)?;

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
				.map_err(CompactError::Compaction)
		}
		.boxed()
	}
}

impl<I, M> JsonLdProcessor<I, M> for RemoteDocumentReference<I, M, json_syntax::Value<M>> {
	fn expand_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, M, C>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<ExpandResult<I, B, M, L::Error, L::ContextError>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		async move {
			let doc = self.loaded_with(vocabulary, loader).await.map_err(ExpandError::Loading)?;
			JsonLdProcessor::expand_full(doc.as_ref(), vocabulary, loader, options, warnings).await
		}
		.boxed()
	}

	fn compact_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: RemoteDocumentReference<I, M, C>,
		loader: &'a mut L,
		options: Options<I, M, C>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, CompactError<M, L::Error, L::ContextError>>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		async move {
			let doc = self.loaded_with(vocabulary, loader).await.map_err(CompactError::Loading)?;
			JsonLdProcessor::compact_full(doc.as_ref(), vocabulary, context, loader, options, warnings).await
		}
		.boxed()
	}
}