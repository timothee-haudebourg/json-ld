use rdf_syntax::{Iri, IriBuf};
use std::collections::{BTreeMap, HashMap};

use crate::{Document, LoadError};

use super::{AsyncLoader, Loader};

/// Error returned using [`HashMap`] or [`BTreeMap`] as a [`Loader`] with the
/// requested document is not found.
#[derive(Debug, thiserror::Error)]
#[error("document not found")]
pub struct EntryNotFound;

impl Loader for HashMap<IriBuf, Document> {
	fn load(&self, url: &Iri) -> Result<Document, LoadError> {
		match self.get(url) {
			Some(document) => Ok(document.clone()),
			None => Err(LoadError::new(url.to_owned(), EntryNotFound)),
		}
	}
}

impl AsyncLoader for HashMap<IriBuf, Document> {
	async fn async_load(&self, url: &Iri) -> Result<Document, LoadError> {
		match self.get(url) {
			Some(document) => Ok(document.clone()),
			None => Err(LoadError::new(url.to_owned(), EntryNotFound)),
		}
	}
}

impl Loader for BTreeMap<IriBuf, Document> {
	fn load(&self, url: &Iri) -> Result<Document, LoadError> {
		match self.get(url) {
			Some(document) => Ok(document.clone()),
			None => Err(LoadError::new(url.to_owned(), EntryNotFound)),
		}
	}
}

impl AsyncLoader for BTreeMap<IriBuf, Document> {
	async fn async_load(&self, url: &Iri) -> Result<Document, LoadError> {
		match self.get(url) {
			Some(document) => Ok(document.clone()),
			None => Err(LoadError::new(url.to_owned(), EntryNotFound)),
		}
	}
}
