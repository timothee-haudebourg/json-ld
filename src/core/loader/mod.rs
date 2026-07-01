use json_syntax::JsonValue;
use rdf_syntax::{Iri, IriBuf};
use std::borrow::Cow;

use crate::{Document, syntax::ContextDocumentValue};

pub mod chain;
pub mod fs;
pub mod map;
pub mod none;
#[cfg(feature = "reqwest")]
pub mod reqwest;
#[cfg(feature = "tokio")]
pub mod tokio_fs;

#[cfg(feature = "reqwest")]
pub use self::reqwest::ReqwestLoader;
pub use chain::ChainLoader;
pub use fs::FsLoader;
pub use none::NoLoader;
#[cfg(feature = "tokio")]
pub use tokio_fs::TokioFsLoader;

pub type RemoteContext = RemoteDocument<ContextDocumentValue>;

/// Remote document, loaded or not.
///
/// Either an IRI or the actual document content.
#[derive(Clone)]
pub enum RemoteDocument<T = JsonValue> {
	/// IRI to the remote document.
	Iri(IriBuf),

	/// Remote document content.
	Loaded(Document<T>),
}

impl<T> RemoteDocument<T> {
	/// Creates an IRI to a `JsonValue` JSON document.
	///
	/// This method can replace `RemoteDocumentReference::Iri` to help the type
	/// inference in the case where `T = JsonValue`.
	pub fn iri(iri: IriBuf) -> Self {
		Self::Iri(iri)
	}
}

impl<T> RemoteDocument<T> {
	pub fn url(&self) -> Option<&Iri> {
		match self {
			Self::Iri(iri) => Some(iri),
			Self::Loaded(t) => t.url(),
		}
	}
}

impl<T> RemoteDocument<T>
where
	T: TryFrom<JsonValue>,
	T::Error: Into<anyhow::Error>,
{
	/// Loads the remote document with the given `loader`.
	///
	/// If the document is already [`Self::Loaded`], simply returns the inner
	/// [`RemoteDocument`].
	pub async fn load(self, loader: &impl AsyncLoader) -> Result<Document<T>, LoadError> {
		match self {
			Self::Iri(r) => loader
				.async_load(&r)
				.await?
				.try_map(TryInto::try_into)
				.map_err(|e| LoadError::new(r, e)),
			Self::Loaded(doc) => Ok(doc),
		}
	}

	/// Loads the remote document with the given `loader`.
	///
	/// For [`Self::Iri`] returns an owned [`RemoteDocument`] with
	/// [`Cow::Owned`].
	/// For [`Self::Loaded`] returns a reference to the inner [`RemoteDocument`]
	/// with [`Cow::Borrowed`].
	pub async fn loaded(&self, loader: &impl AsyncLoader) -> Result<Cow<'_, Document<T>>, LoadError>
	where
		T: Clone,
	{
		match self {
			Self::Iri(r) => loader
				.async_load(r)
				.await?
				.try_map(TryInto::try_into)
				.map_err(|e| LoadError::new(r.clone(), e))
				.map(Cow::Owned),
			Self::Loaded(doc) => Ok(Cow::Borrowed(doc)),
		}
	}
}

/// Loading error.
#[derive(Debug, thiserror::Error)]
#[error("loading document `{target}` failed: {cause}")]
pub struct LoadError {
	pub target: IriBuf,
	pub cause: anyhow::Error,
}

impl LoadError {
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
pub trait AsyncLoader {
	/// Loads the document behind the given IRI.
	#[allow(async_fn_in_trait)]
	async fn async_load(&self, url: &Iri) -> Result<Document, LoadError>;
}

impl<L: AsyncLoader> AsyncLoader for &L {
	async fn async_load(&self, url: &Iri) -> Result<Document, LoadError> {
		L::async_load(self, url).await
	}
}

impl<L: AsyncLoader> AsyncLoader for &mut L {
	async fn async_load(&self, url: &Iri) -> Result<Document, LoadError> {
		L::async_load(self, url).await
	}
}

pub trait Loader {
	/// Loads the document behind the given IRI.
	fn load(&self, url: &Iri) -> Result<Document, LoadError>;

	/// Returns this loader as an [`AsyncLoader`].
	fn as_async_loader(&self) -> &ToAsyncLoader<Self> {
		ToAsyncLoader::from_ref(self)
	}

	/// Turns this loader into an async loader.
	fn into_async_loader(self) -> ToAsyncLoader<Self>
	where
		Self: Sized,
	{
		ToAsyncLoader(self)
	}
}

impl<L: Loader> Loader for &L {
	fn load(&self, url: &Iri) -> Result<Document, LoadError> {
		L::load(self, url)
	}
}

impl<L: Loader> Loader for &mut L {
	fn load(&self, url: &Iri) -> Result<Document, LoadError> {
		L::load(self, url)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ToAsyncLoader<T: ?Sized>(pub T);

impl<T: ?Sized> ToAsyncLoader<T> {
	pub fn from_ref(loader: &T) -> &Self {
		unsafe {
			// SAFETY: `ToAsyncLoader` uses the `transparent` repr.
			std::mem::transmute(loader)
		}
	}
}

impl<T: Loader> AsyncLoader for ToAsyncLoader<T> {
	async fn async_load(&self, url: &Iri) -> Result<Document, LoadError> {
		self.0.load(url)
	}
}
