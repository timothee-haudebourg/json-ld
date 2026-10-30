use core::fmt;

use crate::{JsonLdDocument, JsonLdLoadError};
use rdf_syntax::Iri;

use super::AsyncJsonLdLoader;

/// * [`JsonLdChainLoader`]: loads document from the first loader, otherwise falls back to the second one.
///
/// This can be useful for combining, for example,
/// a [`JsonLdFsLoader`](super::JsonLdFsLoader) for loading some contexts from a local cache,
/// and a [`JsonLdReqwestLoader`](super::JsonLdReqwestLoader) for loading any other context from the web.
///
/// Note that it is also possible to nest several [`JsonLdChainLoader`]s,
/// to combine more than two loaders.
pub struct JsonLdChainLoader<L1, L2>(L1, L2);

impl<L1, L2> JsonLdChainLoader<L1, L2> {
	/// Build a new chain loader
	pub fn new(l1: L1, l2: L2) -> Self {
		JsonLdChainLoader(l1, l2)
	}
}

impl<L1, L2> AsyncJsonLdLoader for JsonLdChainLoader<L1, L2>
where
	L1: AsyncJsonLdLoader,
	L2: AsyncJsonLdLoader,
{
	async fn async_load(&self, url: &Iri) -> Result<JsonLdDocument, JsonLdLoadError> {
		match self.0.async_load(url).await {
			Ok(doc) => Ok(doc),
			Err(JsonLdLoadError { cause: e1, .. }) => match self.1.async_load(url).await {
				Ok(doc) => Ok(doc),
				Err(JsonLdLoadError { target, cause: e2 }) => {
					Err(JsonLdLoadError::new(target, Error(e1, e2)))
				}
			},
		}
	}
}

/// Either-or error.
#[derive(Debug)]
pub struct Error(pub anyhow::Error, pub anyhow::Error);

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Error(e1, e2) = self;
		write!(f, "{e1}, then {e2}")
	}
}

impl std::error::Error for Error {}
