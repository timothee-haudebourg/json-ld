//! JSON-LD context processing types and algorithms.
use algorithm::{Action, RejectVocab};
pub use json_ld_core::{warning, Context, ProcessingMode};
use json_ld_core::{ExtractContextError, LoadError, Loader};
use json_ld_syntax::ErrorCode;
use rdf_types::VocabularyMut;
use std::{fmt, hash::Hash};

pub mod algorithm;
mod processed;
mod stack;

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
			Self::KeywordLikeTerm(s) => write!(f, "keyword-like term `{s}`"),
			Self::KeywordLikeValue(s) => write!(f, "keyword-like value `{s}`"),
			Self::MalformedIri(s) => write!(f, "malformed IRI `{s}`"),
		}
	}
}

impl<N> contextual::DisplayWithContext<N> for Warning {
	fn fmt_with(&self, _: &N, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, f)
	}
}

pub trait WarningHandler<N>: json_ld_core::warning::Handler<N, Warning> {}

impl<N, H> WarningHandler<N> for H where H: json_ld_core::warning::Handler<N, Warning> {}

/// Errors that can happen during context processing.
#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Invalid context nullification")]
	InvalidContextNullification,

	#[error("Remote document loading failed")]
	LoadingDocumentFailed,

	#[error("Processing mode conflict")]
	ProcessingModeConflict,

	#[error("Invalid `@context` entry")]
	InvalidContextEntry,

	#[error("Invalid `@import` value")]
	InvalidImportValue,

	#[error("Invalid remote context")]
	InvalidRemoteContext,

	#[error("Invalid base IRI")]
	InvalidBaseIri,

	#[error("Invalid vocabulary mapping")]
	InvalidVocabMapping,

	#[error("Cyclic IRI mapping")]
	CyclicIriMapping,

	#[error("Invalid term definition")]
	InvalidTermDefinition,

	#[error("Keyword redefinition")]
	KeywordRedefinition,

	#[error("Invalid `@protected` value")]
	InvalidProtectedValue,

	#[error("Invalid type mapping")]
	InvalidTypeMapping,

	#[error("Invalid reverse property")]
	InvalidReverseProperty,

	#[error("Invalid IRI mapping")]
	InvalidIriMapping,

	#[error("Invalid keyword alias")]
	InvalidKeywordAlias,

	#[error("Invalid container mapping")]
	InvalidContainerMapping,

	#[error("Invalid scoped context")]
	InvalidScopedContext,

	#[error("Protected term redefinition")]
	ProtectedTermRedefinition,

	#[error(transparent)]
	ContextLoadingFailed(#[from] LoadError),

	#[error("Unable to extract JSON-LD context: {0}")]
	ContextExtractionFailed(ExtractContextError),

	#[error("Use of forbidden `@vocab`")]
	ForbiddenVocab,
}

impl From<RejectVocab> for Error {
	fn from(_value: RejectVocab) -> Self {
		Self::ForbiddenVocab
	}
}

impl Error {
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
			Self::ContextExtractionFailed(_) => ErrorCode::LoadingRemoteContextFailed,
			Self::ForbiddenVocab => ErrorCode::InvalidVocabMapping,
		}
	}
}

/// Result of context processing functions.
pub type ProcessingResult<'a, T, B> = Result<Processed<'a, T, B>, Error>;

pub trait Process {
	/// Process the local context with specific options.
	#[allow(async_fn_in_trait)]
	async fn process_full<N, L, W>(
		&self,
		vocabulary: &mut N,
		active_context: &Context<N::Iri, N::BlankId>,
		loader: &L,
		base_url: Option<N::Iri>,
		options: Options,
		warnings: W,
	) -> Result<Processed<N::Iri, N::BlankId>, Error>
	where
		N: VocabularyMut,
		N::Iri: Clone + Eq + Hash,
		N::BlankId: Clone + PartialEq,
		L: Loader,
		W: WarningHandler<N>;

	/// Process the local context with specific options.
	#[allow(clippy::type_complexity)]
	#[allow(async_fn_in_trait)]
	async fn process_with<N, L>(
		&self,
		vocabulary: &mut N,
		active_context: &Context<N::Iri, N::BlankId>,
		loader: &L,
		base_url: Option<N::Iri>,
		options: Options,
	) -> Result<Processed<N::Iri, N::BlankId>, Error>
	where
		N: VocabularyMut,
		N::Iri: Clone + Eq + Hash,
		N::BlankId: Clone + PartialEq,
		L: Loader,
	{
		self.process_full(
			vocabulary,
			active_context,
			loader,
			base_url,
			options,
			warning::Print,
		)
		.await
	}

	/// Process the local context with the given initial active context with the default options:
	/// `is_remote` is `false`, `override_protected` is `false` and `propagate` is `true`.
	#[allow(async_fn_in_trait)]
	async fn process<N, L>(
		&self,
		vocabulary: &mut N,
		loader: &L,
		base_url: Option<N::Iri>,
	) -> Result<Processed<N::Iri, N::BlankId>, Error>
	where
		N: VocabularyMut,
		N::Iri: Clone + Eq + Hash,
		N::BlankId: Clone + PartialEq,
		L: Loader,
	{
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

	/// Forbid the use of `@vocab` to expand terms.
	pub vocab: Action,
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
			vocab: Action::Keep,
		}
	}
}
