use super::{Loader, RemoteDocument};
use crate::{LoaderError, LoadingResult};
use iref::{Iri, IriBuf};
use std::collections::{BTreeMap, HashMap};

/// Error returned using [`HashMap`] or [`BTreeMap`] as a [`Loader`] with the
/// requested document is not found.
#[derive(Debug, thiserror::Error)]
#[error("document `{0}` not found")]
pub struct EntryNotFound(pub IriBuf);

impl LoaderError for EntryNotFound {
	fn into_iri_and_message(self) -> (IriBuf, String) {
		(self.0, "not found".to_string())
	}
}

impl Loader for HashMap<IriBuf, RemoteDocument> {
	type Error = EntryNotFound;

	async fn load(&mut self, url: &Iri) -> LoadingResult<IriBuf, Self::Error> {
		match self.get(url) {
			Some(document) => Ok(document.clone()),
			None => Err(EntryNotFound(url.to_owned())),
		}
	}
}

impl Loader for BTreeMap<IriBuf, RemoteDocument> {
	type Error = EntryNotFound;

	async fn load(&mut self, url: &Iri) -> LoadingResult<IriBuf, Self::Error> {
		match self.get(url) {
			Some(document) => Ok(document.clone()),
			None => Err(EntryNotFound(url.to_owned())),
		}
	}
}