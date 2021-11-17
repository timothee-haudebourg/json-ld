use std::ops::{Deref, DerefMut};

/// Value `T` attached to some metadata `M`.
#[derive(Clone, Copy)]
pub struct Meta<T, M> {
	value: T,
	metadata: M,
}

impl<T, M> Meta<T, M> {
	/// Creates a new value attached to the given `metadata`.
	pub fn new(value: T, metadata: M) -> Self {
		Self { value, metadata }
	}

	/// Returns a reference to the metadata associated to this value.
	pub fn metadata(&self) -> &M {
		&self.metadata
	}

	pub fn into_parts(self) -> (T, M) {
		(self.value, self.metadata)
	}

	pub fn cast_metadata<N>(self) -> Meta<T, N> where N: From<M> {
		Meta::new(self.value, self.metadata.into())
	}

	pub fn with_metadata<N>(self, metadata: N) -> Meta<T, N> {
		Meta::new(self.value, metadata)
	}
}

impl<T, M> Deref for Meta<T, M> {
	type Target = T;

	fn deref(&self) -> &T {
		&self.value
	}
}

impl<T, M> DerefMut for Meta<T, M> {
	fn deref_mut(&mut self) -> &mut T {
		&mut self.value
	}
}

impl<T, M> AsRef<T> for Meta<T, M> {
	fn as_ref(&self) -> &T {
		&self.value
	}
}

impl<T, M> AsMut<T> for Meta<T, M> {
	fn as_mut(&mut self) -> &mut T {
		&mut self.value
	}
}