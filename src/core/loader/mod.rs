use json_syntax::JsonValue;
use rdf_syntax::{Iri, IriBuf};
use std::{borrow::Cow, sync::LazyLock};

use crate::{JsonLdDocument, syntax::JsonLdContextDocumentValue};

pub mod chain;
pub mod fs;
pub mod map;
pub mod none;
#[cfg(feature = "reqwest")]
pub mod reqwest;
#[cfg(feature = "tokio")]
pub mod tokio_fs;

#[cfg(feature = "reqwest")]
pub use self::reqwest::JsonLdReqwestLoader;
pub use chain::JsonLdChainLoader;
pub use fs::JsonLdFsLoader;
pub use map::StaticJsonLdLoader;
pub use none::NoJsonLdLoader;
#[cfg(feature = "tokio")]
pub use tokio_fs::JsonLdTokioFsLoader;

/// Remote JSON-LD context.
pub type RemoteJsonLdContext = RemoteJsonLdDocument<JsonLdContextDocumentValue>;

/// Remote document, loaded or not.
///
/// Either an IRI or the actual document content.
#[derive(Clone)]
pub enum RemoteJsonLdDocument<T = JsonValue> {
	/// IRI to the remote document.
	Iri(IriBuf),

	/// Remote document content.
	Loaded(JsonLdDocument<T>),
}

impl<T> RemoteJsonLdDocument<T> {
	/// Creates an IRI to a `JsonValue` JSON document.
	///
	/// This method can replace `RemoteJsonLdDocument::Iri` to help the type
	/// inference in the case where `T = JsonValue`.
	pub fn iri(iri: IriBuf) -> Self {
		Self::Iri(iri)
	}
}

impl<T> RemoteJsonLdDocument<T> {
	pub fn url(&self) -> Option<&Iri> {
		match self {
			Self::Iri(iri) => Some(iri),
			Self::Loaded(t) => t.url(),
		}
	}
}

impl<T> RemoteJsonLdDocument<T>
where
	T: TryFrom<JsonValue>,
	T::Error: Into<anyhow::Error>,
{
	/// Loads the remote document with the given `loader`.
	///
	/// If the document is already [`Self::Loaded`], simply returns the inner
	/// [`JsonLdDocument`].
	pub async fn load(
		self,
		loader: &impl AsyncJsonLdLoader,
	) -> Result<JsonLdDocument<T>, JsonLdLoadError> {
		match self {
			Self::Iri(r) => loader
				.async_load(&r)
				.await?
				.try_map(TryInto::try_into)
				.map_err(|e| JsonLdLoadError::new(r, e)),
			Self::Loaded(doc) => Ok(doc),
		}
	}

	/// Loads the remote document with the given `loader`.
	///
	/// For [`Self::Iri`] returns an owned [`JsonLdDocument`] with
	/// [`Cow::Owned`].
	/// For [`Self::Loaded`] returns a reference to the inner [`JsonLdDocument`]
	/// with [`Cow::Borrowed`].
	pub async fn loaded(
		&self,
		loader: &impl AsyncJsonLdLoader,
	) -> Result<Cow<'_, JsonLdDocument<T>>, JsonLdLoadError>
	where
		T: Clone,
	{
		match self {
			Self::Iri(r) => loader
				.async_load(r)
				.await?
				.try_map(TryInto::try_into)
				.map_err(|e| JsonLdLoadError::new(r.clone(), e))
				.map(Cow::Owned),
			Self::Loaded(doc) => Ok(Cow::Borrowed(doc)),
		}
	}
}

/// Loading error.
#[derive(Debug, thiserror::Error)]
#[error("loading document `{target}` failed: {cause}")]
pub struct JsonLdLoadError {
	pub target: IriBuf,
	pub cause: anyhow::Error,
}

impl JsonLdLoadError {
	pub fn new(target: IriBuf, cause: impl Into<anyhow::Error>) -> Self {
		Self {
			target,
			cause: cause.into(),
		}
	}
}

/// Async Document loader.
///
/// A document loader is required by most processing functions to fetch remote
/// documents identified by an IRI. In particular, the loader is in charge of
/// fetching all the remote contexts imported in a `@context` entry.
///
/// This library provides a few default loader implementations:
///   - [`NoJsonLdLoader`] dummy loader that always fail. Perfect if you are
///     certain that the processing will not require any loading.
///   - Standard [`HashMap`](std::collections::HashMap) and
///     [`BTreeMap`](std::collections::BTreeMap) mapping IRIs to pre-loaded
///     documents. This way no network calls are performed and the loaded
///     content can be trusted.
///   - [`JsonLdFsLoader`] that redirecting registered IRI prefixes to a local
///     directory on the file system. This also avoids network calls. The loaded
///     content can be trusted as long as the file system is trusted.
///   - `JsonLdReqwestLoader` actually downloading the remote documents using the
///     [`reqwest`](https://crates.io/crates/reqwest) library.
///     This requires the `reqwest` feature to be enabled.
pub trait AsyncJsonLdLoader {
	/// Loads the document behind the given IRI.
	#[allow(async_fn_in_trait)]
	async fn async_load(&self, url: &Iri) -> Result<JsonLdDocument, JsonLdLoadError>;
}

impl<L: AsyncJsonLdLoader> AsyncJsonLdLoader for &L {
	async fn async_load(&self, url: &Iri) -> Result<JsonLdDocument, JsonLdLoadError> {
		L::async_load(self, url).await
	}
}

impl<L: AsyncJsonLdLoader> AsyncJsonLdLoader for &mut L {
	async fn async_load(&self, url: &Iri) -> Result<JsonLdDocument, JsonLdLoadError> {
		L::async_load(self, url).await
	}
}

/// Lazily initialized loader, e.g. a `static` map of pre-loaded documents.
impl<L: AsyncJsonLdLoader, F: FnOnce() -> L> AsyncJsonLdLoader for LazyLock<L, F> {
	async fn async_load(&self, url: &Iri) -> Result<JsonLdDocument, JsonLdLoadError> {
		L::async_load(self, url).await
	}
}

pub trait JsonLdLoader {
	/// Loads the document behind the given IRI.
	fn load(&self, url: &Iri) -> Result<JsonLdDocument, JsonLdLoadError>;

	/// Returns this loader as an [`AsyncJsonLdLoader`].
	fn as_async_loader(&self) -> &ToAsyncJsonLdLoader<Self> {
		ToAsyncJsonLdLoader::from_ref(self)
	}

	/// Turns this loader into an async loader.
	fn into_async_loader(self) -> ToAsyncJsonLdLoader<Self>
	where
		Self: Sized,
	{
		ToAsyncJsonLdLoader(self)
	}
}

impl<L: JsonLdLoader> JsonLdLoader for &L {
	fn load(&self, url: &Iri) -> Result<JsonLdDocument, JsonLdLoadError> {
		L::load(self, url)
	}
}

impl<L: JsonLdLoader> JsonLdLoader for &mut L {
	fn load(&self, url: &Iri) -> Result<JsonLdDocument, JsonLdLoadError> {
		L::load(self, url)
	}
}

/// Lazily initialized loader, e.g. a `static` map of pre-loaded documents.
impl<L: JsonLdLoader, F: FnOnce() -> L> JsonLdLoader for LazyLock<L, F> {
	fn load(&self, url: &Iri) -> Result<JsonLdDocument, JsonLdLoadError> {
		L::load(self, url)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ToAsyncJsonLdLoader<T: ?Sized>(pub T);

impl<T: ?Sized> ToAsyncJsonLdLoader<T> {
	pub fn from_ref(loader: &T) -> &Self {
		unsafe {
			// SAFETY: `ToAsyncJsonLdLoader` uses the `transparent` repr.
			std::mem::transmute(loader)
		}
	}
}

impl<T: JsonLdLoader> AsyncJsonLdLoader for ToAsyncJsonLdLoader<T> {
	async fn async_load(&self, url: &Iri) -> Result<JsonLdDocument, JsonLdLoadError> {
		self.0.load(url)
	}
}
