use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy, Debug)]
pub struct Location<F, M> {
	source: F,
	metadata: M,
}

impl<F, M> Location<F, M> {
	pub fn new(source: F, metadata: M) -> Self {
		Self { source, metadata }
	}

	pub fn source(&self) -> &F {
		&self.source
	}

	/// Returns a reference to the metadata associated to this value.
	pub fn metadata(&self) -> &M {
		&self.metadata
	}
}

/// Value located behind an IRI reference.
#[derive(Clone, Copy, Debug)]
pub struct Loc<T, F, M> {
	/// The value.
	value: T,

	loc: Location<F, M>,
}

impl<T, F, M> Loc<T, F, M> {
	/// Creates a new value from the given `source` attached to the given `metadata`.
	pub fn new(value: T, loc: Location<F, M>) -> Self {
		Self { value, loc }
	}

	pub fn value(&self) -> &T {
		&self.value
	}

	pub fn location(&self) -> &Location<F, M> {
		&self.loc
	}

	/// Returns a reference to the metadata associated to this value.
	pub fn metadata(&self) -> &M {
		&self.loc.metadata
	}

	pub fn into_parts(self) -> (T, Location<F, M>) {
		(self.value, self.loc)
	}

	pub fn unwrap(self) -> T {
		self.value
	}

	// pub fn cast_metadata<N>(self) -> Loc<T, F, N>
	// where
	// 	N: From<M>,
	// {
	// 	Loc::new(self.value, self.source, self.metadata.into())
	// }

	// pub fn with_metadata<N>(self, metadata: N) -> Loc<T, F, N> {
	// 	Loc::new(self.value, self.source, metadata)
	// }

	// pub fn map_metadata<N, F>(self, f: F) -> Loc<T, N>
	// where
	// 	F: FnOnce(M) -> N,
	// {
	// 	Loc::new(self.value, self.source, f(self.metadata))
	// }
}

impl<T, F, M> Deref for Loc<T, F, M> {
	type Target = T;

	fn deref(&self) -> &T {
		&self.value
	}
}

impl<T, F, M> DerefMut for Loc<T, F, M> {
	fn deref_mut(&mut self) -> &mut T {
		&mut self.value
	}
}

impl<T, F, M> AsRef<T> for Loc<T, F, M> {
	fn as_ref(&self) -> &T {
		&self.value
	}
}

impl<T, F, M> AsMut<T> for Loc<T, F, M> {
	fn as_mut(&mut self) -> &mut T {
		&mut self.value
	}
}
