use super::Loader;
use crate::LoadingResult;
use contextual::{DisplayWithContext, WithContext};
use rdf_types::vocabulary::IriVocabulary;
use std::fmt;

/// Dummy loader.
///
/// A dummy loader that does not load anything.
/// Can be useful when you know that you will never need to load remote resource.
///
/// Raises an `LoadingDocumentFailed` at every attempt to load a resource.
#[derive(Debug, Default)]
pub struct NoLoader;

#[derive(Debug, thiserror::Error)]
#[error("cannot load `{0}`")]
pub struct CannotLoad<I>(I);

impl<I: DisplayWithContext<N>, N> DisplayWithContext<N> for CannotLoad<I> {
	fn fmt_with(&self, vocabulary: &N, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "cannot load `{}`", self.0.with(vocabulary))
	}
}

impl<I> Loader<I> for NoLoader {
	type Error = CannotLoad<I>;

	#[inline(always)]
	async fn load_with<V>(&mut self, _vocabulary: &mut V, url: I) -> LoadingResult<I, CannotLoad<I>>
	where
		V: IriVocabulary<Iri = I>,
	{
		Err(CannotLoad(url))
	}
}
