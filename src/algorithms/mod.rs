mod compaction;
mod context_processing;
mod deserialization;
mod error;
mod expansion;
mod flattening;
mod serialization;
mod warning;

pub use compaction::*;
pub use context_processing::*;
pub use error::*;
pub use expansion::*;
pub use warning::*;

use crate::Loader;

pub trait ProcessingEnvironment {
	type Loader: Loader;

	fn loader(&self) -> &Self::Loader;

	fn warn(&self, w: Warning);

	fn as_ref(&self) -> ProcessingEnvironmentRef<'_, Self> {
		ProcessingEnvironmentRef(self)
	}
}

impl<L: Loader> ProcessingEnvironment for L {
	type Loader = Self;

	fn loader(&self) -> &Self::Loader {
		self
	}

	fn warn(&self, _: Warning) {
		// Ignore.
	}
}

pub struct ProcessingEnvironmentRef<'a, T: ?Sized>(pub &'a T);

impl<'a, T: ?Sized + ProcessingEnvironment> ProcessingEnvironment
	for ProcessingEnvironmentRef<'a, T>
{
	type Loader = T::Loader;

	fn loader(&self) -> &Self::Loader {
		self.0.loader()
	}

	fn warn(&self, w: Warning) {
		self.0.warn(w);
	}
}
