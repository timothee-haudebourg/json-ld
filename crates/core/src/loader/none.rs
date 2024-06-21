use super::Loader;
use crate::{LoaderError, LoadingResult};
use iref::IriBuf;
use rdf_types::vocabulary::IriVocabulary;

/// Dummy loader.
///
/// A dummy loader that does not load anything.
/// Can be useful when you know that you will never need to load remote resource.
///
/// Raises an `LoadingDocumentFailed` at every attempt to load a resource.
#[derive(Debug, Default)]
pub struct NoLoader;

#[derive(Debug, thiserror::Error)]
#[error("no loader for `{0}`")]
pub struct CannotLoad(pub IriBuf);

impl LoaderError for CannotLoad {
	fn into_iri_and_message(self) -> (IriBuf, String) {
		(self.0, "no loader".to_string())
	}
}

impl<I> Loader<I> for NoLoader {
	type Error = CannotLoad;

	#[inline(always)]
	async fn load_with<V>(&mut self, vocabulary: &mut V, url: I) -> LoadingResult<I, CannotLoad>
	where
		V: IriVocabulary<Iri = I>,
	{
		Err(CannotLoad(vocabulary.owned_iri(url).ok().unwrap()))
	}
}
