use crate::compaction::{self, Compact, CompactMeta};
use crate::context_processing::{self, Process, ProcessMeta, Processed};
use crate::expansion;
use crate::syntax::{self, ErrorCode};
use crate::{
	id::Generator, ConflictingIndexes, Context, ContextLoader, ExpandedDocument, Loader,
	ProcessingMode, RemoteDocumentReference,
};
use futures::future::BoxFuture;
use locspan::{Location, Meta};
use rdf_types::vocabulary::Index;
use rdf_types::{vocabulary, VocabularyMut};
use std::fmt::{self, Pointer};
use std::hash::Hash;

mod remote_document;

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

impl<M, E, C> ExpandError<M, E, C> {
	pub fn code(&self) -> ErrorCode {
		match self {
			Self::Expansion(e) => e.code(),
			Self::ContextProcessing(e) => e.code(),
			Self::Loading(_) => ErrorCode::LoadingDocumentFailed,
			Self::ContextLoading(_) => ErrorCode::LoadingRemoteContextFailed,
		}
	}
}

impl<M, E: fmt::Debug, C: fmt::Debug> fmt::Debug for ExpandError<M, E, C> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Expansion(e) => e.fmt(f),
			Self::ContextProcessing(e) => e.fmt(f),
			Self::Loading(e) => e.fmt(f),
			Self::ContextLoading(e) => e.fmt(f),
		}
	}
}

pub type ExpandResult<I, B, M, L> = Result<
	Meta<ExpandedDocument<I, B, M>, M>,
	ExpandError<M, <L as Loader<I, M>>::Error, <L as ContextLoader<I, M>>::ContextError>,
>;

pub enum CompactError<M, E, C> {
	Expand(ExpandError<M, E, C>),
	ContextProcessing(Meta<context_processing::Error<C>, M>),
	Compaction(Meta<compaction::Error<C>, M>),
	Loading(E),
	ContextLoading(C),
}

impl<M, E, C> CompactError<M, E, C> {
	pub fn code(&self) -> ErrorCode {
		match self {
			Self::Expand(e) => e.code(),
			Self::ContextProcessing(e) => e.code(),
			Self::Compaction(e) => e.code(),
			Self::Loading(_) => ErrorCode::LoadingDocumentFailed,
			Self::ContextLoading(_) => ErrorCode::LoadingRemoteContextFailed,
		}
	}
}

impl<M, E: fmt::Debug, C: fmt::Debug> fmt::Debug for CompactError<M, E, C> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Expand(e) => e.fmt(f),
			Self::ContextProcessing(e) => e.fmt(f),
			Self::Compaction(e) => e.fmt(f),
			Self::Loading(e) => e.fmt(f),
			Self::ContextLoading(e) => e.fmt(f),
		}
	}
}

pub type CompactResult<I, M, L> = Result<
	json_syntax::MetaValue<M>,
	CompactError<M, <L as Loader<I, M>>::Error, <L as ContextLoader<I, M>>::ContextError>,
>;

pub enum FlattenError<I, B, M, E, C> {
	Expand(ExpandError<M, E, C>),
	Compact(CompactError<M, E, C>),
	ConflictingIndexes(ConflictingIndexes<I, B, M>),
	Loading(E),
	ContextLoading(C),
}

impl<I, B, M, E, C> FlattenError<I, B, M, E, C> {
	pub fn code(&self) -> ErrorCode {
		match self {
			Self::Expand(e) => e.code(),
			Self::Compact(e) => e.code(),
			Self::ConflictingIndexes(_) => ErrorCode::ConflictingIndexes,
			Self::Loading(_) => ErrorCode::LoadingDocumentFailed,
			Self::ContextLoading(_) => ErrorCode::LoadingRemoteContextFailed,
		}
	}
}

impl<I, B, M, E: fmt::Debug, C: fmt::Debug> fmt::Debug for FlattenError<I, B, M, E, C> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Expand(e) => e.fmt(f),
			Self::Compact(e) => e.fmt(f),
			Self::ConflictingIndexes(e) => e.fmt(f),
			Self::Loading(e) => e.fmt(f),
			Self::ContextLoading(e) => e.fmt(f),
		}
	}
}

pub type FlattenResult<I, B, M, L> = Result<
	json_syntax::MetaValue<M>,
	FlattenError<I, B, M, <L as Loader<I, M>>::Error, <L as ContextLoader<I, M>>::ContextError>,
>;

pub type CompareResult<I, M, L> = Result<
	bool,
	ExpandError<M, <L as Loader<I, M>>::Error, <L as ContextLoader<I, M>>::ContextError>,
>;

pub trait JsonLdProcessor<I, M> {
	fn compare_full<'a, B, C, N, L>(
		&'a self,
		other: &'a Self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, M, C>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<CompareResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send;

	fn expand_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, M, C>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<ExpandResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
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
	) -> BoxFuture<ExpandResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
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
	) -> BoxFuture<ExpandResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		(): Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
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
	) -> BoxFuture<'a, CompactResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
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
	) -> BoxFuture<'a, CompactResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
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
	) -> BoxFuture<'a, CompactResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		(): Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		self.compact_with(vocabulary::no_vocabulary_mut(), context, loader, options)
	}

	fn flatten_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		generator: &'a mut (impl Send + Generator<I, B, N, M>),
		context: Option<RemoteDocumentReference<I, M, C>>,
		loader: &'a mut L,
		options: Options<I, M, C>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<'a, FlattenResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send;
}

async fn compact_expanded_full<'a, T, I, B, M, C, N, L>(
	expanded_input: &'a Meta<T, M>,
	url: Option<&'a I>,
	vocabulary: &'a mut N,
	context: RemoteDocumentReference<I, M, C>,
	loader: &'a mut L,
	options: Options<I, M, C>,
	warnings: impl Send + context_processing::WarningHandler<N, M>,
) -> Result<json_syntax::MetaValue<M>, CompactError<M, L::Error, L::ContextError>>
where
	T: CompactMeta<I, B, M>,
	I: Clone + Eq + Hash + Send + Sync,
	B: 'a + Clone + Eq + Hash + Send + Sync,
	C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
	N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
	M: Clone + Send + Sync,
	L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
	L::Output: Into<syntax::Value<M>>,
	L::Error: Send,
	L::Context: Into<C>,
	L::ContextError: Send,
{
	let context_base = url.or(options.base.as_ref());

	let context = context
		.load_context_with(vocabulary, loader)
		.await
		.map_err(CompactError::ContextLoading)?
		.into_document();

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

	match options.base.as_ref() {
		Some(base) => active_context.set_base_iri(Some(base.clone())),
		None => {
			if options.compact_to_relative && active_context.base_iri().is_none() {
				eprintln!("compact to relative");
				active_context.set_base_iri(url.cloned());
			}
		}
	}

	if let Some(base_iri) = active_context.base_iri() {
		eprintln!("base IRI: {}", vocabulary.iri(base_iri).unwrap());
	}

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
