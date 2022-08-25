use crate::IriNamespace;
use futures::future::{BoxFuture, FutureExt};
use locspan::Meta;
use std::fmt;

pub mod fs;
pub mod none;

pub use fs::FsLoader;
pub use none::NoLoader;

pub type LoadingResult<O, M, E> = Result<Meta<O, M>, E>;

/// JSON document loader.
pub trait Loader<I> {
	/// The type of documents that can be loaded.
	type Output;
	type Error;
	type Metadata;

	/// Loads the document behind the given IRI, inside the given namespace.
	fn load_in<'a>(
		&'a mut self,
		namespace: &'a (impl Sync + IriNamespace<I>),
		url: I,
	) -> BoxFuture<'a, LoadingResult<Self::Output, Self::Metadata, Self::Error>>
	where
		I: 'a;

	/// Loads the document behind the given IRI.
	fn load<'a>(
		&'a mut self,
		url: I,
	) -> BoxFuture<'a, LoadingResult<Self::Output, Self::Metadata, Self::Error>>
	where
		I: 'a,
		(): IriNamespace<I>,
	{
		self.load_in(&(), url)
	}
}

pub trait ContextLoader<I> {
	/// Output of the loader.
	type Output;
	type ContextError;
	type Metadata;

	fn load_context_in<'a>(
		&'a mut self,
		namespace: &'a (impl Sync + IriNamespace<I>),
		url: I,
	) -> BoxFuture<'a, LoadingResult<Self::Output, Self::Metadata, Self::ContextError>>
	where
		I: 'a;

	fn load_context<'a>(
		&'a mut self,
		url: I,
	) -> BoxFuture<'a, LoadingResult<Self::Output, Self::Metadata, Self::ContextError>>
	where
		I: 'a,
		(): IriNamespace<I>,
	{
		self.load_context_in(&(), url)
	}
}

pub trait ExtractContext: Sized {
	type Context;
	type Error;
	type Metadata;

	fn extract_context(
		value: Meta<Self, Self::Metadata>,
	) -> Result<Meta<Self::Context, Self::Metadata>, Self::Error>;
}

#[derive(Debug)]
pub enum ExtractContextError {
	Unexpected(json_syntax::Kind),
	NoContext,
}

impl fmt::Display for ExtractContextError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Unexpected(k) => write!(f, "unexpected {}", k),
			Self::NoContext => write!(f, "missing context"),
		}
	}
}

impl<M, C> ExtractContext for json_ld_syntax::Value<M, C> {
	type Context = C;
	type Error = Meta<ExtractContextError, M>;
	type Metadata = M;

	fn extract_context(
		Meta(value, meta): Meta<Self, M>,
	) -> Result<Meta<Self::Context, M>, Self::Error> {
		match value {
			json_ld_syntax::Value::Object(mut o) => match o.remove_context() {
				Some(context) => Ok(context.into_context()),
				None => Err(Meta(ExtractContextError::NoContext, meta)),
			},
			other => Err(Meta(ExtractContextError::Unexpected(other.kind()), meta)),
		}
	}
}

#[derive(Debug)]
pub enum ContextLoaderError<D, C> {
	LoadingDocumentFailed(D),
	ContextExtractionFailed(C),
}

impl<D: fmt::Display, C: fmt::Display> fmt::Display for ContextLoaderError<D, C> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::LoadingDocumentFailed(e) => e.fmt(f),
			Self::ContextExtractionFailed(e) => e.fmt(f),
		}
	}
}

impl<I: Send, L: Loader<I>> ContextLoader<I> for L
where
	L::Output: ExtractContext<Metadata = L::Metadata>,
{
	type Output = <L::Output as ExtractContext>::Context;
	type ContextError = ContextLoaderError<L::Error, <L::Output as ExtractContext>::Error>;
	type Metadata = L::Metadata;

	fn load_context_in<'a>(
		&'a mut self,
		namespace: &'a (impl Sync + IriNamespace<I>),
		url: I,
	) -> BoxFuture<'a, Result<Meta<Self::Output, L::Metadata>, Self::ContextError>>
	where
		I: 'a,
	{
		let load_document = self.load_in(namespace, url);
		async move {
			let doc = load_document
				.await
				.map_err(ContextLoaderError::LoadingDocumentFailed)?;
			ExtractContext::extract_context(doc)
				.map_err(ContextLoaderError::ContextExtractionFailed)
		}
		.boxed()
	}
}
