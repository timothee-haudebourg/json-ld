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

pub trait AsyncJsonLdEnvironment {
	type Loader: AsyncJsonLdLoader;

	fn loader(&self) -> &Self::Loader;

	fn warn(&self, w: JsonLdWarning);

	fn as_ref(&self) -> AsyncProcessingEnvironmentRef<'_, Self> {
		AsyncProcessingEnvironmentRef(self)
	}
}

impl<L: AsyncJsonLdLoader> AsyncJsonLdEnvironment for L {
	type Loader = Self;

	fn loader(&self) -> &Self::Loader {
		self
	}

	fn warn(&self, _: JsonLdWarning) {
		// Ignore.
	}
}

pub struct AsyncProcessingEnvironmentRef<'a, T: ?Sized>(pub &'a T);

impl<'a, T: ?Sized + AsyncJsonLdEnvironment> AsyncJsonLdEnvironment
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

pub trait JsonLdEnvironment {
	type Loader: JsonLdLoader;

	fn loader(&self) -> &Self::Loader;

	fn warn(&self, w: JsonLdWarning);

	fn as_async_environment(&self) -> &ToAsyncJsonLdEnvironment<Self> {
		ToAsyncJsonLdEnvironment::from_ref(self)
	}

	fn into_async_environment(self) -> ToAsyncJsonLdEnvironment<Self>
	where
		Self: Sized,
	{
		ToAsyncJsonLdEnvironment(self)
	}
}

impl<L: JsonLdLoader> JsonLdEnvironment for L {
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
pub struct ToAsyncJsonLdEnvironment<T: ?Sized>(pub T);

impl<T: ?Sized> ToAsyncJsonLdEnvironment<T> {
	pub fn from_ref(env: &T) -> &Self {
		unsafe {
			// SAFETY: `ToAsyncJsonLdEnvironment` uses the `transparent` repr.
			std::mem::transmute(env)
		}
	}
}

impl<T: JsonLdEnvironment> AsyncJsonLdEnvironment for ToAsyncJsonLdEnvironment<T> {
	type Loader = ToAsyncJsonLdLoader<T::Loader>;

	fn loader(&self) -> &Self::Loader {
		self.0.loader().as_async_loader()
	}

	fn warn(&self, w: JsonLdWarning) {
		self.0.warn(w);
	}
}
