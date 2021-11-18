use crate::loader;
use std::ops::{Deref, DerefMut};

/// Value located behind an IRI reference.
#[derive(Clone, Copy, Debug)]
pub struct Loc<T, M> {
	/// The value.
	value: T,

	/// Source document.
	source: Option<loader::Id>,

	/// Metadata.
	metadata: M,
}

impl<T, M> Loc<T, M> {
	/// Creates a new value from the given `source` attached to the given `metadata`.
	pub fn new(value: T, source: Option<loader::Id>, metadata: M) -> Self {
		Self {
			value,
			source,
			metadata,
		}
	}

	pub fn value(&self) -> &T {
		&self.value
	}

	pub fn source(&self) -> Option<loader::Id> {
		self.source
	}

	/// Returns a reference to the metadata associated to this value.
	pub fn metadata(&self) -> &M {
		&self.metadata
	}

	pub fn into_parts(self) -> (T, Option<loader::Id>, M) {
		(self.value, self.source, self.metadata)
	}

	pub fn unwrap(self) -> T {
		self.value
	}

	pub fn cast_metadata<N>(self) -> Loc<T, N>
	where
		N: From<M>,
	{
		Loc::new(self.value, self.source, self.metadata.into())
	}

	pub fn with_metadata<N>(self, metadata: N) -> Loc<T, N> {
		Loc::new(self.value, self.source, metadata)
	}

	pub fn map_metadata<N, F>(self, f: F) -> Loc<T, N>
	where
		F: FnOnce(M) -> N,
	{
		Loc::new(self.value, self.source, f(self.metadata))
	}
}

impl<T, M> Deref for Loc<T, M> {
	type Target = T;

	fn deref(&self) -> &T {
		&self.value
	}
}

impl<T, M> DerefMut for Loc<T, M> {
	fn deref_mut(&mut self) -> &mut T {
		&mut self.value
	}
}

impl<T, M> AsRef<T> for Loc<T, M> {
	fn as_ref(&self) -> &T {
		&self.value
	}
}

impl<T, M> AsMut<T> for Loc<T, M> {
	fn as_mut(&mut self) -> &mut T {
		&mut self.value
	}
}
