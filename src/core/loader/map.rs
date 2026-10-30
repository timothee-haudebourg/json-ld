use rdf_syntax::{Iri, IriBuf};
use std::{
	collections::{BTreeMap, HashMap},
	sync::LazyLock,
};

use crate::{JsonLdDocument, JsonLdLoadError};

use super::{AsyncJsonLdLoader, JsonLdLoader};

/// Lazily initialized map of pre-loaded documents, suitable for a `static`.
///
/// It implements [`JsonLdLoader`] and [`AsyncJsonLdLoader`], and `&StaticJsonLdLoader`
/// can be used wherever a loader is expected.
///
/// ```
/// use json_ld::{AsyncJsonLdLoader, StaticJsonLdLoader};
/// use std::{collections::HashMap, sync::LazyLock};
///
/// static LOADER: StaticJsonLdLoader = LazyLock::new(HashMap::new);
/// # fn is_loader(_: &impl AsyncJsonLdLoader) {}
/// # is_loader(&LOADER);
/// ```
pub type StaticJsonLdLoader = LazyLock<HashMap<IriBuf, JsonLdDocument>>;

/// Error returned using [`HashMap`] or [`BTreeMap`] as a [`JsonLdLoader`] with the
/// requested document is not found.
#[derive(Debug, thiserror::Error)]
#[error("document not found")]
pub struct EntryNotFound;

impl JsonLdLoader for HashMap<IriBuf, JsonLdDocument> {
	fn load(&self, url: &Iri) -> Result<JsonLdDocument, JsonLdLoadError> {
		match self.get(url) {
			Some(document) => Ok(document.clone()),
			None => Err(JsonLdLoadError::new(url.to_owned(), EntryNotFound)),
		}
	}
}

impl AsyncJsonLdLoader for HashMap<IriBuf, JsonLdDocument> {
	async fn async_load(&self, url: &Iri) -> Result<JsonLdDocument, JsonLdLoadError> {
		match self.get(url) {
			Some(document) => Ok(document.clone()),
			None => Err(JsonLdLoadError::new(url.to_owned(), EntryNotFound)),
		}
	}
}

impl JsonLdLoader for BTreeMap<IriBuf, JsonLdDocument> {
	fn load(&self, url: &Iri) -> Result<JsonLdDocument, JsonLdLoadError> {
		match self.get(url) {
			Some(document) => Ok(document.clone()),
			None => Err(JsonLdLoadError::new(url.to_owned(), EntryNotFound)),
		}
	}
}

impl AsyncJsonLdLoader for BTreeMap<IriBuf, JsonLdDocument> {
	async fn async_load(&self, url: &Iri) -> Result<JsonLdDocument, JsonLdLoadError> {
		match self.get(url) {
			Some(document) => Ok(document.clone()),
			None => Err(JsonLdLoadError::new(url.to_owned(), EntryNotFound)),
		}
	}
}
