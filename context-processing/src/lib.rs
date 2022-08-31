use futures::future::{BoxFuture, FutureExt};
pub use json_ld_core::{warning, Context, ContextLoader, NamespaceMut, ProcessingMode};
use locspan::Meta;
use std::fmt;

mod stack;
pub mod syntax;

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

/// Located warning.
pub type MetaWarning<M> = Meta<Warning, M>;

pub trait WarningHandler<N, M>:
	json_ld_core::warning::Handler<N, MetaWarning<M>>
{
}

impl<N, M, H> WarningHandler<N, M> for H where
	H: json_ld_core::warning::Handler<N, MetaWarning<M>>
{
}

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
pub type ProcessingResult<T, B, M, C, E> = Result<Context<T, B, C>, MetaError<M, E>>;

/// Context processing functions.
// FIXME: unclear why the `'static` lifetime is now required.
pub trait Process<T, B, M>:
	json_ld_syntax::IntoJson<M> + json_ld_syntax::context::AnyValue<M>
{
	/// Process the local context with specific options.
	fn process_full<'a, N, L: ContextLoader<T, M> + Send + Sync>(
		&'a self,
		namespace: &'a mut N,
		active_context: &'a Context<T, B, Self>,
		stack: ProcessingStack<T>,
		loader: &'a mut L,
		base_url: Option<T>,
		options: Options,
		warnings: impl 'a + Send + WarningHandler<N, M>,
	) -> BoxFuture<'a, ProcessingResult<T, B, M, Self, L::ContextError>>
	where
		N: Send + Sync + NamespaceMut<T, B>,
		T: Clone + PartialEq + Send + Sync,
		B: Clone + PartialEq + Send + Sync,
		M: 'a + Clone + Send + Sync,
		L::Context: Into<Self>;

	/// Process the local context with specific options.
	fn process_with<'a, N, L: ContextLoader<T, M> + Send + Sync>(
		&'a self,
		namespace: &'a mut N,
		active_context: &'a Context<T, B, Self>,
		loader: &'a mut L,
		base_url: Option<T>,
		options: Options,
	) -> BoxFuture<'a, ProcessingResult<T, B, M, Self, L::ContextError>>
	where
		N: Send + Sync + NamespaceMut<T, B>,
		T: Clone + PartialEq + Send + Sync,
		B: Clone + PartialEq + Send + Sync,
		M: 'a + Clone + Send + Sync,
		L::Context: Into<Self>
	{
		self.process_full(
			namespace,
			active_context,
			ProcessingStack::new(),
			loader,
			base_url,
			options,
			warning::print,
		)
	}

	/// Process the local context with the given initial active context with the default options:
	/// `is_remote` is `false`, `override_protected` is `false` and `propagate` is `true`.
	fn process<'a, N, L: ContextLoader<T, M> + Send + Sync>(
		&'a self,
		namespace: &'a mut N,
		loader: &'a mut L,
		base_url: Option<T>,
	) -> BoxFuture<'a, ProcessingResult<T, B, M, Self, L::ContextError>>
	where
		N: Send + Sync + NamespaceMut<T, B>,
		T: 'a + Clone + PartialEq + Send + Sync,
		B: 'a + Clone + PartialEq + Send + Sync,
		M: 'a + Clone + Send + Sync,
		L::Context: Into<Self>
	{
		async move {
			let active_context = Context::default();
			self.process_full(
				namespace,
				&active_context,
				ProcessingStack::new(),
				loader,
				base_url,
				Options::default(),
				warning::print,
			)
			.await
		}
		.boxed()
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
