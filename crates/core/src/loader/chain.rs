use crate::LoadingResult;
use rdf_types::vocabulary::IriVocabularyMut;
use std::fmt;

use super::Loader;

/// * [`ChainLoader`]: loads document from the first loader, otherwise falls back to the second one.
///
/// This can be useful for combining, for example,
/// an [`FsLoader`](super::FsLoader) for loading some contexts from a local cache,
/// and a [`ReqwestLoader`](super::ReqwestLoader) for loading any other context from the web.
///
/// Note that it is also possible to nest several [`ChainLoader`]s,
/// to combine more than two loaders.
pub struct ChainLoader<L1, L2>(L1, L2);

impl<L1, L2> ChainLoader<L1, L2> {
	/// Build a new chain loader
	pub fn new(l1: L1, l2: L2) -> Self {
		ChainLoader(l1, l2)
	}
}

impl<I, L1, L2> Loader<I> for ChainLoader<L1, L2>
where
	I: Clone + Send + Sync,
	L1: Loader<I> + Send,
	L2: Loader<I> + Send,
	L1::Error: Send,
{
	type Error = Error<L1::Error, L2::Error>;

	async fn load_with<V>(&mut self, vocabulary: &mut V, url: I) -> LoadingResult<I, Self::Error>
	where
		V: IriVocabularyMut<Iri = I>,
	{
		match self.0.load_with(vocabulary, url.clone()).await {
			Ok(doc) => Ok(doc),
			Err(err1) => match self.1.load_with(vocabulary, url).await {
				Ok(doc) => Ok(doc),
				Err(err2) => Err(Error(err1, err2)),
			},
		}
	}
}

/// Either-or error.
#[derive(Debug)]
pub struct Error<E1, E2>(E1, E2);

impl<E1: fmt::Display, E2: fmt::Display> fmt::Display for Error<E1, E2> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Error(e1, e2) = self;
		write!(f, "First: {e1} / Second: {e2}")
	}
}

impl<E1: std::error::Error, E2: std::error::Error> std::error::Error for Error<E1, E2> {}
