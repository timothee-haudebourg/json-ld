use crate::future::{BoxFuture, FutureExt};
use hashbrown::HashSet;
use iref::{Iri, IriBuf};
use locspan::{MapLocErr, Meta};
use mime::Mime;
use mown::Mown;
use rdf_types::{IriVocabulary, IriVocabularyMut};
use static_iref::iri;
use std::fmt;

pub mod fs;
pub mod none;

pub use fs::FsLoader;
pub use none::NoLoader;

#[cfg(feature = "reqwest")]
pub mod reqwest;

#[cfg(feature = "reqwest")]
pub use self::reqwest::ReqwestLoader;

pub type LoadingResult<I, M, O, E> = Result<RemoteDocument<I, M, O>, E>;

/// Remote document, loaded or not.
///
/// Either an IRI or the actual document content.
#[derive(Clone)]
pub enum RemoteDocumentReference<I = IriBuf, M = (), T = json_syntax::Value<M>> {
	/// IRI to the remote document.
	Iri(I),

	/// Remote document content.
	Loaded(RemoteDocument<I, M, T>),
}

pub type RemoteContextReference<I = IriBuf, M = ()> =
	RemoteDocumentReference<I, M, json_ld_syntax::context::Context<M>>;

impl<I, M> RemoteDocumentReference<I, M, json_syntax::Value<M>> {
	/// Creates an IRI to a `json_syntax::Value<M>` JSON document.
	///
	/// This method can replace `RemoteDocumentReference::Iri` to help the type
	/// inference in the case where `T = json_syntax::Value<M>`.
	pub fn iri(iri: I) -> Self {
		Self::Iri(iri)
	}
}

impl<I, M> RemoteDocumentReference<I, M, json_ld_syntax::context::Context<M>> {
	/// Creates an IRI to a `json_ld_syntax::context::Value<M>` JSON-LD context document.
	///
	/// This method can replace `RemoteDocumentReference::Iri` to help the type
	/// inference in the case where `T = json_ld_syntax::context::Value<M>`.
	pub fn context_iri(iri: I) -> Self {
		Self::Iri(iri)
	}
}

impl<I, M, T> RemoteDocumentReference<I, M, T> {
	/// Loads the remote document with the given `vocabulary` and `loader`.
	///
	/// If the document is already [`Self::Loaded`], simply returns the inner
	/// [`RemoteDocument`].
	pub async fn load_with<L: Loader<I, M>>(
		self,
		vocabulary: &mut (impl Sync + Send + IriVocabularyMut<Iri = I>),
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

	/// Loads the remote document with the given `vocabulary` and `loader`.
	///
	/// For [`Self::Iri`] returns an owned [`RemoteDocument`] with
	/// [`Mown::Owned`].
	/// For [`Self::Loaded`] returns a reference to the inner [`RemoteDocument`]
	/// with [`Mown::Borrowed`].
	pub async fn loaded_with<L: Loader<I, M>>(
		&self,
		vocabulary: &mut (impl Sync + Send + IriVocabularyMut<Iri = I>),
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
}

impl<I, M> RemoteContextReference<I, M> {
	/// Loads the remote context definition with the given `vocabulary` and
	/// `loader`.
	///
	/// If the document is already [`Self::Loaded`], simply returns the inner
	/// [`RemoteDocument`].
	pub async fn load_context_with<L: ContextLoader<I, M>>(
		self,
		vocabulary: &mut (impl Send + Sync + IriVocabularyMut<Iri = I>),
		loader: &mut L,
	) -> LoadingResult<I, M, json_ld_syntax::context::Context<M>, L::ContextError> {
		match self {
			Self::Iri(r) => Ok(loader
				.load_context_with(vocabulary, r)
				.await?
				.map(|m| m.map(Into::into))),
			Self::Loaded(doc) => Ok(doc),
		}
	}

	/// Loads the remote context definition with the given `vocabulary` and
	/// `loader`.
	///
	/// For [`Self::Iri`] returns an owned [`RemoteDocument`] with
	/// [`Mown::Owned`].
	/// For [`Self::Loaded`] returns a reference to the inner [`RemoteDocument`]
	/// with [`Mown::Borrowed`].
	pub async fn loaded_context_with<L: ContextLoader<I, M>>(
		&self,
		vocabulary: &mut (impl Send + Sync + IriVocabularyMut<Iri = I>),
		loader: &mut L,
	) -> Result<Mown<'_, RemoteDocument<I, M, json_ld_syntax::context::Context<M>>>, L::ContextError>
	where
		I: Clone,
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
#[derive(Debug, Clone)]
pub struct RemoteDocument<I = IriBuf, M = (), T = json_syntax::Value<M>> {
	/// The final URL of the loaded document, after eventual redirection.
	url: Option<I>,

	/// The HTTP `Content-Type` header value of the loaded document, exclusive
	/// of any optional parameters.
	content_type: Option<Mime>,

	/// If available, the value of the HTTP `Link Header` [RFC 8288] using the
	/// `http://www.w3.org/ns/json-ld#context` link relation in the response.
	///
	/// If the response's `Content-Type` is `application/ld+json`, the HTTP
	/// `Link Header` is ignored. If multiple HTTP `Link Headers` using the
	/// `http://www.w3.org/ns/json-ld#context` link relation are found, the
	/// loader fails with a `multiple context link headers` error.
	///
	/// [RFC 8288]: https://www.rfc-editor.org/rfc/rfc8288
	context_url: Option<I>,

	profile: HashSet<Profile<I>>,

	/// The retrieved document.
	document: Meta<T, M>,
}

pub type RemoteContext<I = IriBuf, M = ()> =
	RemoteDocument<I, M, json_ld_syntax::context::Context<M>>;

impl<I, M, T> RemoteDocument<I, M, T> {
	/// Creates a new remote document.
	///
	/// `url` is the final URL of the loaded document, after eventual
	/// redirection.
	/// `content_type` is the HTTP `Content-Type` header value of the loaded
	/// document, exclusive of any optional parameters.
	pub fn new(url: Option<I>, content_type: Option<Mime>, document: Meta<T, M>) -> Self {
		Self::new_full(url, content_type, None, HashSet::new(), document)
	}

	/// Creates a new remote document.
	///
	/// `url` is the final URL of the loaded document, after eventual
	/// redirection.
	/// `content_type` is the HTTP `Content-Type` header value of the loaded
	/// document, exclusive of any optional parameters.
	/// `context_url` is the value of the HTTP `Link Header` [RFC 8288] using the
	/// `http://www.w3.org/ns/json-ld#context` link relation in the response,
	/// if any.
	/// `profile` is the value of any profile parameter retrieved as part of the
	/// original contentType.
	///
	/// [RFC 8288]: https://www.rfc-editor.org/rfc/rfc8288
	pub fn new_full(
		url: Option<I>,
		content_type: Option<Mime>,
		context_url: Option<I>,
		profile: HashSet<Profile<I>>,
		document: Meta<T, M>,
	) -> Self {
		Self {
			url,
			content_type,
			context_url,
			profile,
			document,
		}
	}

	/// Maps the content of the remote document.
	pub fn map<U, N>(self, f: impl Fn(Meta<T, M>) -> Meta<U, N>) -> RemoteDocument<I, N, U> {
		RemoteDocument {
			url: self.url,
			content_type: self.content_type,
			context_url: self.context_url,
			profile: self.profile,
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
			content_type: self.content_type,
			context_url: self.context_url,
			profile: self.profile,
			document: f(self.document)?,
		})
	}

	/// Returns a reference to the final URL of the loaded document, after eventual redirection.
	pub fn url(&self) -> Option<&I> {
		self.url.as_ref()
	}

	/// Returns the HTTP `Content-Type` header value of the loaded document,
	/// exclusive of any optional parameters.
	pub fn content_type(&self) -> Option<&Mime> {
		self.content_type.as_ref()
	}

	/// Returns the value of the HTTP `Link Header` [RFC 8288] using the
	/// `http://www.w3.org/ns/json-ld#context` link relation in the response,
	/// if any.
	///
	/// If the response's `Content-Type` is `application/ld+json`, the HTTP
	/// `Link Header` is ignored. If multiple HTTP `Link Headers` using the
	/// `http://www.w3.org/ns/json-ld#context` link relation are found, the
	/// loader fails with a `multiple context link headers` error.
	///
	/// [RFC 8288]: https://www.rfc-editor.org/rfc/rfc8288
	pub fn context_url(&self) -> Option<&I> {
		self.context_url.as_ref()
	}

	/// Returns a reference to the content of the document.
	pub fn document(&self) -> &Meta<T, M> {
		&self.document
	}

	/// Returns a mutable reference to the content of the document.
	pub fn document_mut(&mut self) -> &mut Meta<T, M> {
		&mut self.document
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

/// Standard `profile` parameter values defined for the `application/ld+json`.
///
/// See: <https://www.w3.org/TR/json-ld11/#iana-considerations>
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StandardProfile {
	/// To request or specify expanded JSON-LD document form.
	Expanded,

	/// To request or specify compacted JSON-LD document form.
	Compacted,

	/// To request or specify a JSON-LD context document.
	Context,

	/// To request or specify flattened JSON-LD document form.
	Flattened,

	// /// To request or specify a JSON-LD frame document.
	// Frame,
	/// To request or specify a JSON-LD framed document.
	Framed,
}

impl StandardProfile {
	pub fn from_iri(iri: &Iri) -> Option<Self> {
		if iri == iri!("http://www.w3.org/ns/json-ld#expanded") {
			Some(Self::Expanded)
		} else if iri == iri!("http://www.w3.org/ns/json-ld#compacted") {
			Some(Self::Compacted)
		} else if iri == iri!("http://www.w3.org/ns/json-ld#context") {
			Some(Self::Context)
		} else if iri == iri!("http://www.w3.org/ns/json-ld#flattened") {
			Some(Self::Flattened)
		} else if iri == iri!("http://www.w3.org/ns/json-ld#framed") {
			Some(Self::Framed)
		} else {
			None
		}
	}

	pub fn iri(&self) -> &'static Iri {
		match self {
			Self::Expanded => iri!("http://www.w3.org/ns/json-ld#expanded"),
			Self::Compacted => iri!("http://www.w3.org/ns/json-ld#compacted"),
			Self::Context => iri!("http://www.w3.org/ns/json-ld#context"),
			Self::Flattened => iri!("http://www.w3.org/ns/json-ld#flattened"),
			Self::Framed => iri!("http://www.w3.org/ns/json-ld#framed"),
		}
	}
}

/// Value for the `profile` parameter defined for the `application/ld+json`.
///
/// Standard values defined by the JSON-LD specification are defined by the
/// [`StandardProfile`] type.
///
/// See: <https://www.w3.org/TR/json-ld11/#iana-considerations>
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Profile<I> {
	Standard(StandardProfile),
	Custom(I),
}

impl<I> Profile<I> {
	pub fn new(iri: &Iri, vocabulary: &mut impl IriVocabularyMut<Iri = I>) -> Self {
		match StandardProfile::from_iri(iri) {
			Some(p) => Self::Standard(p),
			None => Self::Custom(vocabulary.insert(iri)),
		}
	}

	pub fn iri<'a>(&'a self, vocabulary: &'a impl IriVocabulary<Iri = I>) -> &'a Iri {
		match self {
			Self::Standard(s) => s.iri(),
			Self::Custom(c) => vocabulary.iri(c).unwrap(),
		}
	}
}

/// Document loader.
///
/// A document loader is required by most processing functions to fetch remote
/// documents identified by an IRI.
///
/// This library provides a few default loader implementations:
///   - [`NoLoader`] dummy loader that always fail. Perfect if you are certain
///     that the processing will not require any loading.
///   - [`FsLoader`] that redirect registered IRI prefixes to a local directory
///     on the file system. This way no network calls are performed and the
///     loaded content can be trusted.
///   - `ReqwestLoader` that actually download the remote documents using the
///     [`reqwest`](https://crates.io/crates/reqwest) library.
///     This requires the `reqwest` feature to be enabled.
pub trait Loader<I = IriBuf, M = ()> {
	/// The type of documents that can be loaded.
	type Output;

	/// Error type.
	type Error;

	/// Loads the document behind the given IRI, using the given vocabulary.
	fn load_with<'a>(
		&'a mut self,
		vocabulary: &'a mut (impl Sync + Send + IriVocabularyMut<Iri = I>),
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
		self.load_with(rdf_types::vocabulary::no_vocabulary_mut(), url)
	}
}

/// Context document loader.
///
/// This is a subclass of loaders able to extract a local context from a loaded
/// JSON-LD document.
///
/// It is implemented for any loader where the output type implements
/// [`ExtractContext`].
pub trait ContextLoader<I = IriBuf, M = ()> {
	/// Error type.
	type ContextError;

	/// Loads the context behind the given IRI, using the given vocabulary.
	fn load_context_with<'a>(
		&'a mut self,
		vocabulary: &'a mut (impl Send + Sync + IriVocabularyMut<Iri = I>),
		url: I,
	) -> BoxFuture<'a, LoadingResult<I, M, json_ld_syntax::context::Context<M>, Self::ContextError>>
	where
		I: 'a,
		M: 'a;

	/// Loads the context behind the given IRI.
	fn load_context<'a>(
		&'a mut self,
		url: I,
	) -> BoxFuture<'a, LoadingResult<I, M, json_ld_syntax::context::Context<M>, Self::ContextError>>
	where
		I: 'a,
		M: 'a,
		(): IriVocabulary<Iri = I>,
	{
		self.load_context_with(rdf_types::vocabulary::no_vocabulary_mut(), url)
	}
}

/// Context extraction method.
///
/// Implemented by documents containing a JSON-LD context definition, providing
/// a method to extract this context.
pub trait ExtractContext<M>: Sized {
	/// Error type.
	///
	/// May be raised if the inner context is missing or invalid.
	type Error;

	/// Extract the context definition.
	fn extract_context(
		value: Meta<Self, M>,
	) -> Result<Meta<json_ld_syntax::context::Context<M>, M>, Self::Error>;
}

/// Context extraction error.
#[derive(Debug)]
pub enum ExtractContextError<M = ()> {
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
			Self::Unexpected(k) => write!(f, "unexpected {k}"),
			Self::NoContext => write!(f, "missing context"),
			Self::DuplicateContext(_) => write!(f, "duplicate context"),
			Self::Syntax(e) => e.fmt(f),
		}
	}
}

impl<M: Clone> ExtractContext<M> for json_syntax::Value<M> {
	type Error = Meta<ExtractContextError<M>, M>;

	fn extract_context(
		Meta(value, meta): Meta<Self, M>,
	) -> Result<Meta<json_ld_syntax::context::Context<M>, M>, Self::Error> {
		match value {
			json_syntax::Value::Object(mut o) => match o
				.remove_unique("@context")
				.map_err(ExtractContextError::duplicate_context)?
			{
				Some(context) => {
					use json_ld_syntax::TryFromJson;
					json_ld_syntax::context::Context::try_from_json(context.value)
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
	type ContextError = ContextLoaderError<L::Error, <L::Output as ExtractContext<M>>::Error>;

	fn load_context_with<'a>(
		&'a mut self,
		vocabulary: &'a mut (impl Send + Sync + IriVocabularyMut<Iri = I>),
		url: I,
	) -> BoxFuture<
		'a,
		Result<RemoteDocument<I, M, json_ld_syntax::context::Context<M>>, Self::ContextError>,
	>
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
