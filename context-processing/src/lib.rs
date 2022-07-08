pub use json_ld_core::{
	ProcessingMode,
	Context
};
use locspan::Meta;
use iref::Iri;
use futures::future::{BoxFuture, FutureExt};

mod stack;
// mod json;
pub mod syntax;

pub use stack::ProcessingStack;

pub trait ContextLoader {
	/// Output of the loader.
	type Output;
	type Source;

	fn load_context<'a>(
		&'a mut self,
		url: Iri,
	) -> BoxFuture<'a, Result<Self::Output, Error>>;
}

/// Warnings that can be raised during context processing.
pub enum Warning {
	KeywordLikeTerm(String),
	KeywordLikeValue(String),
	MalformedIri(String)
}

/// Located warning.
pub type MetaWarning<C> = Meta<Warning, <C as json_ld_syntax::AnyContextEntry>::Metadata>;

/// Errors that can happen during context processing.
pub enum Error {
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
	ProtectedTermRedefinition
}

/// Located error.
pub type MetaError<C> = Meta<Error, <C as json_ld_syntax::AnyContextEntry>::Metadata>;

pub fn ignore_warnings<M>(_warning: Meta<Warning, M>) {}

/// Result of context processing functions.
pub type ProcessingResult<T, C> = Result<Context<T, C>, MetaError<C>>;

/// Context processing functions.
pub trait Process<T>: json_ld_syntax::AnyContextEntry {
	/// Process the local context with specific options.
	fn process_full<'a, L: ContextLoader + Send + Sync>(
		&'a self,
		active_context: &'a Context<T, Self>,
		stack: ProcessingStack,
		loader: &'a mut L,
		base_url: Option<Iri<'a>>,
		options: ProcessingOptions,
		warnings: impl 'a + Send + FnMut(MetaWarning<Self>)
	) -> BoxFuture<'a, ProcessingResult<T, Self>>
	where
		L::Output: Into<Self>,
		T: Send + Sync;

	/// Process the local context with specific options.
	fn process_with<'a, L: ContextLoader + Send + Sync>(
		&'a self,
		active_context: &'a Context<T, Self>,
		loader: &'a mut L,
		base_url: Option<Iri<'a>>,
		options: ProcessingOptions,
	) -> BoxFuture<'a, ProcessingResult<T, Self>>
	where
		L::Output: Into<Self>,
		T: Send + Sync
	{
		self.process_full(
			active_context,
			ProcessingStack::new(),
			loader,
			base_url,
			options,
			ignore_warnings
		)
	}

	/// Process the local context with the given initial active context with the default options:
	/// `is_remote` is `false`, `override_protected` is `false` and `propagate` is `true`.
	fn process<'a, L: ContextLoader + Send + Sync>(
		&'a self,
		loader: &'a mut L,
		base_url: Option<Iri<'a>>,
	) -> BoxFuture<'a, ProcessingResult<T, Self>>
	where
		L::Output: Into<Self>,
		T: Send + Sync
	{
		async move {
			let active_context = Context::default();
			self.process_full(
				&active_context,
				ProcessingStack::new(),
				loader,
				base_url,
				ProcessingOptions::default(),
				ignore_warnings
			)
			.await
		}
		.boxed()
	}
}

/// Options of the Context Processing Algorithm.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ProcessingOptions {
	/// The processing mode
	pub processing_mode: ProcessingMode,

	/// Override protected definitions.
	pub override_protected: bool,

	/// Propagate the processed context.
	pub propagate: bool,
}

impl ProcessingOptions {
	/// Return the same set of options, but with `override_protected` set to `true`.
	#[must_use]
	pub fn with_override(&self) -> ProcessingOptions {
		let mut opt = *self;
		opt.override_protected = true;
		opt
	}

	/// Return the same set of options, but with `override_protected` set to `false`.
	#[must_use]
	pub fn with_no_override(&self) -> ProcessingOptions {
		let mut opt = *self;
		opt.override_protected = false;
		opt
	}

	/// Return the same set of options, but with `propagate` set to `false`.
	#[must_use]
	pub fn without_propagation(&self) -> ProcessingOptions {
		let mut opt = *self;
		opt.propagate = false;
		opt
	}
}

impl Default for ProcessingOptions {
	fn default() -> ProcessingOptions {
		ProcessingOptions {
			processing_mode: ProcessingMode::default(),
			override_protected: false,
			propagate: true,
		}
	}
}