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
use rdf_syntax::{Uri, UriBuf};
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

pub enum JsonFragmentAddrSegment {
	ArrayIndex(usize),
	ObjectKey(String),
	ObjectValue(String),
}

pub enum JsonFragmentAddrSegmentRef<'a> {
	ArrayIndex(usize),
	ObjectKey(&'a str),
	ObjectValue(&'a str),
}

pub type JsonFragmentAddrBuf = Vec<JsonFragmentAddrSegment>;

pub type JsonFragmentAddr = [JsonFragmentAddrSegment];

pub enum JsonLdLocationStack<'a> {
	Root(Option<&'a Uri>),
	Segment(&'a Self, JsonFragmentAddrSegmentRef<'a>),
}

impl<'a> JsonLdLocationStack<'a> {
	pub fn array_index(&self, index: usize) -> JsonLdLocationStack<'_> {
		JsonLdLocationStack::Segment(self, JsonFragmentAddrSegmentRef::ArrayIndex(index))
	}

	pub fn object_key<'b>(&'b self, key: impl Into<&'b str>) -> JsonLdLocationStack<'b> {
		JsonLdLocationStack::Segment(self, JsonFragmentAddrSegmentRef::ObjectKey(key.into()))
	}

	pub fn object_value<'b>(&'b self, key: impl Into<&'b str>) -> JsonLdLocationStack<'b> {
		JsonLdLocationStack::Segment(self, JsonFragmentAddrSegmentRef::ObjectValue(key.into()))
	}

	pub fn build(&self) -> JsonLdLocation {
		todo!()
	}
}

pub struct JsonLdLocation {
	pub uri: Option<UriBuf>,
	pub fragment: JsonFragmentAddrBuf,
}
