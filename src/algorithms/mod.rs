mod compaction;
pub mod context_processing;
mod error;
pub mod expansion;
mod warning;

pub use error::*;
pub use warning::*;

use crate::Loader;

pub trait ProcessingEnvironment {
	type Loader: Loader;

	fn loader_mut(&mut self) -> &mut Self::Loader;

	fn warn(&mut self, w: Warning);
}

impl<L: Loader> ProcessingEnvironment for L {
	type Loader = Self;

	fn loader_mut(&mut self) -> &mut Self::Loader {
		self
	}

	fn warn(&mut self, _: Warning) {
		// Ignore.
	}
}

pub struct ProcessingEnvironmentRefMut<'a, T>(pub &'a mut T);

impl<'a, T: ProcessingEnvironment> ProcessingEnvironment for ProcessingEnvironmentRefMut<'a, T> {
	type Loader = T::Loader;

	fn loader_mut(&mut self) -> &mut Self::Loader {
		self.0.loader_mut()
	}

	fn warn(&mut self, w: Warning) {
		self.0.warn(w);
	}
}
