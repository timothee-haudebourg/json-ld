use super::{Loader, RemoteDocument};
use crate::{LoadError, LoadingResult};
use iref::{Iri, IriBuf};
use std::collections::{BTreeMap, HashMap};

/// Error returned using [`HashMap`] or [`BTreeMap`] as a [`Loader`] with the
/// requested document is not found.
#[derive(Debug, thiserror::Error)]
#[error("document not found")]
pub struct EntryNotFound;

impl Loader for HashMap<IriBuf, RemoteDocument> {
	async fn load(&self, url: &Iri) -> LoadingResult<IriBuf> {
		match self.get(url) {
			Some(document) => Ok(document.clone()),
			None => Err(LoadError::new(url.to_owned(), EntryNotFound)),
		}
	}
}

impl Loader for BTreeMap<IriBuf, RemoteDocument> {
	async fn load(&self, url: &Iri) -> LoadingResult<IriBuf> {
		match self.get(url) {
			Some(document) => Ok(document.clone()),
			None => Err(LoadError::new(url.to_owned(), EntryNotFound)),
		}
	}
}
