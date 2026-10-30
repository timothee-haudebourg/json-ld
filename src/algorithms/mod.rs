use crate::{AsyncJsonLdLoader, JsonLdLoader, JsonLdWarning, ToAsyncJsonLdLoader};

mod compaction;
mod context_processing;
mod deserialization;
mod expansion;
mod flattening;
mod serialization;

pub use compaction::*;
pub use context_processing::*;
pub use deserialization::RdfSerializationOptions;
pub use expansion::*;

pub trait AsyncProcessingEnvironment {
	type Loader: AsyncJsonLdLoader;

	fn loader(&self) -> &Self::Loader;

	fn warn(&self, w: JsonLdWarning);

	fn as_ref(&self) -> AsyncProcessingEnvironmentRef<'_, Self> {
		AsyncProcessingEnvironmentRef(self)
	}
}

impl<L: AsyncJsonLdLoader> AsyncProcessingEnvironment for L {
	type Loader = Self;

	fn loader(&self) -> &Self::Loader {
		self
	}

	fn warn(&self, _: JsonLdWarning) {
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

	fn warn(&self, w: JsonLdWarning) {
		self.0.warn(w);
	}
}

pub trait ProcessingEnvironment {
	type Loader: JsonLdLoader;

	fn loader(&self) -> &Self::Loader;

	fn warn(&self, w: JsonLdWarning);

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

impl<L: JsonLdLoader> ProcessingEnvironment for L {
	type Loader = Self;

	fn loader(&self) -> &Self::Loader {
		self
	}

	fn warn(&self, _: JsonLdWarning) {
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
	type Loader = ToAsyncJsonLdLoader<T::Loader>;

	fn loader(&self) -> &Self::Loader {
		self.0.loader().as_async_loader()
	}

	fn warn(&self, w: JsonLdWarning) {
		self.0.warn(w);
	}
}
