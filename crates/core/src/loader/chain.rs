use core::fmt;

use crate::{LoaderError, LoadingResult};
use iref::{Iri, IriBuf};

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

impl<L1, L2> Loader for ChainLoader<L1, L2>
where
	L1: Loader,
	L2: Loader,
{
	type Error = Error<L1::Error, L2::Error>;

	async fn load(&mut self, url: &Iri) -> LoadingResult<IriBuf, Self::Error> {
		match self.0.load(url).await {
			Ok(doc) => Ok(doc),
			Err(err1) => match self.1.load(url).await {
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
		write!(f, "{e1}, then {e2}")
	}
}

impl<E1: std::error::Error, E2: std::error::Error> std::error::Error for Error<E1, E2> {}

impl<E1: LoaderError, E2: LoaderError> LoaderError for Error<E1, E2> {
	fn into_iri_and_message(self) -> (IriBuf, String) {
		let (iri, m1) = self.0.into_iri_and_message();
		let (_, m2) = self.1.into_iri_and_message();

		(iri, format!("{m1}, then {m2}"))
	}
}
