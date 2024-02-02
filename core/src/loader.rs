use crate::future::BoxFuture;
use hashbrown::HashSet;
use iref::{Iri, IriBuf};
use mime::Mime;
use rdf_types::{IriVocabulary, IriVocabularyMut};
use static_iref::iri;
use std::borrow::Cow;

pub mod chain;
pub mod fs;
pub mod none;

pub use chain::ChainLoader;
pub use fs::FsLoader;
pub use none::NoLoader;

#[cfg(feature = "reqwest")]
pub mod reqwest;

#[cfg(feature = "reqwest")]
pub use self::reqwest::ReqwestLoader;

pub type LoadingResult<I, E> = Result<RemoteDocument<I>, E>;

pub type RemoteContextReference<I = IriBuf> = RemoteDocumentReference<I, json_ld_syntax::Context>;

/// Remote document, loaded or not.
///
/// Either an IRI or the actual document content.
#[derive(Clone)]
pub enum RemoteDocumentReference<I = IriBuf, T = json_syntax::Value> {
	/// IRI to the remote document.
	Iri(I),

	/// Remote document content.
	Loaded(RemoteDocument<I, T>),
}

impl<I, T> RemoteDocumentReference<I, T> {
	/// Creates an IRI to a `json_syntax::Value` JSON document.
	///
	/// This method can replace `RemoteDocumentReference::Iri` to help the type
	/// inference in the case where `T = json_syntax::Value`.
	pub fn iri(iri: I) -> Self {
		Self::Iri(iri)
	}
}

impl<I> RemoteDocumentReference<I> {
	/// Loads the remote document with the given `vocabulary` and `loader`.
	///
	/// If the document is already [`Self::Loaded`], simply returns the inner
	/// [`RemoteDocument`].
	pub async fn load_with<V, L: Loader<I>>(
		self,
		vocabulary: &mut V,
		loader: &mut L,
	) -> Result<RemoteDocument<I>, L::Error>
	where
		V: IriVocabularyMut<Iri = I>,
		//
		V: Send + Sync,
		I: Send,
	{
		match self {
			Self::Iri(r) => Ok(loader.load_with(vocabulary, r).await?.map(Into::into)),
			Self::Loaded(doc) => Ok(doc),
		}
	}

	/// Loads the remote document with the given `vocabulary` and `loader`.
	///
	/// For [`Self::Iri`] returns an owned [`RemoteDocument`] with
	/// [`Cow::Owned`].
	/// For [`Self::Loaded`] returns a reference to the inner [`RemoteDocument`]
	/// with [`Cow::Borrowed`].
	pub async fn loaded_with<V, L: Loader<I>>(
		&self,
		vocabulary: &mut V,
		loader: &mut L,
	) -> Result<Cow<'_, RemoteDocument<I>>, L::Error>
	where
		V: IriVocabularyMut<Iri = I>,
		I: Clone,
		//
		V: Send + Sync,
		I: Send,
	{
		match self {
			Self::Iri(r) => Ok(Cow::Owned(
				loader
					.load_with(vocabulary, r.clone())
					.await?
					.map(Into::into),
			)),
			Self::Loaded(doc) => Ok(Cow::Borrowed(doc)),
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum ContextLoadError<E> {
	#[error("loading document failed: {0}")]
	LoadingDocumentFailed(E),

	#[error("context extraction failed")]
	ContextExtractionFailed(#[from] ExtractContextError),
}

impl<I> RemoteContextReference<I> {
	/// Loads the remote context with the given `vocabulary` and `loader`.
	///
	/// If the context is already [`Self::Loaded`], simply returns the inner
	/// [`RemoteContext`].
	pub async fn load_context_with<V, L: Loader<I>>(
		self,
		vocabulary: &mut V,
		loader: &mut L,
	) -> Result<RemoteContext<I>, ContextLoadError<L::Error>>
	where
		V: IriVocabularyMut<Iri = I>,
		//
		V: Send + Sync,
		I: Send,
	{
		match self {
			Self::Iri(r) => Ok(loader
				.load_with(vocabulary, r)
				.await
				.map_err(ContextLoadError::LoadingDocumentFailed)?
				.try_map(|d| d.into_ld_context())?),
			Self::Loaded(doc) => Ok(doc),
		}
	}

	/// Loads the remote context with the given `vocabulary` and `loader`.
	///
	/// For [`Self::Iri`] returns an owned [`RemoteContext`] with
	/// [`Cow::Owned`].
	/// For [`Self::Loaded`] returns a reference to the inner [`RemoteContext`]
	/// with [`Cow::Borrowed`].
	pub async fn loaded_context_with<V, L: Loader<I>>(
		&self,
		vocabulary: &mut V,
		loader: &mut L,
	) -> Result<Cow<'_, RemoteContext<I>>, ContextLoadError<L::Error>>
	where
		V: IriVocabularyMut<Iri = I>,
		I: Clone,
		//
		V: Send + Sync,
		I: Send,
	{
		match self {
			Self::Iri(r) => Ok(Cow::Owned(
				loader
					.load_with(vocabulary, r.clone())
					.await
					.map_err(ContextLoadError::LoadingDocumentFailed)?
					.try_map(|d| d.into_ld_context())?,
			)),
			Self::Loaded(doc) => Ok(Cow::Borrowed(doc)),
		}
	}
}

/// Remote document.
///
/// Stores the content of a loaded remote document along with its original URL.
#[derive(Debug, Clone)]
pub struct RemoteDocument<I = IriBuf, T = json_syntax::Value> {
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
	document: T,
}

pub type RemoteContext<I = IriBuf> = RemoteDocument<I, json_ld_syntax::context::Context>;

impl<I, T> RemoteDocument<I, T> {
	/// Creates a new remote document.
	///
	/// `url` is the final URL of the loaded document, after eventual
	/// redirection.
	/// `content_type` is the HTTP `Content-Type` header value of the loaded
	/// document, exclusive of any optional parameters.
	pub fn new(url: Option<I>, content_type: Option<Mime>, document: T) -> Self {
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
		document: T,
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
	pub fn map<U>(self, f: impl Fn(T) -> U) -> RemoteDocument<I, U> {
		RemoteDocument {
			url: self.url,
			content_type: self.content_type,
			context_url: self.context_url,
			profile: self.profile,
			document: f(self.document),
		}
	}

	/// Tries to map the content of the remote document.
	pub fn try_map<U, E>(self, f: impl Fn(T) -> Result<U, E>) -> Result<RemoteDocument<I, U>, E> {
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
	pub fn document(&self) -> &T {
		&self.document
	}

	/// Returns a mutable reference to the content of the document.
	pub fn document_mut(&mut self) -> &mut T {
		&mut self.document
	}

	/// Drops the original URL and returns the content of the document.
	pub fn into_document(self) -> T {
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
pub trait Loader<I = IriBuf> {
	/// Error type.
	type Error;

	/// Loads the document behind the given IRI, using the given vocabulary.
	fn load_with<'a, V>(
		&'a mut self,
		vocabulary: &'a mut V,
		url: I,
	) -> BoxFuture<'a, LoadingResult<I, Self::Error>>
	where
		V: IriVocabularyMut<Iri = I>,
		//
		V: Send + Sync,
		I: 'a + Send;

	/// Loads the document behind the given IRI.
	#[allow(async_fn_in_trait)]
	async fn load(&mut self, url: I) -> Result<RemoteDocument<I>, Self::Error>
	where
		(): IriVocabulary<Iri = I>,
		//
		I: Send,
	{
		self.load_with(rdf_types::vocabulary::no_vocabulary_mut(), url)
			.await
	}
}

// /// Context document loader.
// ///
// /// This is a subclass of loaders able to extract a local context from a loaded
// /// JSON-LD document.
// ///
// /// It is implemented for any loader where the output type implements
// /// [`ExtractContext`].
// pub trait ContextLoader<I = IriBuf> {
// 	/// Error type.
// 	type ContextError;

// 	/// Loads the context behind the given IRI, using the given vocabulary.
// 	fn load_context_with<'a>(
// 		&'a mut self,
// 		vocabulary: &'a mut (impl Send + Sync + IriVocabularyMut<Iri = I>),
// 		url: I,
// 	) -> BoxFuture<'a, LoadingResult<I, M, json_ld_syntax::context::Context, Self::ContextError>>
// 	where
// 		I: 'a,
// 		M: 'a;

// 	/// Loads the context behind the given IRI.
// 	fn load_context<'a>(
// 		&'a mut self,
// 		url: I,
// 	) -> BoxFuture<'a, LoadingResult<I, M, json_ld_syntax::context::Context, Self::ContextError>>
// 	where
// 		I: 'a,
// 		M: 'a,
// 		(): IriVocabulary<Iri = I>,
// 	{
// 		self.load_context_with(rdf_types::vocabulary::no_vocabulary_mut(), url)
// 	}
// }

// /// Context extraction method.
// ///
// /// Implemented by documents containing a JSON-LD context definition, providing
// /// a method to extract this context.
// pub trait ExtractContext: Sized {
// 	/// Error type.
// 	///
// 	/// May be raised if the inner context is missing or invalid.
// 	type Error;

// 	/// Extract the context definition.
// 	fn extract_context(
// 		self,
// 	) -> Result<json_ld_syntax::context::Context, Self::Error>;
// }

/// Context extraction error.
#[derive(Debug, thiserror::Error)]
pub enum ExtractContextError {
	/// Unexpected JSON value.
	#[error("unexpected {0}")]
	Unexpected(json_syntax::Kind),

	/// No context definition found.
	#[error("missing `@context` entry")]
	NoContext,

	/// Multiple context definitions found.
	#[error("duplicate `@context` entry")]
	DuplicateContext,

	/// JSON syntax error.
	#[error("JSON-LD context syntax error: {0}")]
	Syntax(json_ld_syntax::context::InvalidContext),
}

impl ExtractContextError {
	fn duplicate_context(
		json_syntax::object::Duplicate(_, _): json_syntax::object::Duplicate<
			json_syntax::object::Entry,
		>,
	) -> Self {
		Self::DuplicateContext
	}
}

pub trait ExtractContext {
	fn into_ld_context(self) -> Result<json_ld_syntax::context::Context, ExtractContextError>;
}

impl ExtractContext for json_syntax::Value {
	fn into_ld_context(self) -> Result<json_ld_syntax::context::Context, ExtractContextError> {
		match self {
			Self::Object(mut o) => match o
				.remove_unique("@context")
				.map_err(ExtractContextError::duplicate_context)?
			{
				Some(context) => {
					use json_ld_syntax::TryFromJson;
					json_ld_syntax::context::Context::try_from_json(context.value)
						.map_err(ExtractContextError::Syntax)
				}
				None => Err(ExtractContextError::NoContext),
			},
			other => Err(ExtractContextError::Unexpected(other.kind())),
		}
	}
}
