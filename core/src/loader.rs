use crate::IriNamespace;
use futures::future::{BoxFuture, FutureExt};
use locspan::{Meta, MapLocErr};
use std::fmt;

pub mod fs;
pub mod none;

pub use fs::FsLoader;
pub use none::NoLoader;

pub type LoadingResult<O, M, E> = Result<Meta<O, M>, E>;

/// JSON document loader.
pub trait Loader<I, M> {
	/// The type of documents that can be loaded.
	type Output;
	type Error;

	/// Loads the document behind the given IRI, inside the given namespace.
	fn load_in<'a>(
		&'a mut self,
		namespace: &'a (impl Sync + IriNamespace<I>),
		url: I,
	) -> BoxFuture<'a, LoadingResult<Self::Output, M, Self::Error>>
	where
		I: 'a;

	/// Loads the document behind the given IRI.
	fn load<'a>(
		&'a mut self,
		url: I,
	) -> BoxFuture<'a, LoadingResult<Self::Output, M, Self::Error>>
	where
		I: 'a,
		(): IriNamespace<I>,
	{
		self.load_in(&(), url)
	}
}

pub trait ContextLoader<I, M> {
	/// Output of the loader, a context.
	type Context;
	type ContextError;

	fn load_context_in<'a>(
		&'a mut self,
		namespace: &'a (impl Sync + IriNamespace<I>),
		url: I,
	) -> BoxFuture<'a, LoadingResult<Self::Context, M, Self::ContextError>>
	where
		I: 'a,
		M: 'a;

	fn load_context<'a>(
		&'a mut self,
		url: I,
	) -> BoxFuture<'a, LoadingResult<Self::Context, M, Self::ContextError>>
	where
		I: 'a,
		M: 'a,
		(): IriNamespace<I>,
	{
		self.load_context_in(&(), url)
	}
}

pub trait ExtractContext<M>: Sized {
	type Context;
	type Error;

	fn extract_context(
		value: Meta<Self, M>,
	) -> Result<Meta<Self::Context, M>, Self::Error>;
}

#[derive(Debug)]
pub enum ExtractContextError<M> {
	Unexpected(json_syntax::Kind),
	NoContext,
	DuplicateContext(M),
	Syntax(json_ld_syntax::context::InvalidContext)
}

impl<M> ExtractContextError<M> {
	fn duplicate_context(json_syntax::object::Duplicate(a, b): json_syntax::object::Duplicate<json_syntax::object::Entry<M>>) -> Meta<Self, M> {
		Meta(Self::DuplicateContext(a.key.into_metadata()), b.key.into_metadata())
	}
}

impl<M> fmt::Display for ExtractContextError<M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Unexpected(k) => write!(f, "unexpected {}", k),
			Self::NoContext => write!(f, "missing context"),
			Self::DuplicateContext(_) => write!(f, "duplicate context"),
			Self::Syntax(e) => e.fmt(f)
		}
	}
}

impl<M: Clone> ExtractContext<M> for json_syntax::Value<M> {
	type Context = json_ld_syntax::context::Value<M>;
	type Error = Meta<ExtractContextError<M>, M>;

	fn extract_context(
		Meta(value, meta): Meta<Self, M>,
	) -> Result<Meta<Self::Context, M>, Self::Error> {
		match value {
			json_syntax::Value::Object(mut o) => match o.remove_unique("@context").map_err(ExtractContextError::duplicate_context)? {
				Some(context) => {
					use json_ld_syntax::TryFromJson;
					json_ld_syntax::context::Value::try_from_json(context.value).map_loc_err(ExtractContextError::Syntax)
				},
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

impl<I: Send, M, L: Loader<I, M>> ContextLoader<I, M> for L
where
	L::Output: ExtractContext<M>,
{
	type Context = <L::Output as ExtractContext<M>>::Context;
	type ContextError = ContextLoaderError<L::Error, <L::Output as ExtractContext<M>>::Error>;

	fn load_context_in<'a>(
		&'a mut self,
		namespace: &'a (impl Sync + IriNamespace<I>),
		url: I,
	) -> BoxFuture<'a, Result<Meta<Self::Context, M>, Self::ContextError>>
	where
		I: 'a,
		M: 'a
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
