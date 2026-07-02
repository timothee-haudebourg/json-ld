use rdf_syntax::Iri;

use crate::{JsonLdDocument, LoadError};

use super::{AsyncLoader, Loader};

/// Dummy loader.
///
/// A dummy loader that does not load anything.
/// Can be useful when you know that you will never need to load remote resource.
///
/// Raises an `LoadingDocumentFailed` at every attempt to load a resource.
#[derive(Debug, Default)]
pub struct NoLoader;

#[derive(Debug, thiserror::Error)]
#[error("no loader")]
pub struct CannotLoad;

impl Loader for NoLoader {
	#[inline(always)]
	fn load(&self, url: &Iri) -> Result<JsonLdDocument, LoadError> {
		Err(LoadError::new(url.to_owned(), CannotLoad))
	}
}

impl AsyncLoader for NoLoader {
	#[inline(always)]
	async fn async_load(&self, url: &Iri) -> Result<JsonLdDocument, LoadError> {
		Err(LoadError::new(url.to_owned(), CannotLoad))
	}
}
