use rdf_syntax::Iri;

use crate::{JsonLdDocument, JsonLdLoadError};

use super::{AsyncJsonLdLoader, JsonLdLoader};

/// Dummy loader.
///
/// A dummy loader that does not load anything.
/// Can be useful when you know that you will never need to load remote resource.
///
/// Raises an `LoadingDocumentFailed` at every attempt to load a resource.
#[derive(Debug, Default)]
pub struct NoJsonLdLoader;

#[derive(Debug, thiserror::Error)]
#[error("no loader")]
pub struct CannotLoad;

impl JsonLdLoader for NoJsonLdLoader {
	#[inline(always)]
	fn load(&self, url: &Iri) -> Result<JsonLdDocument, JsonLdLoadError> {
		Err(JsonLdLoadError::new(url.to_owned(), CannotLoad))
	}
}

impl AsyncJsonLdLoader for NoJsonLdLoader {
	#[inline(always)]
	async fn async_load(&self, url: &Iri) -> Result<JsonLdDocument, JsonLdLoadError> {
		Err(JsonLdLoadError::new(url.to_owned(), CannotLoad))
	}
}
