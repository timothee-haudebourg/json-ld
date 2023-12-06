use crate::compaction::{self, Compact, CompactMeta};
use crate::context_processing::{self, Process, Processed};
use crate::expansion;
use crate::syntax::{self, ErrorCode};
use crate::{
	flattening::ConflictingIndexes, id::Generator, Context, ContextLoader, ExpandedDocument,
	Loader, ProcessingMode,
};
use json_ld_core::rdf::RdfDirection;
use json_ld_core::{
	future::{BoxFuture, FutureExt},
	Document, RdfQuads, RemoteContextReference,
};
use locspan::{Location, Meta};
use rdf_types::vocabulary::IriIndex;
use rdf_types::{vocabulary, IriVocabulary, Vocabulary, VocabularyMut};
use std::hash::Hash;

mod remote_document;

/// JSON-LD Processor options.
#[derive(Clone)]
pub struct Options<I = IriIndex, M = Location<I>> {
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
	pub expand_context: Option<RemoteContextReference<I, M>>,

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

	/// Determines how value objects containing a base direction are transformed
	/// to and from RDF.
	///
	///   - If set to [`RdfDirection::I18nDatatype`], an RDF literal is
	/// generated using a datatype IRI based on <https://www.w3.org/ns/i18n#>
	/// with both the language tag (if present) and base direction encoded. When
	/// transforming from RDF, this datatype is decoded to create a value object
	/// containing `@language` (if present) and `@direction`.
	///   - If set to [`RdfDirection::CompoundLiteral`], a blank node is emitted
	/// instead of a literal, where the blank node is the subject of
	/// `rdf:value`, `rdf:direction`, and `rdf:language` (if present)
	/// properties. When transforming from RDF, this object is decoded to create
	/// a value object containing `@language` (if present) and `@direction`.
	pub rdf_direction: Option<RdfDirection>,

	/// If set to `true`, the JSON-LD processor may emit blank nodes for triple
	/// predicates, otherwise they will be omitted.
	/// See <https://www.w3.org/TR/rdf11-concepts/>.
	///
	/// The use of blank node identifiers to label properties is obsolete, and
	/// may be removed in a future version of JSON-LD, as is the support for
	/// generalized RDF Datasets and thus this option
	/// may be also be removed.
	pub produce_generalized_rdf: bool,

	/// Term expansion policy, passed to the document expansion algorithm.
	pub expansion_policy: expansion::Policy,
}

impl<I, M> Options<I, M> {
	/// Returns these options with the `ordered` flag set to `false`.
	///
	/// This means entries will not be ordered by keys before being processed.
	pub fn unordered(self) -> Self {
		Self {
			ordered: false,
			..self
		}
	}

	/// Returns these options with the `expand_context` set to the given
	/// `context`.
	pub fn with_expand_context(self, context: RemoteContextReference<I, M>) -> Self {
		Self {
			expand_context: Some(context),
			..self
		}
	}

	/// Builds options for the context processing algorithm from these options.
	pub fn context_processing_options(&self) -> context_processing::Options {
		context_processing::Options {
			processing_mode: self.processing_mode,
			..Default::default()
		}
	}

	/// Builds options for the expansion algorithm from these options.
	pub fn expansion_options(&self) -> expansion::Options {
		expansion::Options {
			processing_mode: self.processing_mode,
			ordered: self.ordered,
			policy: self.expansion_policy,
		}
	}

	/// Builds options for the compaction algorithm from these options.
	pub fn compaction_options(&self) -> compaction::Options {
		compaction::Options {
			processing_mode: self.processing_mode,
			compact_to_relative: self.compact_to_relative,
			compact_arrays: self.compact_arrays,
			ordered: self.ordered,
		}
	}
}

impl<I, M> Default for Options<I, M> {
	fn default() -> Self {
		Self {
			base: None,
			compact_arrays: true,
			compact_to_relative: true,
			expand_context: None,
			ordered: false,
			processing_mode: ProcessingMode::JsonLd1_1,
			rdf_direction: None,
			produce_generalized_rdf: false,
			expansion_policy: expansion::Policy::default(),
		}
	}
}

/// Error that can be raised by the [`JsonLdProcessor::expand`] function.
#[derive(Debug, thiserror::Error)]
pub enum ExpandError<M, E, C> {
	/// Document expansion failed.
	#[error("Expansion failed: {0}")]
	Expansion(Meta<expansion::Error<M, C>, M>),

	/// Context processing failed.
	#[error("Context processing failed: {0}")]
	ContextProcessing(Meta<context_processing::Error<C>, M>),

	/// Remote document loading failed with the given precise error.
	#[error("Remote document loading failed: {0}")]
	Loading(E),

	/// Remote context loading failed with the given precise error.
	#[error("Remote context loading failed: {0}")]
	ContextLoading(C),
}

impl<M, E, C> ExpandError<M, E, C> {
	/// Returns the code of this error.
	pub fn code(&self) -> ErrorCode {
		match self {
			Self::Expansion(e) => e.code(),
			Self::ContextProcessing(e) => e.code(),
			Self::Loading(_) => ErrorCode::LoadingDocumentFailed,
			Self::ContextLoading(_) => ErrorCode::LoadingRemoteContextFailed,
		}
	}
}

/// Result returned by the [`JsonLdProcessor::expand`] function.
pub type ExpandResult<I, B, M, L> = Result<
	Meta<ExpandedDocument<I, B, M>, M>,
	ExpandError<M, <L as Loader<I, M>>::Error, <L as ContextLoader<I, M>>::ContextError>,
>;

/// Result returned by the [`JsonLdProcessor::into_document`] function.
pub type IntoDocumentResult<I, B, M, L> = Result<
	Document<I, B, M>,
	ExpandError<M, <L as Loader<I, M>>::Error, <L as ContextLoader<I, M>>::ContextError>,
>;

/// Error that can be raised by the [`JsonLdProcessor::compact`] function.
#[derive(Debug, thiserror::Error)]
pub enum CompactError<M, E, C> {
	/// Document expansion failed.
	#[error("Expansion failed: {0}")]
	Expand(ExpandError<M, E, C>),

	/// Context processing failed.
	#[error("Context processing failed: {0}")]
	ContextProcessing(Meta<context_processing::Error<C>, M>),

	/// Document compaction failed.
	#[error("Compaction failed: {0}")]
	Compaction(Meta<compaction::Error<C>, M>),

	/// Remote document loading failed.
	#[error("Remote document loading failed: {0}")]
	Loading(E),

	/// Remote context loading failed.
	#[error("Remote context loading failed: {0}")]
	ContextLoading(C),
}

impl<M, E, C> CompactError<M, E, C> {
	/// Returns the code of this error.
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

/// Result of the [`JsonLdProcessor::compact`] function.
pub type CompactResult<I, M, L> = Result<
	json_syntax::MetaValue<M>,
	CompactError<M, <L as Loader<I, M>>::Error, <L as ContextLoader<I, M>>::ContextError>,
>;

/// Error that can be raised by the [`JsonLdProcessor::flatten`] function.
#[derive(Debug, thiserror::Error)]
pub enum FlattenError<I, B, M, E, C> {
	#[error("Expansion failed: {0}")]
	Expand(ExpandError<M, E, C>),

	#[error("Compaction failed: {0}")]
	Compact(CompactError<M, E, C>),

	#[error("Conflicting indexes: {0}")]
	ConflictingIndexes(ConflictingIndexes<I, B, M>),

	#[error("Remote document loading failed: {0}")]
	Loading(E),

	#[error("Remote context loading failed: {0}")]
	ContextLoading(C),
}

impl<I, B, M, E, C> FlattenError<I, B, M, E, C> {
	/// Returns the code of this error.
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

/// Result of the [`JsonLdProcessor::flatten`] function.
pub type FlattenResult<I, B, M, L> = Result<
	json_syntax::MetaValue<M>,
	FlattenError<I, B, M, <L as Loader<I, M>>::Error, <L as ContextLoader<I, M>>::ContextError>,
>;

/// Error that can be raised by the [`JsonLdProcessor::to_rdf`] function.
#[derive(Debug, thiserror::Error)]
pub enum ToRdfError<M, E, C> {
	/// Document expansion failed.
	#[error("Expansion failed: {0}")]
	Expand(ExpandError<M, E, C>),
}

impl<M, E, C> ToRdfError<M, E, C> {
	/// Returns the code of this error.
	pub fn code(&self) -> ErrorCode {
		match self {
			Self::Expand(e) => e.code(),
		}
	}
}

/// Error that can be raised by the [`JsonLdProcessor::to_rdf`] function.
pub type ToRdfResult<'a, V, M, G, L> = Result<
	ToRdf<'a, 'a, V, M, G>,
	ToRdfError<
		M,
		<L as Loader<<V as IriVocabulary>::Iri, M>>::Error,
		<L as ContextLoader<<V as IriVocabulary>::Iri, M>>::ContextError,
	>,
>;

/// Result of the [`JsonLdProcessor::compare`] function.
pub type CompareResult<I, M, L> = Result<
	bool,
	ExpandError<M, <L as Loader<I, M>>::Error, <L as ContextLoader<I, M>>::ContextError>,
>;

/// Application Programming Interface.
///
/// The `JsonLdProcessor` interface is the high-level programming structure that
/// developers use to access the JSON-LD transformation methods.
///
/// It is notably implemented for the [`RemoteDocument<I, M, json_syntax::Value<M>>`](crate::RemoteDocument)
/// and [`RemoteDocumentReference<I, M, json_syntax::Value<M>>`] types.
///
/// # Methods naming
///
/// Each processing function is declined in four variants depending on your
/// needs, with the following suffix convention:
///
///   - `_full`: function with all the possible options. This is the only way
///     to specify a custom warning handler.
///   - `_with`: allows passing a custom [`Vocabulary`].
///   - `_using`: allows passing custom [`Options`].
///   - `_with_using`: allows passing both a custom [`Vocabulary`] and
///     custom [`Options`].
///   - no suffix: minimum parameters. No custom vocabulary: [`IriBuf`] and
///     [`BlankIdBuf`] must be used as IRI and blank node id respectively.
///
/// [`IriBuf`]: https://docs.rs/iref/latest/iref/struct.IriBuf.html
/// [`BlankIdBuf`]: rdf_types::BlankIdBuf
/// [`Vocabulary`]: rdf_types::Vocabulary
///
/// # Example
///
/// ```
/// use static_iref::iri;
/// use json_ld::{JsonLdProcessor, RemoteDocumentReference};
///
/// # #[async_std::main]
/// # async fn main() {
/// let input = RemoteDocumentReference::iri(iri!("https://example.com/sample.jsonld").to_owned());
///
/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
/// // the local `example` directory. No HTTP query.
/// let mut loader = json_ld::FsLoader::default();
/// loader.mount(iri!("https://example.com/").to_owned(), "examples");
///
/// let expanded = input.expand(&mut loader)
///   .await
///   .expect("expansion failed");
/// # }
/// ```
pub trait JsonLdProcessor<I, M>: Sized {
	/// Compare this document against `other` with a custom vocabulary using the
	/// given `options` and warnings handler.
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::IriVocabularyMut;
	/// use locspan::Meta;
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// let mut vocabulary: rdf_types::IndexVocabulary = rdf_types::IndexVocabulary::new();
	///
	/// let iri = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input1 = RemoteDocumentReference::iri(iri);
	/// let input2 = RemoteDocumentReference::iri(iri);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(vocabulary.insert(iri!("https://example.com/")), "examples");
	///  
	/// assert!(input1.compare_full(
	///   &input2,
	///   &mut vocabulary,
	///   &mut loader,
	///   Options::<_, _>::default(),
	///   warning::PrintWith
	/// ).await.expect("comparison failed"));
	/// # }
	/// ```
	fn compare_full<'a, B, N, L>(
		&'a self,
		other: &'a Self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, M>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<CompareResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send;

	/// Compare this document against `other` with a custom vocabulary using the
	/// given `options`.
	///
	/// Warnings are ignored.
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference};
	/// use rdf_types::IriVocabularyMut;
	/// use locspan::Meta;
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// let mut vocabulary: rdf_types::IndexVocabulary = rdf_types::IndexVocabulary::new();
	///
	/// let iri = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input1 = RemoteDocumentReference::iri(iri);
	/// let input2 = RemoteDocumentReference::iri(iri);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(vocabulary.insert(iri!("https://example.com/")), "examples");
	///  
	/// assert!(input1.compare_with_using(
	///   &input2,
	///   &mut vocabulary,
	///   &mut loader,
	///   Options::<_, _>::default()
	/// ).await.expect("comparison failed"));
	/// # }
	/// ```
	fn compare_with_using<'a, B, N, L>(
		&'a self,
		other: &'a Self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, M>,
	) -> BoxFuture<CompareResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.compare_full(other, vocabulary, loader, options, ())
	}

	/// Compare this document against `other` with a custom vocabulary.
	///
	/// Default options are used.
	/// Warnings are ignored.
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference};
	/// use rdf_types::IriVocabularyMut;
	/// use locspan::Meta;
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// let mut vocabulary: rdf_types::IndexVocabulary = rdf_types::IndexVocabulary::new();
	///
	/// let iri = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input1 = RemoteDocumentReference::iri(iri);
	/// let input2 = RemoteDocumentReference::iri(iri);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(vocabulary.insert(iri!("https://example.com/")), "examples");
	///  
	/// assert!(input1.compare_with(
	///   &input2,
	///   &mut vocabulary,
	///   &mut loader
	/// ).await.expect("comparison failed"));
	/// # }
	/// ```
	fn compare_with<'a, B, N, L>(
		&'a self,
		other: &'a Self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
	) -> BoxFuture<CompareResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: 'a + Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.compare_with_using(other, vocabulary, loader, Options::default())
	}

	/// Compare this document against `other` using the given `options`.
	///
	/// Warnings are ignored.
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference};
	/// use locspan::Meta;
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// let iri = iri!("https://example.com/sample.jsonld").to_owned();
	/// let input1 = RemoteDocumentReference::iri(iri.clone());
	/// let input2 = RemoteDocumentReference::iri(iri);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(iri!("https://example.com/").to_owned(), "examples");
	///  
	/// assert!(input1.compare_using(
	///   &input2,
	///   &mut loader,
	///   Options::<_, _>::default()
	/// ).await.expect("comparison failed"));
	/// # }
	/// ```
	fn compare_using<'a, B, L>(
		&'a self,
		other: &'a Self,
		loader: &'a mut L,
		options: Options<I, M>,
	) -> BoxFuture<CompareResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		(): Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.compare_with_using(
			other,
			rdf_types::vocabulary::no_vocabulary_mut(),
			loader,
			options,
		)
	}

	/// Compare this document against `other` with a custom vocabulary.
	///
	/// Default options are used.
	/// Warnings are ignored.
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference};
	/// use locspan::Meta;
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// let iri = iri!("https://example.com/sample.jsonld").to_owned();
	/// let input1 = RemoteDocumentReference::iri(iri.clone());
	/// let input2 = RemoteDocumentReference::iri(iri);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(iri!("https://example.com/").to_owned(), "examples");
	///  
	/// assert!(input1.compare(
	///   &input2,
	///   &mut loader
	/// ).await.expect("comparison failed"));
	/// # }
	/// ```
	fn compare<'a, B, L>(
		&'a self,
		other: &'a Self,
		loader: &'a mut L,
	) -> BoxFuture<CompareResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		(): Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: 'a + Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.compare_with(other, rdf_types::vocabulary::no_vocabulary_mut(), loader)
	}

	/// Expand the document with the given `vocabulary` and `loader`, using
	/// the given `options` and warning handler.
	///
	/// On success, the result is an [`ExpandedDocument`].
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::IriVocabularyMut;
	/// # #[async_std::main]
	/// # async fn main() {
	/// // Creates the vocabulary that will map each `rdf_types::vocabulary::Index`
	/// // to an actual `IriBuf`.
	/// let mut vocabulary: rdf_types::IndexVocabulary = rdf_types::IndexVocabulary::new();
	///
	/// let iri_index = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input = RemoteDocumentReference::iri(iri_index);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(vocabulary.insert(iri!("https://example.com/")), "examples");
	///
	/// let expanded = input
	///   .expand_full(
	///     &mut vocabulary,
	///     &mut loader,
	///     Options::<_, _>::default(),
	///     warning::PrintWith
	///   )
	///   .await
	///   .expect("expansion failed");
	/// # }
	/// ```
	fn expand_full<'a, B, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, M>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<ExpandResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send;

	fn into_document_full<'a, B, N, L>(
		self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, M>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<'a, IntoDocumentResult<I, B, M, L>>
	where
		I: 'a + Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: 'a + Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send;

	fn into_document_with_using<'a, B, N, L>(
		self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, M>,
	) -> BoxFuture<'a, IntoDocumentResult<I, B, M, L>>
	where
		I: 'a + Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: 'a + Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.into_document_full(vocabulary, loader, options, ())
	}

	fn into_document_with<'a, B, N, L>(
		self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
	) -> BoxFuture<'a, IntoDocumentResult<I, B, M, L>>
	where
		I: 'a + Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: 'a + Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.into_document_with_using(vocabulary, loader, Options::default())
	}

	fn into_document<'a, B, L>(
		self,
		loader: &'a mut L,
	) -> BoxFuture<'a, IntoDocumentResult<I, B, M, L>>
	where
		I: 'a + Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		(): VocabularyMut<Iri = I, BlankId = B>,
		M: 'a + Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.into_document_with(vocabulary::no_vocabulary_mut(), loader)
	}

	/// Expand the document with the given `vocabulary` and `loader`, using
	/// the given `options`.
	///
	/// Warnings are ignored.
	/// On success, the result is an [`ExpandedDocument`].
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::IriVocabularyMut;
	/// # #[async_std::main]
	/// # async fn main() {
	/// // Creates the vocabulary that will map each `rdf_types::vocabulary::Index`
	/// // to an actual `IriBuf`.
	/// let mut vocabulary: rdf_types::IndexVocabulary = rdf_types::IndexVocabulary::new();
	///
	/// let iri_index = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input = RemoteDocumentReference::iri(iri_index);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(vocabulary.insert(iri!("https://example.com/")), "examples");
	///
	/// let expanded = input
	///   .expand_with_using(
	///     &mut vocabulary,
	///     &mut loader,
	///     Options::<_, _>::default()
	///   )
	///   .await
	///   .expect("expansion failed");
	/// # }
	/// ```
	fn expand_with_using<'a, B, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, M>,
	) -> BoxFuture<ExpandResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.expand_full(vocabulary, loader, options, ())
	}

	/// Expand the document with the given `vocabulary` and `loader`.
	///
	/// Default options are used.
	/// Warnings are ignored.
	/// On success, the result is an [`ExpandedDocument`].
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::IriVocabularyMut;
	/// # #[async_std::main]
	/// # async fn main() {
	/// // Creates the vocabulary that will map each `rdf_types::vocabulary::Index`
	/// // to an actual `IriBuf`.
	/// let mut vocabulary: rdf_types::IndexVocabulary = rdf_types::IndexVocabulary::new();
	///
	/// let iri_index = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input = RemoteDocumentReference::iri(iri_index);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(vocabulary.insert(iri!("https://example.com/")), "examples");
	///
	/// let expanded = input
	///   .expand_with(
	///     &mut vocabulary,
	///     &mut loader
	///   )
	///   .await
	///   .expect("expansion failed");
	/// # }
	/// ```
	fn expand_with<'a, B, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
	) -> BoxFuture<ExpandResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: 'a + Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.expand_with_using(vocabulary, loader, Options::default())
	}

	/// Expand the document with the given `loader` using the given `options`.
	///
	/// Warnings are ignored.
	/// On success, the result is an [`ExpandedDocument`].
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// let iri = iri!("https://example.com/sample.jsonld").to_owned();
	/// let input = RemoteDocumentReference::iri(iri);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(iri!("https://example.com/").to_owned(), "examples");
	///
	/// let expanded = input
	///   .expand_using(
	///     &mut loader,
	///     Options::<_, _>::default()
	///   )
	///   .await
	///   .expect("expansion failed");
	/// # }
	/// ```
	fn expand_using<'a, B, L>(
		&'a self,
		loader: &'a mut L,
		options: Options<I, M>,
	) -> BoxFuture<ExpandResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		(): Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.expand_with_using(vocabulary::no_vocabulary_mut(), loader, options)
	}

	/// Expand the document with the given `loader`.
	///
	/// Default options are used.
	/// Warnings are ignored.
	/// On success, the result is an [`ExpandedDocument`].
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// let iri = iri!("https://example.com/sample.jsonld").to_owned();
	/// let input = RemoteDocumentReference::iri(iri);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(iri!("https://example.com/").to_owned(), "examples");
	///
	/// let expanded = input
	///   .expand(&mut loader)
	///   .await
	///   .expect("expansion failed");
	/// # }
	/// ```
	fn expand<'a, B, L>(&'a self, loader: &'a mut L) -> BoxFuture<ExpandResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		(): Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: 'a + Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.expand_with(vocabulary::no_vocabulary_mut(), loader)
	}

	/// Compact the document relative to `context` with the given `vocabulary`
	/// and `loader`, using the given `options` and warning handler.
	///
	/// On success, the result is an [`syntax::Value`] wrapped inside a
	/// [`Meta`].
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::IriVocabularyMut;
	/// # #[async_std::main]
	/// # async fn main() {
	/// // Creates the vocabulary that will map each `rdf_types::vocabulary::Index`
	/// // to an actual `IriBuf`.
	/// let mut vocabulary: rdf_types::IndexVocabulary = rdf_types::IndexVocabulary::new();
	///
	/// let iri_index = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input = RemoteDocumentReference::iri(iri_index);
	///
	/// let context_iri_index = vocabulary.insert(iri!("https://example.com/context.jsonld"));
	/// let context = RemoteDocumentReference::context_iri(context_iri_index);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(vocabulary.insert(iri!("https://example.com/")), "examples");
	///
	/// let compact = input
	///   .compact_full(
	///     &mut vocabulary,
	///     context,
	///     &mut loader,
	///     Options::<_, _>::default(),
	///     warning::PrintWith
	///   )
	///   .await
	///   .expect("compaction failed");
	/// # }
	/// ```
	fn compact_full<'a, B, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: RemoteContextReference<I, M>,
		loader: &'a mut L,
		options: Options<I, M>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<'a, CompactResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send;

	/// Compact the document relative to `context` with the given `vocabulary`
	/// and `loader`, using the given `options`.
	///
	/// Warnings are ignored.
	/// On success, the result is an [`syntax::Value`] wrapped inside a
	/// [`Meta`].
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::IriVocabularyMut;
	/// # #[async_std::main]
	/// # async fn main() {
	/// // Creates the vocabulary that will map each `rdf_types::vocabulary::Index`
	/// // to an actual `IriBuf`.
	/// let mut vocabulary: rdf_types::IndexVocabulary = rdf_types::IndexVocabulary::new();
	///
	/// let iri_index = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input = RemoteDocumentReference::iri(iri_index);
	///
	/// let context_iri_index = vocabulary.insert(iri!("https://example.com/context.jsonld"));
	/// let context = RemoteDocumentReference::context_iri(context_iri_index);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(vocabulary.insert(iri!("https://example.com/")), "examples");
	///
	/// let compact = input
	///   .compact_with_using(
	///     &mut vocabulary,
	///     context,
	///     &mut loader,
	///     Options::<_, _>::default()
	///   )
	///   .await
	///   .expect("compaction failed");
	/// # }
	/// ```
	fn compact_with_using<'a, B, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: RemoteContextReference<I, M>,
		loader: &'a mut L,
		options: Options<I, M>,
	) -> BoxFuture<'a, CompactResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.compact_full(vocabulary, context, loader, options, ())
	}

	/// Compact the document relative to `context` with the given `vocabulary`
	/// and `loader`.
	///
	/// Default options are used.
	/// Warnings are ignored.
	/// On success, the result is an [`syntax::Value`] wrapped inside a
	/// [`Meta`].
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::IriVocabularyMut;
	/// # #[async_std::main]
	/// # async fn main() {
	/// // Creates the vocabulary that will map each `rdf_types::vocabulary::Index`
	/// // to an actual `IriBuf`.
	/// let mut vocabulary: rdf_types::IndexVocabulary = rdf_types::IndexVocabulary::new();
	///
	/// let iri_index = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input = RemoteDocumentReference::iri(iri_index);
	///
	/// let context_iri_index = vocabulary.insert(iri!("https://example.com/context.jsonld"));
	/// let context = RemoteDocumentReference::context_iri(context_iri_index);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(vocabulary.insert(iri!("https://example.com/")), "examples");
	///
	/// let compact = input
	///   .compact_with(
	///     &mut vocabulary,
	///     context,
	///     &mut loader
	///   )
	///   .await
	///   .expect("compaction failed");
	/// # }
	/// ```
	fn compact_with<'a, B, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: RemoteContextReference<I, M>,
		loader: &'a mut L,
	) -> BoxFuture<'a, CompactResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.compact_with_using(vocabulary, context, loader, Options::default())
	}

	/// Compact the document relative to `context` with the given `loader`,
	/// using the given `options`.
	///
	/// Warnings are ignored.
	/// On success, the result is an [`syntax::Value`] wrapped inside a
	/// [`Meta`].
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// let iri = iri!("https://example.com/sample.jsonld").to_owned();
	/// let input = RemoteDocumentReference::iri(iri);
	///
	/// let context_iri = iri!("https://example.com/context.jsonld").to_owned();
	/// let context = RemoteDocumentReference::context_iri(context_iri);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(iri!("https://example.com/").to_owned(), "examples");
	///
	/// let compact = input
	///   .compact_using(
	///     context,
	///     &mut loader,
	///     Options::<_, _>::default()
	///   )
	///   .await
	///   .expect("compaction failed");
	/// # }
	/// ```
	fn compact_using<'a, B, L>(
		&'a self,
		context: RemoteContextReference<I, M>,
		loader: &'a mut L,
		options: Options<I, M>,
	) -> BoxFuture<'a, CompactResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		(): Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.compact_with_using(vocabulary::no_vocabulary_mut(), context, loader, options)
	}

	/// Compact the document relative to `context` with the given `loader`.
	///
	/// Default options are used.
	/// Warnings are ignored.
	/// On success, the result is an [`syntax::Value`] wrapped inside a
	/// [`Meta`].
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// let iri = iri!("https://example.com/sample.jsonld").to_owned();
	/// let input = RemoteDocumentReference::iri(iri);
	///
	/// let context_iri = iri!("https://example.com/context.jsonld").to_owned();
	/// let context = RemoteDocumentReference::context_iri(context_iri);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(iri!("https://example.com/").to_owned(), "examples");
	///
	/// let compact = input
	///   .compact(
	///     context,
	///     &mut loader
	///   )
	///   .await
	///   .expect("compaction failed");
	/// # }
	/// ```
	fn compact<'a, B, L>(
		&'a self,
		context: RemoteContextReference<I, M>,
		loader: &'a mut L,
	) -> BoxFuture<'a, CompactResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		(): Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.compact_with(vocabulary::no_vocabulary_mut(), context, loader)
	}

	/// Flatten the document with the given `vocabulary`, `generator`
	/// and `loader`, using the given `options` and warning handler.
	///
	/// An optional `context` can be given to compact the document.
	///
	/// Flattening requires assigning an identifier to nested anonymous nodes,
	/// which is why the flattening functions take an [`rdf_types::MetaGenerator`]
	/// as parameter. This generator is in charge of creating new fresh identifiers
	/// (with their metadata). The most common generator is
	/// [`rdf_types::generator::Blank`] that creates blank node identifiers.
	///
	/// On success, the result is a
	/// [`FlattenedDocument`](crate::FlattenedDocument), which is a list of
	/// indexed nodes.
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::IriVocabularyMut;
	/// use locspan::{Location, Span};
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// // Creates the vocabulary that will map each `rdf_types::vocabulary::Index`
	/// // to an actual `IriBuf`.
	/// let mut vocabulary: rdf_types::IndexVocabulary = rdf_types::IndexVocabulary::new();
	///
	/// let iri_index = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input = RemoteDocumentReference::iri(iri_index);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(vocabulary.insert(iri!("https://example.com/")), "examples");
	///
	/// let mut generator = rdf_types::generator::Blank::new().with_metadata(
	///   // Each blank id will be associated to the document URL with a dummy span.
	///   Location::new(vocabulary.insert(iri!("https://example.com/")), Span::default())
	/// );
	///
	/// let nodes = input
	///   .flatten_full(
	///     &mut vocabulary,
	///     &mut generator,
	///     None,
	///     &mut loader,
	///     Options::<_, _>::default(),
	///     warning::PrintWith
	///   )
	///   .await
	///   .expect("flattening failed");
	/// # }
	/// ```
	fn flatten_full<'a, B, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		generator: &'a mut (impl Send + Generator<N, M>),
		context: Option<RemoteContextReference<I, M>>,
		loader: &'a mut L,
		options: Options<I, M>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<'a, FlattenResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send;

	/// Flatten the document with the given `vocabulary`, `generator`
	/// and `loader`, using the given `options`.
	///
	/// Flattening requires assigning an identifier to nested anonymous nodes,
	/// which is why the flattening functions take an [`rdf_types::MetaGenerator`]
	/// as parameter. This generator is in charge of creating new fresh identifiers
	/// (with their metadata). The most common generator is
	/// [`rdf_types::generator::Blank`] that creates blank node identifiers.
	///
	/// Warnings are ignored.
	/// On success, the result is a
	/// [`FlattenedDocument`](crate::FlattenedDocument), which is a list of
	/// indexed nodes.
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::IriVocabularyMut;
	/// use locspan::{Location, Span};
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// // Creates the vocabulary that will map each `rdf_types::vocabulary::Index`
	/// // to an actual `IriBuf`.
	/// let mut vocabulary: rdf_types::IndexVocabulary = rdf_types::IndexVocabulary::new();
	///
	/// let iri_index = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input = RemoteDocumentReference::iri(iri_index);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(vocabulary.insert(iri!("https://example.com/")), "examples");
	///
	/// let mut generator = rdf_types::generator::Blank::new().with_metadata(
	///   // Each blank id will be associated to the document URL with a dummy span.
	///   Location::new(vocabulary.insert(iri!("https://example.com/")), Span::default())
	/// );
	///
	/// let nodes = input
	///   .flatten_with_using(
	///     &mut vocabulary,
	///     &mut generator,
	///     &mut loader,
	///     Options::<_, _>::default()
	///   )
	///   .await
	///   .expect("flattening failed");
	/// # }
	/// ```
	fn flatten_with_using<'a, B, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		generator: &'a mut (impl Send + Generator<N, M>),
		loader: &'a mut L,
		options: Options<I, M>,
	) -> BoxFuture<'a, FlattenResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.flatten_full(vocabulary, generator, None, loader, options, ())
	}

	/// Flatten the document with the given `vocabulary`, `generator`
	/// and `loader`.
	///
	/// Flattening requires assigning an identifier to nested anonymous nodes,
	/// which is why the flattening functions take an [`rdf_types::MetaGenerator`]
	/// as parameter. This generator is in charge of creating new fresh identifiers
	/// (with their metadata). The most common generator is
	/// [`rdf_types::generator::Blank`] that creates blank node identifiers.
	///
	/// Default options are used.
	/// Warnings are ignored.
	/// On success, the result is a
	/// [`FlattenedDocument`](crate::FlattenedDocument), which is a list of
	/// indexed nodes.
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::IriVocabularyMut;
	/// use locspan::{Location, Span};
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// // Creates the vocabulary that will map each `rdf_types::vocabulary::Index`
	/// // to an actual `IriBuf`.
	/// let mut vocabulary: rdf_types::IndexVocabulary = rdf_types::IndexVocabulary::new();
	///
	/// let iri_index = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input = RemoteDocumentReference::iri(iri_index);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(vocabulary.insert(iri!("https://example.com/")), "examples");
	///
	/// let mut generator = rdf_types::generator::Blank::new().with_metadata(
	///   // Each blank id will be associated to the document URL with a dummy span.
	///   Location::new(vocabulary.insert(iri!("https://example.com/")), Span::default())
	/// );
	///
	/// let nodes = input
	///   .flatten_with(
	///     &mut vocabulary,
	///     &mut generator,
	///     &mut loader
	///   )
	///   .await
	///   .expect("flattening failed");
	/// # }
	/// ```
	fn flatten_with<'a, B, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		generator: &'a mut (impl Send + Generator<N, M>),
		loader: &'a mut L,
	) -> BoxFuture<'a, FlattenResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: 'a + Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.flatten_with_using(vocabulary, generator, loader, Options::default())
	}

	/// Flatten the document with the given `generator`, `loader` and using the
	/// given `options`.
	///
	/// Flattening requires assigning an identifier to nested anonymous nodes,
	/// which is why the flattening functions take an [`rdf_types::MetaGenerator`]
	/// as parameter. This generator is in charge of creating new fresh identifiers
	/// (with their metadata). The most common generator is
	/// [`rdf_types::generator::Blank`] that creates blank node identifiers.
	///
	/// Warnings are ignored.
	/// On success, the result is a
	/// [`FlattenedDocument`](crate::FlattenedDocument), which is a list of
	/// indexed nodes.
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use locspan::{Location, Span};
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// let iri = iri!("https://example.com/sample.jsonld").to_owned();
	/// let input = RemoteDocumentReference::iri(iri);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(iri!("https://example.com/").to_owned(), "examples");
	///
	/// let mut generator = rdf_types::generator::Blank::new().with_metadata(
	///   // Each blank id will be associated to the document URL with a dummy span.
	///   Location::new(iri!("https://example.com/").to_owned(), Span::default())
	/// );
	///
	/// let nodes = input
	///   .flatten_using(
	///     &mut generator,
	///     &mut loader,
	///     Options::<_, _>::default()
	///   )
	///   .await
	///   .expect("flattening failed");
	/// # }
	/// ```
	fn flatten_using<'a, B, L>(
		&'a self,
		generator: &'a mut (impl Send + Generator<(), M>),
		loader: &'a mut L,
		options: Options<I, M>,
	) -> BoxFuture<'a, FlattenResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		(): Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.flatten_with_using(vocabulary::no_vocabulary_mut(), generator, loader, options)
	}

	/// Flatten the document with the given `generator` and `loader`.
	///
	/// Flattening requires assigning an identifier to nested anonymous nodes,
	/// which is why the flattening functions take an [`rdf_types::MetaGenerator`]
	/// as parameter. This generator is in charge of creating new fresh identifiers
	/// (with their metadata). The most common generator is
	/// [`rdf_types::generator::Blank`] that creates blank node identifiers.
	///
	/// Default options are used.
	/// Warnings are ignored.
	/// On success, the result is a
	/// [`FlattenedDocument`](crate::FlattenedDocument), which is a list of
	/// indexed nodes.
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use locspan::{Location, Span};
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// let iri = iri!("https://example.com/sample.jsonld").to_owned();
	/// let input = RemoteDocumentReference::iri(iri);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(iri!("https://example.com/").to_owned(), "examples");
	///
	/// let mut generator = rdf_types::generator::Blank::new().with_metadata(
	///   // Each blank id will be associated to the document URL with a dummy span.
	///   Location::new(iri!("https://example.com/").to_owned(), Span::default())
	/// );
	///
	/// let nodes = input
	///   .flatten(
	///     &mut generator,
	///     &mut loader
	///   )
	///   .await
	///   .expect("flattening failed");
	/// # }
	/// ```
	fn flatten<'a, B, L>(
		&'a self,
		generator: &'a mut (impl Send + Generator<(), M>),
		loader: &'a mut L,
	) -> BoxFuture<'a, FlattenResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		(): Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: 'a + Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		self.flatten_with(vocabulary::no_vocabulary_mut(), generator, loader)
	}

	/// Serializes the document into an RDF dataset with a custom vocabulary
	/// using the given `options` and warnings handler.
	///
	/// Expands the document and returns a [`ToRdf`] instance from which an
	/// iterator over the RDF quads defined by the document can be accessed
	/// using the [`ToRdf::quads`] method.
	///
	/// The quads will have type [`rdf::Quads`] which borrows the subject,
	/// predicate and graph values from the documents if possible using [`Cow`].
	/// If you prefer to have quads owning the values directly you can use the
	/// [`ToRdf::cloned_quads`] method or call the [`rdf::Quads::cloned`]
	/// method method form the value returned by [`ToRdf::quads`].
	///
	/// [`rdf::Quads`]: json_ld_core::rdf::Quads
	/// [`rdf::Quads::cloned`]: json_ld_core::rdf::Quads::cloned
	/// [`Cow`]: std::borrow::Cow
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::{IriVocabularyMut, Quad};
	/// use locspan::{Location, Span};
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// // Creates the vocabulary that will map each `rdf_types::vocabulary::Index`
	/// // to an actual `IriBuf`.
	/// let mut vocabulary: rdf_types::IndexVocabulary = rdf_types::IndexVocabulary::new();
	///
	/// let iri_index = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input = RemoteDocumentReference::iri(iri_index);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(vocabulary.insert(iri!("https://example.com/")), "examples");
	///
	/// let mut generator = rdf_types::generator::Blank::new().with_metadata(
	///   // Each blank id will be associated to the document URL with a dummy span.
	///   Location::new(vocabulary.insert(iri!("https://example.com/")), Span::default())
	/// );
	///
	/// let mut rdf = input
	///   .to_rdf_full(
	///     &mut vocabulary,
	///     &mut generator,
	///     &mut loader,
	///     Options::<_, _>::default(),
	///     warning::PrintWith
	///   )
	///   .await
	///   .expect("flattening failed");
	///
	/// for Quad(_s, _p, _o, _g) in rdf.quads() {
	///   // ...
	/// }
	/// # }
	/// ```
	fn to_rdf_full<'a, B, N, G, L>(
		&'a self,
		vocabulary: &'a mut N,
		generator: &'a mut G,
		loader: &'a mut L,
		options: Options<I, M>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<ToRdfResult<'a, N, M, G, L>>
	where
		I: 'a + Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: 'a + Clone + Send + Sync,
		G: Send + Generator<N, M>,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
		Self: Sync,
	{
		async move {
			let rdf_direction = options.rdf_direction;
			let produce_generalized_rdf = options.produce_generalized_rdf;
			let expanded_input = self
				.expand_full(&mut *vocabulary, loader, options.unordered(), warnings)
				.await
				.map_err(ToRdfError::Expand)?;
			Ok(ToRdf::new(
				vocabulary,
				generator,
				expanded_input,
				rdf_direction,
				produce_generalized_rdf,
			))
		}
		.boxed()
	}

	/// Serializes the document into an RDF dataset with a custom vocabulary
	/// using the given `options`.
	///
	/// Expands the document and returns a [`ToRdf`] instance from which an
	/// iterator over the RDF quads defined by the document can be accessed
	/// using the [`ToRdf::quads`] method.
	///
	/// The quads will have type [`rdf::Quads`] which borrows the subject,
	/// predicate and graph values from the documents if possible using [`Cow`].
	/// If you prefer to have quads owning the values directly you can use the
	/// [`ToRdf::cloned_quads`] method or call the [`rdf::Quads::cloned`]
	/// method method form the value returned by [`ToRdf::quads`].
	///
	/// [`rdf::Quads`]: json_ld_core::rdf::Quads
	/// [`rdf::Quads::cloned`]: json_ld_core::rdf::Quads::cloned
	/// [`Cow`]: std::borrow::Cow
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::{IriVocabularyMut, Quad};
	/// use locspan::{Location, Span};
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// // Creates the vocabulary that will map each `rdf_types::vocabulary::Index`
	/// // to an actual `IriBuf`.
	/// let mut vocabulary: rdf_types::IndexVocabulary = rdf_types::IndexVocabulary::new();
	///
	/// let iri_index = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input = RemoteDocumentReference::iri(iri_index);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(vocabulary.insert(iri!("https://example.com/")), "examples");
	///
	/// let mut generator = rdf_types::generator::Blank::new().with_metadata(
	///   // Each blank id will be associated to the document URL with a dummy span.
	///   Location::new(vocabulary.insert(iri!("https://example.com/")), Span::default())
	/// );
	///
	/// let mut rdf = input
	///   .to_rdf_with_using(
	///     &mut vocabulary,
	///     &mut generator,
	///     &mut loader,
	///     Options::<_, _>::default()
	///   )
	///   .await
	///   .expect("flattening failed");
	///
	/// for Quad(_s, _p, _o, _g) in rdf.quads() {
	///   // ...
	/// }
	/// # }
	/// ```
	fn to_rdf_with_using<'a, B, N, G, L>(
		&'a self,
		vocabulary: &'a mut N,
		generator: &'a mut G,
		loader: &'a mut L,
		options: Options<I, M>,
	) -> BoxFuture<ToRdfResult<'a, N, M, G, L>>
	where
		I: 'a + Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: 'a + Clone + Send + Sync,
		G: Send + Generator<N, M>,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
		Self: Sync,
	{
		self.to_rdf_full(vocabulary, generator, loader, options, ())
	}

	/// Serializes the document into an RDF dataset with a custom vocabulary.
	///
	/// Default options are used.
	///
	/// Expands the document and returns a [`ToRdf`] instance from which an
	/// iterator over the RDF quads defined by the document can be accessed
	/// using the [`ToRdf::quads`] method.
	///
	/// The quads will have type [`rdf::Quads`] which borrows the subject,
	/// predicate and graph values from the documents if possible using [`Cow`].
	/// If you prefer to have quads owning the values directly you can use the
	/// [`ToRdf::cloned_quads`] method or call the [`rdf::Quads::cloned`]
	/// method method form the value returned by [`ToRdf::quads`].
	///
	/// [`rdf::Quads`]: json_ld_core::rdf::Quads
	/// [`rdf::Quads::cloned`]: json_ld_core::rdf::Quads::cloned
	/// [`Cow`]: std::borrow::Cow
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::{IriVocabularyMut, Quad};
	/// use locspan::{Location, Span};
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// // Creates the vocabulary that will map each `rdf_types::vocabulary::Index`
	/// // to an actual `IriBuf`.
	/// let mut vocabulary: rdf_types::IndexVocabulary = rdf_types::IndexVocabulary::new();
	///
	/// let iri_index = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
	/// let input = RemoteDocumentReference::iri(iri_index);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(vocabulary.insert(iri!("https://example.com/")), "examples");
	///
	/// let mut generator = rdf_types::generator::Blank::new().with_metadata(
	///   // Each blank id will be associated to the document URL with a dummy span.
	///   Location::new(vocabulary.insert(iri!("https://example.com/")), Span::default())
	/// );
	///
	/// let mut rdf = input
	///   .to_rdf_with(
	///     &mut vocabulary,
	///     &mut generator,
	///     &mut loader
	///   )
	///   .await
	///   .expect("flattening failed");
	///
	/// for Quad(_s, _p, _o, _g) in rdf.quads() {
	///   // ...
	/// }
	/// # }
	/// ```
	fn to_rdf_with<'a, B, N, G, L>(
		&'a self,
		vocabulary: &'a mut N,
		generator: &'a mut G,
		loader: &'a mut L,
	) -> BoxFuture<ToRdfResult<'a, N, M, G, L>>
	where
		I: 'a + Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: 'a + Clone + Send + Sync,
		G: Send + Generator<N, M>,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
		Self: Sync,
	{
		self.to_rdf_full(vocabulary, generator, loader, Options::default(), ())
	}

	/// Serializes the document into an RDF dataset using the given `options`.
	///
	/// Expands the document and returns a [`ToRdf`] instance from which an
	/// iterator over the RDF quads defined by the document can be accessed
	/// using the [`ToRdf::quads`] method.
	///
	/// The quads will have type [`rdf::Quads`] which borrows the subject,
	/// predicate and graph values from the documents if possible using [`Cow`].
	/// If you prefer to have quads owning the values directly you can use the
	/// [`ToRdf::cloned_quads`] method or call the [`rdf::Quads::cloned`]
	/// method method form the value returned by [`ToRdf::quads`].
	///
	/// [`rdf::Quads`]: json_ld_core::rdf::Quads
	/// [`rdf::Quads::cloned`]: json_ld_core::rdf::Quads::cloned
	/// [`Cow`]: std::borrow::Cow
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::Quad;
	/// use locspan::{Location, Span};
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// let iri_index = iri!("https://example.com/sample.jsonld").to_owned();
	/// let input = RemoteDocumentReference::iri(iri_index);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(iri!("https://example.com/").to_owned(), "examples");
	///
	/// let mut generator = rdf_types::generator::Blank::new().with_metadata(
	///   // Each blank id will be associated to the document URL with a dummy span.
	///   Location::new(iri!("https://example.com/").to_owned(), Span::default())
	/// );
	///
	/// let mut rdf = input
	///   .to_rdf_using(
	///     &mut generator,
	///     &mut loader,
	///     Options::<_, _>::default()
	///   )
	///   .await
	///   .expect("flattening failed");
	///
	/// for Quad(s, p, o, g) in rdf.quads() {
	///   println!("subject: {}", s);
	///   println!("predicate: {}", p);
	///   println!("object: {}", o);
	///
	///   if let Some(g) = g {
	///     println!("graph: {}", g);
	///   }
	/// }
	/// # }
	/// ```
	fn to_rdf_using<'a, B, G, L>(
		&'a self,
		generator: &'a mut G,
		loader: &'a mut L,
		options: Options<I, M>,
	) -> BoxFuture<ToRdfResult<'a, (), M, G, L>>
	where
		I: 'a + Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		(): VocabularyMut<Iri = I, BlankId = B>,
		M: 'a + Clone + Send + Sync,
		G: Send + Generator<(), M>,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
		Self: Sync,
	{
		self.to_rdf_with_using(
			rdf_types::vocabulary::no_vocabulary_mut(),
			generator,
			loader,
			options,
		)
	}

	/// Serializes the document into an RDF dataset.
	///
	/// Default options are used.
	///
	/// Expands the document and returns a [`ToRdf`] instance from which an
	/// iterator over the RDF quads defined by the document can be accessed
	/// using the [`ToRdf::quads`] method.
	///
	/// The quads will have type [`rdf::Quads`] which borrows the subject,
	/// predicate and graph values from the documents if possible using [`Cow`].
	/// If you prefer to have quads owning the values directly you can use the
	/// [`ToRdf::cloned_quads`] method or call the [`rdf::Quads::cloned`]
	/// method method form the value returned by [`ToRdf::quads`].
	///
	/// [`rdf::Quads`]: json_ld_core::rdf::Quads
	/// [`rdf::Quads::cloned`]: json_ld_core::rdf::Quads::cloned
	/// [`Cow`]: std::borrow::Cow
	///
	/// # Example
	///
	/// ```
	/// use static_iref::iri;
	/// use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, warning};
	/// use rdf_types::Quad;
	/// use locspan::{Location, Span};
	///
	/// # #[async_std::main]
	/// # async fn main() {
	/// let iri_index = iri!("https://example.com/sample.jsonld").to_owned();
	/// let input = RemoteDocumentReference::iri(iri_index);
	///
	/// // Use `FsLoader` to redirect any URL starting with `https://example.com/` to
	/// // the local `example` directory. No HTTP query.
	/// let mut loader = json_ld::FsLoader::default();
	/// loader.mount(iri!("https://example.com/").to_owned(), "examples");
	///
	/// let mut generator = rdf_types::generator::Blank::new().with_metadata(
	///   // Each blank id will be associated to the document URL with a dummy span.
	///   Location::new(iri!("https://example.com/").to_owned(), Span::default())
	/// );
	///
	/// let mut rdf = input
	///   .to_rdf(
	///     &mut generator,
	///     &mut loader
	///   )
	///   .await
	///   .expect("flattening failed");
	///
	/// for Quad(s, p, o, g) in rdf.quads() {
	///   println!("subject: {}", s);
	///   println!("predicate: {}", p);
	///   println!("object: {}", o);
	///
	///   if let Some(g) = g {
	///     println!("graph: {}", g);
	///   }
	/// }
	/// # }
	/// ```
	fn to_rdf<'a, B, G, L>(
		&'a self,
		generator: &'a mut G,
		loader: &'a mut L,
	) -> BoxFuture<ToRdfResult<'a, (), M, G, L>>
	where
		I: 'a + Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		(): VocabularyMut<Iri = I, BlankId = B>,
		M: 'a + Clone + Send + Sync,
		G: Send + Generator<(), M>,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
		Self: Sync,
	{
		self.to_rdf_using(generator, loader, Options::default())
	}
}

pub struct ToRdf<'v, 'g, V: Vocabulary, M, G> {
	vocabulary: &'v mut V,
	generator: &'g mut G,
	doc: Meta<ExpandedDocument<V::Iri, V::BlankId, M>, M>,
	rdf_direction: Option<RdfDirection>,
	produce_generalized_rdf: bool,
}

impl<'v, 'g, V: Vocabulary, M, G: rdf_types::MetaGenerator<V, M>> ToRdf<'v, 'g, V, M, G> {
	fn new(
		vocabulary: &'v mut V,
		generator: &'g mut G,
		mut doc: Meta<ExpandedDocument<V::Iri, V::BlankId, M>, M>,
		rdf_direction: Option<RdfDirection>,
		produce_generalized_rdf: bool,
	) -> Self
	where
		M: Clone,
		V::Iri: Clone + Eq + Hash,
		V::BlankId: Clone + Eq + Hash,
	{
		doc.relabel_and_canonicalize_with(vocabulary, generator);
		Self {
			vocabulary,
			generator,
			doc,
			rdf_direction,
			produce_generalized_rdf,
		}
	}

	pub fn quads<'a: 'v + 'g>(&'a mut self) -> json_ld_core::rdf::Quads<'a, 'v, 'g, V, M, G> {
		self.doc.rdf_quads_full(
			self.vocabulary,
			self.generator,
			self.rdf_direction,
			self.produce_generalized_rdf,
		)
	}

	#[inline(always)]
	pub fn cloned_quads<'a: 'v + 'g>(
		&'a mut self,
	) -> json_ld_core::rdf::ClonedQuads<'a, 'v, 'g, V, M, G> {
		self.quads().cloned()
	}

	pub fn vocabulary(&self) -> &V {
		self.vocabulary
	}

	pub fn vocabulary_mut(&mut self) -> &mut V {
		self.vocabulary
	}

	pub fn into_vocabulary(self) -> &'v mut V {
		self.vocabulary
	}

	pub fn generator(&self) -> &G {
		self.generator
	}

	pub fn generator_mut(&mut self) -> &mut G {
		self.generator
	}

	pub fn into_generator(self) -> &'g mut G {
		self.generator
	}

	pub fn document(&self) -> &Meta<ExpandedDocument<V::Iri, V::BlankId, M>, M> {
		&self.doc
	}

	pub fn document_mut(&mut self) -> &mut Meta<ExpandedDocument<V::Iri, V::BlankId, M>, M> {
		&mut self.doc
	}

	pub fn into_document(self) -> Meta<ExpandedDocument<V::Iri, V::BlankId, M>, M> {
		self.doc
	}
}

async fn compact_expanded_full<'a, T, I, B, M, N, L>(
	expanded_input: &'a Meta<T, M>,
	url: Option<&'a I>,
	vocabulary: &'a mut N,
	context: RemoteContextReference<I, M>,
	loader: &'a mut L,
	options: Options<I, M>,
	warnings: impl Send + context_processing::WarningHandler<N, M>,
) -> Result<json_syntax::MetaValue<M>, CompactError<M, L::Error, L::ContextError>>
where
	T: CompactMeta<I, B, M>,
	I: Clone + Eq + Hash + Send + Sync,
	B: 'a + Clone + Eq + Hash + Send + Sync,
	N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
	M: Clone + Send + Sync,
	L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
	L::Output: Into<syntax::Value<M>>,
	L::Error: Send,
	L::ContextError: Send,
{
	let context_base = url.or(options.base.as_ref());

	let context = context
		.load_context_with(vocabulary, loader)
		.await
		.map_err(CompactError::ContextLoading)?
		.into_document();

	let mut active_context: Processed<I, B, M> = context
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
				active_context.set_base_iri(url.cloned());
			}
		}
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
