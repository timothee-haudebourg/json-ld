use hashbrown::HashSet;
use iref::{Iri, IriBuf};
use mime::Mime;
use rdf_types::vocabulary::{IriVocabulary, IriVocabularyMut};
use static_iref::iri;
use std::{borrow::Cow, hash::Hash};

pub mod chain;
pub mod fs;
pub mod map;
pub mod none;

pub use chain::ChainLoader;
pub use fs::FsLoader;
pub use none::NoLoader;

#[cfg(feature = "reqwest")]
pub mod reqwest;

#[cfg(feature = "reqwest")]
pub use self::reqwest::ReqwestLoader;

pub type LoadingResult<I = IriBuf> = Result<RemoteDocument<I>, LoadError>;

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
	pub async fn load_with<V>(self, vocabulary: &mut V, loader: &impl Loader) -> LoadingResult<I>
	where
		V: IriVocabularyMut<Iri = I>,
		I: Clone + Eq + Hash,
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
	pub async fn loaded_with<V>(
		&self,
		vocabulary: &mut V,
		loader: &impl Loader,
	) -> Result<Cow<'_, RemoteDocument<V::Iri>>, LoadError>
	where
		V: IriVocabularyMut<Iri = I>,
		I: Clone + Eq + Hash,
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
pub enum ContextLoadError {
	#[error(transparent)]
	LoadingDocumentFailed(#[from] LoadError),

	#[error("context extraction failed")]
	ContextExtractionFailed(#[from] ExtractContextError),
}

impl<I> RemoteContextReference<I> {
	/// Loads the remote context with the given `vocabulary` and `loader`.
	///
	/// If the context is already [`Self::Loaded`], simply returns the inner
	/// [`RemoteContext`].
	pub async fn load_context_with<V, L: Loader>(
		self,
		vocabulary: &mut V,
		loader: &L,
	) -> Result<RemoteContext<I>, ContextLoadError>
	where
		V: IriVocabularyMut<Iri = I>,
		I: Clone + Eq + Hash,
	{
		match self {
			Self::Iri(r) => Ok(loader
				.load_with(vocabulary, r)
				.await?
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
	pub async fn loaded_context_with<V, L: Loader>(
		&self,
		vocabulary: &mut V,
		loader: &L,
	) -> Result<Cow<'_, RemoteContext<I>>, ContextLoadError>
	where
		V: IriVocabularyMut<Iri = I>,
		I: Clone + Eq + Hash,
	{
		match self {
			Self::Iri(r) => Ok(Cow::Owned(
				loader
					.load_with(vocabulary, r.clone())
					.await?
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
	pub url: Option<I>,

	/// The HTTP `Content-Type` header value of the loaded document, exclusive
	/// of any optional parameters.
	pub content_type: Option<Mime>,

	/// If available, the value of the HTTP `Link Header` [RFC 8288] using the
	/// `http://www.w3.org/ns/json-ld#context` link relation in the response.
	///
	/// If the response's `Content-Type` is `application/ld+json`, the HTTP
	/// `Link Header` is ignored. If multiple HTTP `Link Headers` using the
	/// `http://www.w3.org/ns/json-ld#context` link relation are found, the
	/// loader fails with a `multiple context link headers` error.
	///
	/// [RFC 8288]: https://www.rfc-editor.org/rfc/rfc8288
	pub context_url: Option<I>,

	pub profile: HashSet<Profile<I>>,

	/// The retrieved document.
	pub document: T,
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

	/// Maps all the IRIs.
	pub fn map_iris<J>(self, mut f: impl FnMut(I) -> J) -> RemoteDocument<J, T>
	where
		J: Eq + Hash,
	{
		RemoteDocument {
			url: self.url.map(&mut f),
			content_type: self.content_type,
			context_url: self.context_url.map(&mut f),
			profile: self
				.profile
				.into_iter()
				.map(|p| p.map_iri(&mut f))
				.collect(),
			document: self.document,
		}
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
pub enum Profile<I = IriBuf> {
	Standard(StandardProfile),
	Custom(I),
}

impl Profile {
	pub fn new(iri: &Iri) -> Self {
		match StandardProfile::from_iri(iri) {
			Some(p) => Self::Standard(p),
			None => Self::Custom(iri.to_owned()),
		}
	}

	pub fn iri(&self) -> &Iri {
		match self {
			Self::Standard(s) => s.iri(),
			Self::Custom(c) => c,
		}
	}
}

impl<I> Profile<I> {
	pub fn new_with(iri: &Iri, vocabulary: &mut impl IriVocabularyMut<Iri = I>) -> Self {
		match StandardProfile::from_iri(iri) {
			Some(p) => Self::Standard(p),
			None => Self::Custom(vocabulary.insert(iri)),
		}
	}

	pub fn iri_with<'a>(&'a self, vocabulary: &'a impl IriVocabulary<Iri = I>) -> &'a Iri {
		match self {
			Self::Standard(s) => s.iri(),
			Self::Custom(c) => vocabulary.iri(c).unwrap(),
		}
	}

	pub fn map_iri<J>(self, f: impl FnOnce(I) -> J) -> Profile<J> {
		match self {
			Self::Standard(p) => Profile::Standard(p),
			Self::Custom(i) => Profile::Custom(f(i)),
		}
	}
}

pub type LoadErrorCause = Box<dyn std::error::Error + Send + Sync>;

/// Loading error.
#[derive(Debug, thiserror::Error)]
#[error("loading document `{target}` failed: {cause}")]
pub struct LoadError {
	pub target: IriBuf,
	pub cause: LoadErrorCause,
}

impl LoadError {
	pub fn new(target: IriBuf, cause: impl 'static + std::error::Error + Send + Sync) -> Self {
		Self {
			target,
			cause: Box::new(cause),
		}
	}
}

/// Document loader.
///
/// A document loader is required by most processing functions to fetch remote
/// documents identified by an IRI. In particular, the loader is in charge of
/// fetching all the remote contexts imported in a `@context` entry.
///
/// This library provides a few default loader implementations:
///   - [`NoLoader`] dummy loader that always fail. Perfect if you are certain
///     that the processing will not require any loading.
///   - Standard [`HashMap`](std::collection::HashMap) and
///     [`BTreeMap`](std::collection::BTreeMap) mapping IRIs to pre-loaded
///     documents. This way no network calls are performed and the loaded
///     content can be trusted.
///   - [`FsLoader`] that redirecting registered IRI prefixes to a local
///     directory on the file system. This also avoids network calls. The loaded
///     content can be trusted as long as the file system is trusted.
///   - `ReqwestLoader` actually downloading the remote documents using the
///     [`reqwest`](https://crates.io/crates/reqwest) library.
///     This requires the `reqwest` feature to be enabled.
pub trait Loader {
	/// Loads the document behind the given IRI, using the given vocabulary.
	#[allow(async_fn_in_trait)]
	async fn load_with<V>(&self, vocabulary: &mut V, url: V::Iri) -> LoadingResult<V::Iri>
	where
		V: IriVocabularyMut,
		V::Iri: Clone + Eq + Hash,
	{
		let lexical_url = vocabulary.iri(&url).unwrap();
		let document = self.load(lexical_url).await?;
		Ok(document.map_iris(|i| vocabulary.insert_owned(i)))
	}

	/// Loads the document behind the given IRI.
	#[allow(async_fn_in_trait)]
	async fn load(&self, url: &Iri) -> Result<RemoteDocument<IriBuf>, LoadError>;
}

impl<'l, L: Loader> Loader for &'l L {
	async fn load_with<V>(&self, vocabulary: &mut V, url: V::Iri) -> LoadingResult<V::Iri>
	where
		V: IriVocabularyMut,
		V::Iri: Clone + Eq + Hash,
	{
		L::load_with(self, vocabulary, url).await
	}

	async fn load(&self, url: &Iri) -> Result<RemoteDocument<IriBuf>, LoadError> {
		L::load(self, url).await
	}
}

impl<'l, L: Loader> Loader for &'l mut L {
	async fn load_with<V>(&self, vocabulary: &mut V, url: V::Iri) -> LoadingResult<V::Iri>
	where
		V: IriVocabularyMut,
		V::Iri: Clone + Eq + Hash,
	{
		L::load_with(self, vocabulary, url).await
	}

	async fn load(&self, url: &Iri) -> Result<RemoteDocument<IriBuf>, LoadError> {
		L::load(self, url).await
	}
}

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
