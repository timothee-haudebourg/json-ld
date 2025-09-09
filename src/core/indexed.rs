use std::convert::{TryFrom, TryInto};
use std::ops::{Deref, DerefMut};

/// Indexed objects.
///
/// Nodes and value objects may be indexed by a string in JSON-LD.
/// This type is a wrapper around any kind of indexable data.
///
/// It is a pointer type that `Deref` into the underlying value.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Indexed<T> {
	/// Index.
	index: Option<String>,

	/// Value.
	value: T,
}

impl<T> Indexed<T> {
	/// Creates a non-indexed value, without metadata.
	pub fn unindexed(value: T) -> Self {
		Self::new(value, None)
	}

	/// Create a new (maybe) indexed value.
	#[inline(always)]
	pub fn new(value: T, index: Option<String>) -> Self {
		Indexed { value, index }
	}

	/// Get a reference to the inner value.
	#[inline(always)]
	pub fn inner(&self) -> &T {
		&self.value
	}

	pub fn inner_mut(&mut self) -> &mut T {
		&mut self.value
	}

	/// Drop the index and return the underlying value.
	#[inline(always)]
	pub fn into_inner(self) -> T {
		self.value
	}

	/// Get the index, if any.
	#[inline(always)]
	pub fn index(&self) -> Option<&str> {
		match &self.index {
			Some(index) => Some(index.as_str()),
			None => None,
		}
	}

	/// Set the value index.
	#[inline(always)]
	pub fn set_index(&mut self, index: Option<String>) {
		self.index = index
	}

	/// Turn this indexed value into its components: inner value and index.
	#[inline(always)]
	pub fn into_parts(self) -> (T, Option<String>) {
		(self.value, self.index)
	}

	/// Cast the inner value.
	#[inline(always)]
	pub fn map_inner<U, F>(self, f: F) -> Indexed<U>
	where
		F: FnOnce(T) -> U,
	{
		Indexed::new(f(self.value), self.index)
	}

	/// Cast the inner value.
	#[inline(always)]
	pub fn cast<U: From<T>>(self) -> Indexed<U> {
		Indexed::new(self.value.into(), self.index)
	}

	/// Try to cast the inner value.
	#[inline(always)]
	pub fn try_cast<U: TryFrom<T>>(self) -> Result<Indexed<U>, Indexed<U::Error>> {
		match self.value.try_into() {
			Ok(value) => Ok(Indexed::new(value, self.index)),
			Err(e) => Err(Indexed::new(e, self.index)),
		}
	}
}

impl<T> From<T> for Indexed<T> {
	#[inline(always)]
	fn from(value: T) -> Indexed<T> {
		Indexed::new(value, None)
	}
}

impl<T> Deref for Indexed<T> {
	type Target = T;

	#[inline(always)]
	fn deref(&self) -> &T {
		&self.value
	}
}

impl<T> DerefMut for Indexed<T> {
	#[inline(always)]
	fn deref_mut(&mut self) -> &mut T {
		&mut self.value
	}
}

impl<T> AsRef<T> for Indexed<T> {
	#[inline(always)]
	fn as_ref(&self) -> &T {
		&self.value
	}
}

impl<T> AsMut<T> for Indexed<T> {
	#[inline(always)]
	fn as_mut(&mut self) -> &mut T {
		&mut self.value
	}
}
