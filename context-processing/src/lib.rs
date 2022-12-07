//! JSON-LD context processing types and algorithms.
use futures::future::{BoxFuture, FutureExt};
pub use json_ld_core::{warning, Context, ContextLoader, ProcessingMode};
use json_ld_syntax::ErrorCode;
use locspan::Meta;
use rdf_types::VocabularyMut;
use std::fmt;

mod processed;
mod stack;
pub mod syntax;

pub use processed::*;
pub use stack::ProcessingStack;

/// Warnings that can be raised during context processing.
pub enum Warning {
	KeywordLikeTerm(String),
	KeywordLikeValue(String),
	MalformedIri(String),
}

impl fmt::Display for Warning {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::KeywordLikeTerm(s) => write!(f, "keyword-like term `{}`", s),
			Self::KeywordLikeValue(s) => write!(f, "keyword-like value `{}`", s),
			Self::MalformedIri(s) => write!(f, "malformed IRI `{}`", s),
		}
	}
}

impl<N> contextual::DisplayWithContext<N> for Warning {
	fn fmt_with(&self, _: &N, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, f)
	}
}

/// Located warning.
pub type MetaWarning<M> = Meta<Warning, M>;

pub trait WarningHandler<N, M>: json_ld_core::warning::Handler<N, MetaWarning<M>> {}

impl<N, M, H> WarningHandler<N, M> for H where H: json_ld_core::warning::Handler<N, MetaWarning<M>> {}

/// Errors that can happen during context processing.
#[derive(Debug)]
pub enum Error<E> {
	InvalidContextNullification,
	LoadingDocumentFailed,
	ProcessingModeConflict,
	InvalidContextEntry,
	InvalidImportValue,
	InvalidRemoteContext,
	InvalidBaseIri,
	InvalidVocabMapping,
	CyclicIriMapping,
	InvalidTermDefinition,
	KeywordRedefinition,
	InvalidProtectedValue,
	InvalidTypeMapping,
	InvalidReverseProperty,
	InvalidIriMapping,
	InvalidKeywordAlias,
	InvalidContainerMapping,
	InvalidScopedContext,
	ProtectedTermRedefinition,
	ContextLoadingFailed(E),
}

impl<E> Error<E> {
	pub fn code(&self) -> ErrorCode {
		match self {
			Self::InvalidContextNullification => ErrorCode::InvalidContextNullification,
			Self::LoadingDocumentFailed => ErrorCode::LoadingDocumentFailed,
			Self::ProcessingModeConflict => ErrorCode::ProcessingModeConflict,
			Self::InvalidContextEntry => ErrorCode::InvalidContextEntry,
			Self::InvalidImportValue => ErrorCode::InvalidImportValue,
			Self::InvalidRemoteContext => ErrorCode::InvalidRemoteContext,
			Self::InvalidBaseIri => ErrorCode::InvalidBaseIri,
			Self::InvalidVocabMapping => ErrorCode::InvalidVocabMapping,
			Self::CyclicIriMapping => ErrorCode::CyclicIriMapping,
			Self::InvalidTermDefinition => ErrorCode::InvalidTermDefinition,
			Self::KeywordRedefinition => ErrorCode::KeywordRedefinition,
			Self::InvalidProtectedValue => ErrorCode::InvalidPropagateValue,
			Self::InvalidTypeMapping => ErrorCode::InvalidTypeMapping,
			Self::InvalidReverseProperty => ErrorCode::InvalidReverseProperty,
			Self::InvalidIriMapping => ErrorCode::InvalidIriMapping,
			Self::InvalidKeywordAlias => ErrorCode::InvalidKeywordAlias,
			Self::InvalidContainerMapping => ErrorCode::InvalidContainerMapping,
			Self::InvalidScopedContext => ErrorCode::InvalidScopedContext,
			Self::ProtectedTermRedefinition => ErrorCode::ProtectedTermRedefinition,
			Self::ContextLoadingFailed(_) => ErrorCode::LoadingRemoteContextFailed,
		}
	}
}

impl<E: fmt::Display> fmt::Display for Error<E> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::InvalidContextNullification => write!(f, "invalid context nullification"),
			Self::LoadingDocumentFailed => write!(f, "loading document failed"),
			Self::ProcessingModeConflict => write!(f, "processing mode conflict"),
			Self::InvalidContextEntry => write!(f, "invalid context entry"),
			Self::InvalidImportValue => write!(f, "invalid import value"),
			Self::InvalidRemoteContext => write!(f, "invalid remote context"),
			Self::InvalidBaseIri => write!(f, "invalid base IRI"),
			Self::InvalidVocabMapping => write!(f, "invalid vocabulary mapping"),
			Self::CyclicIriMapping => write!(f, "cyclic IRI mapping"),
			Self::InvalidTermDefinition => write!(f, "invalid term definition"),
			Self::KeywordRedefinition => write!(f, "keyword redefinition"),
			Self::InvalidProtectedValue => write!(f, "invalid protected value"),
			Self::InvalidTypeMapping => write!(f, "invalid type mapping"),
			Self::InvalidReverseProperty => write!(f, "invalid reverse property"),
			Self::InvalidIriMapping => write!(f, "invalid IRI mapping"),
			Self::InvalidKeywordAlias => write!(f, "invalid keyword alias"),
			Self::InvalidContainerMapping => write!(f, "invalid container mapping"),
			Self::InvalidScopedContext => write!(f, "invalid scoped context"),
			Self::ProtectedTermRedefinition => write!(f, "protected term redefinition"),
			Self::ContextLoadingFailed(e) => write!(f, "context loading failed: {}", e),
		}
	}
}

/// Located error.
pub type MetaError<M, E> = Meta<Error<E>, M>;

/// Result of context processing functions.
pub type ProcessingResult<'a, T, B, M, C, E> = Result<Processed<'a, T, B, C, M>, MetaError<M, E>>;

/// Context processing functions.
// FIXME: unclear why the `'static` lifetime is now required.
pub trait ProcessMeta<T, B, M>:
	json_ld_syntax::IntoJsonMeta<M> + json_ld_syntax::context::AnyValue<M>
{
	/// Process the local context with specific options.
	fn process_meta<'l: 'a, 'a, N, L: ContextLoader<T, M> + Send + Sync>(
		&'l self,
		meta: &'l M,
		vocabulary: &'a mut N,
		active_context: &'a Context<T, B, Self, M>,
		stack: ProcessingStack<T>,
		loader: &'a mut L,
		base_url: Option<T>,
		options: Options,
		warnings: impl 'a + Send + WarningHandler<N, M>,
	) -> BoxFuture<'a, ProcessingResult<'l, T, B, M, Self, L::ContextError>>
	where
		N: Send + Sync + VocabularyMut<Iri = T, BlankId = B>,
		T: Clone + PartialEq + Send + Sync,
		B: Clone + PartialEq + Send + Sync,
		M: 'a + Clone + Send + Sync,
		L::Context: Into<Self>;
}

pub trait Process<T, B, M>: Send + Sync {
	type Stripped: Send + Sync;

	/// Process the local context with specific options.
	#[allow(clippy::type_complexity)]
	fn process_full<'l: 'a, 'a, N, L: ContextLoader<T, M> + Send + Sync>(
		&'l self,
		vocabulary: &'a mut N,
		active_context: &'a Context<T, B, Self::Stripped, M>,
		loader: &'a mut L,
		base_url: Option<T>,
		options: Options,
		warnings: impl 'a + Send + WarningHandler<N, M>,
	) -> BoxFuture<'a, ProcessingResult<'l, T, B, M, Self::Stripped, L::ContextError>>
	where
		N: Send + Sync + VocabularyMut<Iri = T, BlankId = B>,
		T: Clone + PartialEq + Send + Sync,
		B: Clone + PartialEq + Send + Sync,
		M: 'a + Clone + Send + Sync,
		L::Context: Into<Self::Stripped>;

	/// Process the local context with specific options.
	#[allow(clippy::type_complexity)]
	fn process_with<'l: 'a, 'a, N, L: ContextLoader<T, M> + Send + Sync>(
		&'l self,
		vocabulary: &'a mut N,
		active_context: &'a Context<T, B, Self::Stripped, M>,
		loader: &'a mut L,
		base_url: Option<T>,
		options: Options,
	) -> BoxFuture<'a, ProcessingResult<'l, T, B, M, Self::Stripped, L::ContextError>>
	where
		N: Send + Sync + VocabularyMut<Iri = T, BlankId = B>,
		T: Clone + PartialEq + Send + Sync,
		B: Clone + PartialEq + Send + Sync,
		M: 'a + Clone + Send + Sync,
		L::Context: Into<Self::Stripped>,
	{
		self.process_full(
			vocabulary,
			active_context,
			loader,
			base_url,
			options,
			warning::Print,
		)
	}

	/// Process the local context with the given initial active context with the default options:
	/// `is_remote` is `false`, `override_protected` is `false` and `propagate` is `true`.
	#[allow(clippy::type_complexity)]
	fn process<'l: 'a, 'a, N, L: ContextLoader<T, M> + Send + Sync>(
		&'l self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		base_url: Option<T>,
	) -> BoxFuture<'a, ProcessingResult<'l, T, B, M, Self::Stripped, L::ContextError>>
	where
		N: Send + Sync + VocabularyMut<Iri = T, BlankId = B>,
		T: 'a + Clone + PartialEq + Send + Sync,
		B: 'a + Clone + PartialEq + Send + Sync,
		M: 'a + Clone + Send + Sync,
		L::Context: Into<Self::Stripped>,
	{
		async move {
			let active_context = Context::default();
			self.process_full(
				vocabulary,
				&active_context,
				loader,
				base_url,
				Options::default(),
				warning::Print,
			)
			.await
		}
		.boxed()
	}
}

impl<C: ProcessMeta<T, B, M>, T, B, M: Send + Sync> Process<T, B, M> for Meta<C, M> {
	type Stripped = C;

	/// Process the local context with specific options.
	fn process_full<'l: 'a, 'a, N, L: ContextLoader<T, M> + Send + Sync>(
		&'l self,
		vocabulary: &'a mut N,
		active_context: &'a Context<T, B, Self::Stripped, M>,
		loader: &'a mut L,
		base_url: Option<T>,
		options: Options,
		warnings: impl 'a + Send + WarningHandler<N, M>,
	) -> BoxFuture<'a, ProcessingResult<'l, T, B, M, Self::Stripped, L::ContextError>>
	where
		N: Send + Sync + VocabularyMut<Iri = T, BlankId = B>,
		T: Clone + PartialEq + Send + Sync,
		B: Clone + PartialEq + Send + Sync,
		M: 'a + Clone,
		L::Context: Into<Self::Stripped>,
	{
		self.value().process_meta(
			self.metadata(),
			vocabulary,
			active_context,
			ProcessingStack::new(),
			loader,
			base_url,
			options,
			warnings,
		)
	}
}

/// Options of the Context Processing Algorithm.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Options {
	/// The processing mode
	pub processing_mode: ProcessingMode,

	/// Override protected definitions.
	pub override_protected: bool,

	/// Propagate the processed context.
	pub propagate: bool,
}

impl Options {
	/// Return the same set of options, but with `override_protected` set to `true`.
	#[must_use]
	pub fn with_override(&self) -> Options {
		let mut opt = *self;
		opt.override_protected = true;
		opt
	}

	/// Return the same set of options, but with `override_protected` set to `false`.
	#[must_use]
	pub fn with_no_override(&self) -> Options {
		let mut opt = *self;
		opt.override_protected = false;
		opt
	}

	/// Return the same set of options, but with `propagate` set to `false`.
	#[must_use]
	pub fn without_propagation(&self) -> Options {
		let mut opt = *self;
		opt.propagate = false;
		opt
	}
}

impl Default for Options {
	fn default() -> Options {
		Options {
			processing_mode: ProcessingMode::default(),
			override_protected: false,
			propagate: true,
		}
	}
}
