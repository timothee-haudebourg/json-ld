pub use json_ld_core::{
	ProcessingMode,
	Context
};
use locspan::{Loc, Location};
use iref::Iri;
use futures::future::{BoxFuture, FutureExt};

mod stack;
// mod json;
mod syntax;

pub use stack::ProcessingStack;

pub trait Loader {
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
	KeywordLikeValue(String)
}

/// Located warning.
pub type LocWarning<T, C> = Loc<Warning, <C as Process<T>>::Source, <C as Process<T>>::Span>;

impl Warning {
	pub fn located<S, M>(self, loc: Location<S, M>) -> Loc<Warning, S, M> {
		Loc::new(self, loc)
	}
}

/// Errors that can happen during context processing.
pub enum Error {
	InvalidContextNullification,
	LoadingDocumentFailed,
	ProcessingModeConflict,
	InvalidContextEntry,
	InvalidImportValue,
	InvalidRemoteContext,
	InvalidBaseIri,
	InvalidVocabMapping
}

impl Error {
	pub fn located<S, M>(self, loc: Location<S, M>) -> Loc<Error, S, M> {
		Loc::new(self, loc)
	}
}

/// Located error.
pub type LocError<T, C> = Loc<Error, <C as Process<T>>::Source, <C as Process<T>>::Span>;

pub struct ProcessedContext<T, C: Process<T>> {
	context: Context<T, C>,
	warnings: Vec<Loc<Warning, C::Source, C::Span>>
}

impl<T, C: Process<T>> ProcessedContext<T, C> {
	pub fn new(context: Context<T, C>) -> Self {
		Self {
			context,
			warnings: Vec::new()
		}
	}

	pub fn with_warnings(context: Context<T, C>, warnings: Vec<Loc<Warning, C::Source, C::Span>>) -> Self {
		Self {
			context,
			warnings
		}
	}

	pub fn context(&self) -> &Context<T, C> {
		&self.context
	}

	pub fn warnings(&self) -> &[Loc<Warning, C::Source, C::Span>] {
		&self.warnings
	}

	pub fn into_context(self) -> Context<T, C> {
		self.context
	}

	pub fn into_warnings(self) -> Vec<Loc<Warning, C::Source, C::Span>> {
		self.warnings
	}

	pub fn into_parts(self) -> (Context<T, C>, Vec<Loc<Warning, C::Source, C::Span>>) {
		(self.context, self.warnings)
	}
}

/// Result of context processing functions.
pub type ProcessingResult<T, C> = Result<ProcessedContext<T, C>, LocError<T, C>>;

/// Context processing functions.
pub trait Process<T>: Sized + Send + Sync {
	type Source;
	type Span;

	/// Process the local context with specific options.
	fn process_full<'a, L: Loader + Send + Sync>(
		&'a self,
		active_context: &'a Context<T, Self>,
		stack: ProcessingStack,
		loader: &'a mut L,
		base_url: Option<Iri<'a>>,
		options: ProcessingOptions,
	) -> BoxFuture<'a, ProcessingResult<T, Self>>
	where
		L::Output: Into<Self>,
		T: Send + Sync;

	/// Process the local context with specific options.
	fn process_with<'a, L: Loader + Send + Sync>(
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
		)
	}

	/// Process the local context with the given initial active context with the default options:
	/// `is_remote` is `false`, `override_protected` is `false` and `propagate` is `true`.
	fn process<'a, L: Loader + Send + Sync>(
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