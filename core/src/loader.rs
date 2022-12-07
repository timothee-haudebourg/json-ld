use futures::future::{BoxFuture, FutureExt};
use locspan::{Location, MapLocErr, Meta};
use mown::Mown;
use rdf_types::{vocabulary::Index, IriVocabulary};
use std::fmt;

pub mod fs;
pub mod none;

pub use fs::FsLoader;
pub use none::NoLoader;

// #[cfg(feature="reqwest")]
// pub mod reqwest;

// #[cfg(feature="reqwest")]
// pub use self::reqwest::ReqwestLoader;

pub type LoadingResult<I, M, O, E> = Result<RemoteDocument<I, M, O>, E>;

#[derive(Clone)]
pub enum RemoteDocumentReference<I = Index, M = Location<I>, T = json_syntax::Value<M>> {
	Iri(I),
	Loaded(RemoteDocument<I, M, T>),
}

impl<I, M> RemoteDocumentReference<I, M, json_syntax::Value<M>> {
	/// Creates an IRI to a `json_syntax::Value<M>` JSON document.
	///
	/// This method can replace `RemoteDocumentReference::Iri` to help the type
	/// inference in the case where `T = json_syntax::Value<M>`.
	pub fn iri(iri: I) -> Self {
		Self::Iri(iri)
	}
}

impl<I, M> RemoteDocumentReference<I, M, json_ld_syntax::context::Value<M>> {
	/// Creates an IRI to a `json_ld_syntax::context::Value<M>` JSON-LD context document.
	///
	/// This method can replace `RemoteDocumentReference::Iri` to help the type
	/// inference in the case where `T = json_ld_syntax::context::Value<M>`.
	pub fn context_iri(iri: I) -> Self {
		Self::Iri(iri)
	}
}

impl<I, M, T> RemoteDocumentReference<I, M, T> {
	pub async fn load_with<L: Loader<I, M>>(
		self,
		vocabulary: &(impl Sync + IriVocabulary<Iri = I>),
		loader: &mut L,
	) -> LoadingResult<I, M, T, L::Error>
	where
		L::Output: Into<T>,
	{
		match self {
			Self::Iri(r) => Ok(loader
				.load_with(vocabulary, r)
				.await?
				.map(|m| m.map(Into::into))),
			Self::Loaded(doc) => Ok(doc),
		}
	}

	pub async fn loaded_with<L: Loader<I, M>>(
		&self,
		vocabulary: &(impl Sync + IriVocabulary<Iri = I>),
		loader: &mut L,
	) -> Result<Mown<'_, RemoteDocument<I, M, T>>, L::Error>
	where
		I: Clone,
		L::Output: Into<T>,
	{
		match self {
			Self::Iri(r) => Ok(Mown::Owned(
				loader
					.load_with(vocabulary, r.clone())
					.await?
					.map(|m| m.map(Into::into)),
			)),
			Self::Loaded(doc) => Ok(Mown::Borrowed(doc)),
		}
	}

	pub async fn load_context_with<L: ContextLoader<I, M>>(
		self,
		vocabulary: &(impl Sync + IriVocabulary<Iri = I>),
		loader: &mut L,
	) -> LoadingResult<I, M, T, L::ContextError>
	where
		L::Context: Into<T>,
	{
		match self {
			Self::Iri(r) => Ok(loader
				.load_context_with(vocabulary, r)
				.await?
				.map(|m| m.map(Into::into))),
			Self::Loaded(doc) => Ok(doc),
		}
	}

	pub async fn loaded_context_with<L: ContextLoader<I, M>>(
		&self,
		vocabulary: &(impl Sync + IriVocabulary<Iri = I>),
		loader: &mut L,
	) -> Result<Mown<'_, RemoteDocument<I, M, T>>, L::ContextError>
	where
		I: Clone,
		L::Context: Into<T>,
	{
		match self {
			Self::Iri(r) => Ok(Mown::Owned(
				loader
					.load_context_with(vocabulary, r.clone())
					.await?
					.map(|m| m.map(Into::into)),
			)),
			Self::Loaded(doc) => Ok(Mown::Borrowed(doc)),
		}
	}
}

/// Remote document.
///
/// Stores the content of a loaded remote document along with its original URL.
#[derive(Clone)]
pub struct RemoteDocument<I = Index, M = Location<I>, T = json_syntax::Value<M>> {
	/// Document URL.
	url: Option<I>,

	/// Document content.
	document: Meta<T, M>,
}

impl<I, M, T> RemoteDocument<I, M, T> {
	/// Creates a new remote document.
	pub fn new(url: Option<I>, document: Meta<T, M>) -> Self {
		Self { url, document }
	}

	/// Maps the content of the remote document.
	pub fn map<U, N>(self, f: impl Fn(Meta<T, M>) -> Meta<U, N>) -> RemoteDocument<I, N, U> {
		RemoteDocument {
			url: self.url,
			document: f(self.document),
		}
	}

	/// Tries to map the content of the remote document.
	pub fn try_map<U, N, E>(
		self,
		f: impl Fn(Meta<T, M>) -> Result<Meta<U, N>, E>,
	) -> Result<RemoteDocument<I, N, U>, E> {
		Ok(RemoteDocument {
			url: self.url,
			document: f(self.document)?,
		})
	}

	/// Returns a reference the original URL of the document.
	pub fn url(&self) -> Option<&I> {
		self.url.as_ref()
	}

	/// Returns a reference to the content of the document.
	pub fn document(&self) -> &Meta<T, M> {
		&self.document
	}

	/// Drops the original URL and returns the content of the document.
	pub fn into_document(self) -> Meta<T, M> {
		self.document
	}

	/// Drops the content and returns the original URL of the document.
	pub fn into_url(self) -> Option<I> {
		self.url
	}

	/// Sets the URL of the document.
	pub fn set_url(&mut self, url: Option<I>) {
		self.url = url
	}
}

/// Document loader.
pub trait Loader<I, M> {
	/// The type of documents that can be loaded.
	type Output;

	/// Error type.
	type Error;

	/// Loads the document behind the given IRI, using the given vocabulary.
	fn load_with<'a>(
		&'a mut self,
		vocabulary: &'a (impl Sync + IriVocabulary<Iri = I>),
		url: I,
	) -> BoxFuture<'a, LoadingResult<I, M, Self::Output, Self::Error>>
	where
		I: 'a;

	/// Loads the document behind the given IRI.
	fn load<'a>(
		&'a mut self,
		url: I,
	) -> BoxFuture<'a, LoadingResult<I, M, Self::Output, Self::Error>>
	where
		I: 'a,
		(): IriVocabulary<Iri = I>,
	{
		self.load_with(&(), url)
	}
}

/// Context document loader.
///
/// This is a particular loader able to extract a local context from a loaded
/// JSON-LD document.
pub trait ContextLoader<I, M> {
	/// Output of the loader, a context.
	type Context;

	/// Error type.
	type ContextError;

	/// Loads the context behind the given IRI, using the given vocabulary.
	fn load_context_with<'a>(
		&'a mut self,
		vocabulary: &'a (impl Sync + IriVocabulary<Iri = I>),
		url: I,
	) -> BoxFuture<'a, LoadingResult<I, M, Self::Context, Self::ContextError>>
	where
		I: 'a,
		M: 'a;

	/// Loads the context behind the given IRI.
	fn load_context<'a>(
		&'a mut self,
		url: I,
	) -> BoxFuture<'a, LoadingResult<I, M, Self::Context, Self::ContextError>>
	where
		I: 'a,
		M: 'a,
		(): IriVocabulary<Iri = I>,
	{
		self.load_context_with(&(), url)
	}
}

/// Context extraction method.
///
/// Implemented by documents containing a JSON-LD context definition, providing
/// a method to extract this context.
pub trait ExtractContext<M>: Sized {
	/// Context definition type.
	type Context;

	/// Error type.
	///
	/// May be raised if the inner context is missing or invalid.
	type Error;

	/// Extract the context definition.
	fn extract_context(value: Meta<Self, M>) -> Result<Meta<Self::Context, M>, Self::Error>;
}

/// Context extraction error.
#[derive(Debug)]
pub enum ExtractContextError<M> {
	/// Unexpected JSON value.
	Unexpected(json_syntax::Kind),

	/// No context definition found.
	NoContext,

	/// Multiple context definitions found.
	DuplicateContext(M),

	/// JSON syntax error.
	Syntax(json_ld_syntax::context::InvalidContext),
}

impl<M> ExtractContextError<M> {
	fn duplicate_context(
		json_syntax::object::Duplicate(a, b): json_syntax::object::Duplicate<
			json_syntax::object::Entry<M>,
		>,
	) -> Meta<Self, M> {
		Meta(
			Self::DuplicateContext(a.key.into_metadata()),
			b.key.into_metadata(),
		)
	}
}

impl<M> fmt::Display for ExtractContextError<M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Unexpected(k) => write!(f, "unexpected {}", k),
			Self::NoContext => write!(f, "missing context"),
			Self::DuplicateContext(_) => write!(f, "duplicate context"),
			Self::Syntax(e) => e.fmt(f),
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
			json_syntax::Value::Object(mut o) => match o
				.remove_unique("@context")
				.map_err(ExtractContextError::duplicate_context)?
			{
				Some(context) => {
					use json_ld_syntax::TryFromJson;
					json_ld_syntax::context::Value::try_from_json(context.value)
						.map_loc_err(ExtractContextError::Syntax)
				}
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

	fn load_context_with<'a>(
		&'a mut self,
		vocabulary: &'a (impl Sync + IriVocabulary<Iri = I>),
		url: I,
	) -> BoxFuture<'a, Result<RemoteDocument<I, M, Self::Context>, Self::ContextError>>
	where
		I: 'a,
		M: 'a,
	{
		let load_document = self.load_with(vocabulary, url);
		async move {
			let doc = load_document
				.await
				.map_err(ContextLoaderError::LoadingDocumentFailed)?;

			doc.try_map(ExtractContext::extract_context)
				.map_err(ContextLoaderError::ContextExtractionFailed)
		}
		.boxed()
	}
}
