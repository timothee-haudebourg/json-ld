use core::fmt;

use crate::{LoadError, LoadErrorCause, LoadingResult};
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
	async fn load(&self, url: &Iri) -> LoadingResult<IriBuf> {
		match self.0.load(url).await {
			Ok(doc) => Ok(doc),
			Err(LoadError { cause: e1, .. }) => match self.1.load(url).await {
				Ok(doc) => Ok(doc),
				Err(LoadError { target, cause: e2 }) => Err(LoadError::new(target, Error(e1, e2))),
			},
		}
	}
}

/// Either-or error.
#[derive(Debug)]
pub struct Error(pub LoadErrorCause, pub LoadErrorCause);

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Error(e1, e2) = self;
		write!(f, "{e1}, then {e2}")
	}
}

impl std::error::Error for Error {}
