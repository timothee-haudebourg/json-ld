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
pub use deserialization::RdfSerializationOptions;
pub use error::*;
pub use expansion::*;
use json_syntax::tracing::{IntoOwned, JsonFragmentStack, JsonLocated};
use rdf_syntax::{Iri, IriBuf};
pub use warning::*;

use crate::{AsyncLoader, Loader, ToAsyncLoader};

pub trait AsyncProcessingEnvironment {
	type Loader: AsyncLoader;

	fn loader(&self) -> &Self::Loader;

	fn warn(&self, w: Warning);

	fn as_ref(&self) -> AsyncProcessingEnvironmentRef<'_, Self> {
		AsyncProcessingEnvironmentRef(self)
	}
}

impl<L: AsyncLoader> AsyncProcessingEnvironment for L {
	type Loader = Self;

	fn loader(&self) -> &Self::Loader {
		self
	}

	fn warn(&self, _: Warning) {
		// Ignore.
	}
}

pub struct AsyncProcessingEnvironmentRef<'a, T: ?Sized>(pub &'a T);

impl<'a, T: ?Sized + AsyncProcessingEnvironment> AsyncProcessingEnvironment
	for AsyncProcessingEnvironmentRef<'a, T>
{
	type Loader = T::Loader;

	fn loader(&self) -> &Self::Loader {
		self.0.loader()
	}

	fn warn(&self, w: Warning) {
		self.0.warn(w);
	}
}

pub trait ProcessingEnvironment {
	type Loader: Loader;

	fn loader(&self) -> &Self::Loader;

	fn warn(&self, w: Warning);

	fn as_async_environment(&self) -> &ToAsyncProcessingEnvironment<Self> {
		ToAsyncProcessingEnvironment::from_ref(self)
	}

	fn into_async_environment(self) -> ToAsyncProcessingEnvironment<Self>
	where
		Self: Sized,
	{
		ToAsyncProcessingEnvironment(self)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ToAsyncProcessingEnvironment<T: ?Sized>(pub T);

impl<T: ?Sized> ToAsyncProcessingEnvironment<T> {
	pub fn from_ref(env: &T) -> &Self {
		unsafe {
			// SAFETY: `ToAsyncProcessingEnvironment` uses the `transparent` repr.
			std::mem::transmute(env)
		}
	}
}

impl<T: ProcessingEnvironment> AsyncProcessingEnvironment for ToAsyncProcessingEnvironment<T> {
	type Loader = ToAsyncLoader<T::Loader>;

	fn loader(&self) -> &Self::Loader {
		self.0.loader().as_async_loader()
	}

	fn warn(&self, w: Warning) {
		self.0.warn(w);
	}
}

/// Location stack for context-processing and expansion algorithms.
pub type JsonLdLocationStack<'a> = JsonFragmentStack<'a, JsonLdSourceRef<'a>>;

pub type JsonLdLocated<T> = JsonLocated<T, JsonLdSource>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JsonLdSourceRef<'a> {
	Compact(Option<&'a Iri>),
	Expanded(Option<&'a Iri>),
	Context(Option<&'a Iri>),
}

impl IntoOwned for JsonLdSourceRef<'_> {
	type Owned = JsonLdSource;

	fn into_owned(self) -> JsonLdSource {
		match self {
			Self::Compact(iri) => JsonLdSource::Compact(iri.map(Iri::to_owned)),
			Self::Expanded(iri) => JsonLdSource::Expanded(iri.map(Iri::to_owned)),
			Self::Context(iri) => JsonLdSource::Context(iri.map(Iri::to_owned)),
		}
	}
}

/// JSON-LD source document.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JsonLdSource {
	/// Compact JSON-LD document.
	Compact(Option<IriBuf>),

	/// Expanded JSON-LD document.
	Expanded(Option<IriBuf>),

	/// JSON-LD context document.
	Context(Option<IriBuf>),
}
