use iref::{Iri, IriBuf};
use std::borrow::Cow;

pub mod chain;
pub mod fs;
pub mod map;
pub mod none;

pub use chain::ChainLoader;
pub use fs::FsLoader;
pub use none::NoLoader;

#[cfg(feature = "reqwest")]
pub mod reqwest;

use crate::{syntax::Context, Document};

#[cfg(feature = "reqwest")]
pub use self::reqwest::ReqwestLoader;

pub type RemoteContextReference = RemoteDocumentReference<Context>;

/// Remote document, loaded or not.
///
/// Either an IRI or the actual document content.
#[derive(Clone)]
pub enum RemoteDocumentReference<T = json_syntax::Value> {
	/// IRI to the remote document.
	Iri(IriBuf),

	/// Remote document content.
	Loaded(Document<T>),
}

impl<T> RemoteDocumentReference<T> {
	/// Creates an IRI to a `json_syntax::Value` JSON document.
	///
	/// This method can replace `RemoteDocumentReference::Iri` to help the type
	/// inference in the case where `T = json_syntax::Value`.
	pub fn iri(iri: IriBuf) -> Self {
		Self::Iri(iri)
	}
}

impl RemoteDocumentReference {
	/// Loads the remote document with the given `loader`.
	///
	/// If the document is already [`Self::Loaded`], simply returns the inner
	/// [`RemoteDocument`].
	pub async fn load_with<V>(self, loader: &impl Loader) -> Result<Document, LoadError> {
		match self {
			Self::Iri(r) => Ok(loader.load(&r).await?.map(Into::into)),
			Self::Loaded(doc) => Ok(doc),
		}
	}

	/// Loads the remote document with the given `loader`.
	///
	/// For [`Self::Iri`] returns an owned [`RemoteDocument`] with
	/// [`Cow::Owned`].
	/// For [`Self::Loaded`] returns a reference to the inner [`RemoteDocument`]
	/// with [`Cow::Borrowed`].
	pub async fn loaded_with(&self, loader: &impl Loader) -> Result<Cow<'_, Document>, LoadError> {
		match self {
			Self::Iri(r) => Ok(Cow::Owned(loader.load(r).await?.map(Into::into))),
			Self::Loaded(doc) => Ok(Cow::Borrowed(doc)),
		}
	}
}

// #[derive(Debug, thiserror::Error)]
// pub enum ContextLoadError {
// 	#[error(transparent)]
// 	LoadingDocumentFailed(#[from] LoadError),

// 	#[error("context extraction failed")]
// 	ContextExtractionFailed(#[from] ExtractContextError),
// }

// impl<I> RemoteContextReference<I> {
// 	/// Loads the remote context with the given `vocabulary` and `loader`.
// 	///
// 	/// If the context is already [`Self::Loaded`], simply returns the inner
// 	/// [`RemoteContext`].
// 	pub async fn load_context_with<V, L: Loader>(
// 		self,
// 		vocabulary: &mut V,
// 		loader: &L,
// 	) -> Result<RemoteContext<I>, ContextLoadError>
// 	where
// 		V: IriVocabularyMut<Iri = I>,
// 		I: Clone + Eq + Hash,
// 	{
// 		match self {
// 			Self::Iri(r) => Ok(loader
// 				.load_with(vocabulary, r)
// 				.await?
// 				.try_map(|d| d.into_ld_context())?),
// 			Self::Loaded(doc) => Ok(doc),
// 		}
// 	}

// 	/// Loads the remote context with the given `vocabulary` and `loader`.
// 	///
// 	/// For [`Self::Iri`] returns an owned [`RemoteContext`] with
// 	/// [`Cow::Owned`].
// 	/// For [`Self::Loaded`] returns a reference to the inner [`RemoteContext`]
// 	/// with [`Cow::Borrowed`].
// 	pub async fn loaded_context_with<V, L: Loader>(
// 		&self,
// 		vocabulary: &mut V,
// 		loader: &L,
// 	) -> Result<Cow<'_, RemoteContext<I>>, ContextLoadError>
// 	where
// 		V: IriVocabularyMut<Iri = I>,
// 		I: Clone + Eq + Hash,
// 	{
// 		match self {
// 			Self::Iri(r) => Ok(Cow::Owned(
// 				loader
// 					.load_with(vocabulary, r.clone())
// 					.await?
// 					.try_map(|d| d.into_ld_context())?,
// 			)),
// 			Self::Loaded(doc) => Ok(Cow::Borrowed(doc)),
// 		}
// 	}
// }

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
	/// Loads the document behind the given IRI.
	#[allow(async_fn_in_trait)]
	async fn load(&self, url: &Iri) -> Result<Document, LoadError>;
}

impl<'l, L: Loader> Loader for &'l mut L {
	async fn load(&self, url: &Iri) -> Result<Document, LoadError> {
		L::load(self, url).await
	}
}

// /// Context extraction error.
// #[derive(Debug, thiserror::Error)]
// pub enum ExtractContextError {
// 	/// Unexpected JSON value.
// 	#[error("unexpected {0}")]
// 	Unexpected(json_syntax::Kind),

// 	/// No context definition found.
// 	#[error("missing `@context` entry")]
// 	NoContext,

// 	/// Multiple context definitions found.
// 	#[error("duplicate `@context` entry")]
// 	DuplicateContext,

// 	/// JSON syntax error.
// 	#[error("JSON-LD context syntax error: {0}")]
// 	Syntax(InvalidContext),
// }

// impl ExtractContextError {
// 	fn duplicate_context(
// 		json_syntax::object::Duplicate(_, _): json_syntax::object::Duplicate<
// 			json_syntax::object::Entry,
// 		>,
// 	) -> Self {
// 		Self::DuplicateContext
// 	}
// }

// pub trait ExtractContext {
// 	fn into_ld_context(self) -> Result<json_ld_syntax::context::Context, ExtractContextError>;
// }

// impl ExtractContext for json_syntax::Value {
// 	fn into_ld_context(self) -> Result<json_ld_syntax::context::Context, ExtractContextError> {
// 		match self {
// 			Self::Object(mut o) => match o
// 				.remove_unique("@context")
// 				.map_err(ExtractContextError::duplicate_context)?
// 			{
// 				Some(context) => {
// 					use json_ld_syntax::TryFromJson;
// 					json_ld_syntax::context::Context::try_from_json(context.value)
// 						.map_err(ExtractContextError::Syntax)
// 				}
// 				None => Err(ExtractContextError::NoContext),
// 			},
// 			other => Err(ExtractContextError::Unexpected(other.kind())),
// 		}
// 	}
// }
