use futures::future::{BoxFuture, FutureExt};
use iref::Iri;
use locspan::Meta;

mod none;

pub use none::NoLoader;

/// JSON document loader.
pub trait Loader {
	/// The type of documents that can be loaded.
	type Output;
	type Error;
	type Metadata;

	/// Loads the document behind the given IRI.
	fn load<'a>(
		&'a mut self,
		url: Iri,
	) -> BoxFuture<'a, Result<Meta<Self::Output, Self::Metadata>, Self::Error>>;
}

pub trait ContextLoader {
	/// Output of the loader.
	type Output;
	type Error;
	type Metadata;

	fn load_context<'a>(
		&'a mut self,
		url: Iri,
	) -> BoxFuture<'a, Result<Meta<Self::Output, Self::Metadata>, Self::Error>>;
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

impl<C, M> ExtractContext for json_ld_syntax::Value<C, M> {
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

impl<L: Loader> ContextLoader for L
where
	L::Output: ExtractContext<Metadata = L::Metadata>,
{
	type Output = <L::Output as ExtractContext>::Context;
	type Error = ContextLoaderError<L::Error, <L::Output as ExtractContext>::Error>;
	type Metadata = L::Metadata;

	fn load_context<'a>(
		&'a mut self,
		url: Iri,
	) -> BoxFuture<'a, Result<Meta<Self::Output, L::Metadata>, Self::Error>> {
		let load_document = self.load(url);
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
