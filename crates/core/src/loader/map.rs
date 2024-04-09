use super::{Loader, RemoteDocument};
use crate::LoadingResult;
use rdf_types::vocabulary::IriVocabulary;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

/// Error returned using [`HashMap`] or [`BTreeMap`] as a [`Loader`] with the
/// requested document is not found.
#[derive(Debug, thiserror::Error)]
#[error("document `{0}` not found")]
pub struct EntryNotFound<I>(pub I);

impl<I: Clone + Eq + Hash> Loader<I> for HashMap<I, RemoteDocument<I>> {
	type Error = EntryNotFound<I>;

	async fn load_with<V>(&mut self, _vocabulary: &mut V, url: I) -> LoadingResult<I, Self::Error>
	where
		V: IriVocabulary<Iri = I>,
	{
		match self.get(&url) {
			Some(document) => Ok(document.clone()),
			None => Err(EntryNotFound(url)),
		}
	}
}

impl<I: Clone + Ord> Loader<I> for BTreeMap<I, RemoteDocument<I>> {
	type Error = EntryNotFound<I>;

	async fn load_with<V>(&mut self, _vocabulary: &mut V, url: I) -> LoadingResult<I, Self::Error>
	where
		V: IriVocabulary<Iri = I>,
	{
		match self.get(&url) {
			Some(document) => Ok(document.clone()),
			None => Err(EntryNotFound(url)),
		}
	}
}
