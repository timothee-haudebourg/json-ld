use std::convert::{TryFrom, TryInto};
use std::ops::{Deref, DerefMut};

use crate::JsonLdVisit;
use crate::object::{JsonLdObjectMut, JsonLdObjectRef};

/// Indexed objects.
///
/// Nodes and value objects may be indexed by a string in JSON-LD.
/// This type is a wrapper around any kind of indexable data.
///
/// It is a pointer type that `Deref` into the underlying value.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JsonLdIndexed<T> {
	/// Index.
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@index", default, skip_serializing_if = "Option::is_none")
	)]
	index: Option<String>,

	/// Value.
	#[cfg_attr(feature = "serde", serde(flatten))]
	value: T,
}

impl<T> JsonLdIndexed<T> {
	/// Creates a non-indexed value, without metadata.
	pub fn unindexed(value: T) -> Self {
		Self::new(value, None)
	}

	/// Create a new (maybe) indexed value.
	#[inline(always)]
	pub fn new(value: T, index: Option<String>) -> Self {
		JsonLdIndexed { value, index }
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
	pub fn map_inner<U, F>(self, f: F) -> JsonLdIndexed<U>
	where
		F: FnOnce(T) -> U,
	{
		JsonLdIndexed::new(f(self.value), self.index)
	}

	/// Cast the inner value.
	#[inline(always)]
	pub fn cast<U: From<T>>(self) -> JsonLdIndexed<U> {
		JsonLdIndexed::new(self.value.into(), self.index)
	}

	/// Try to cast the inner value.
	#[inline(always)]
	pub fn try_cast<U: TryFrom<T>>(self) -> Result<JsonLdIndexed<U>, JsonLdIndexed<U::Error>> {
		match self.value.try_into() {
			Ok(value) => Ok(JsonLdIndexed::new(value, self.index)),
			Err(e) => Err(JsonLdIndexed::new(e, self.index)),
		}
	}
}

impl<T: JsonLdVisit> JsonLdVisit for JsonLdIndexed<T> {
	fn visit_with(&self, f: &mut impl FnMut(JsonLdObjectRef)) {
		self.value.visit_with(f);
	}

	fn visit_mut_with(&mut self, f: &mut impl FnMut(JsonLdObjectMut)) {
		self.value.visit_mut_with(f);
	}
}

impl<T> From<T> for JsonLdIndexed<T> {
	#[inline(always)]
	fn from(value: T) -> JsonLdIndexed<T> {
		JsonLdIndexed::new(value, None)
	}
}

impl<T> Deref for JsonLdIndexed<T> {
	type Target = T;

	#[inline(always)]
	fn deref(&self) -> &T {
		&self.value
	}
}

impl<T> DerefMut for JsonLdIndexed<T> {
	#[inline(always)]
	fn deref_mut(&mut self) -> &mut T {
		&mut self.value
	}
}

impl<T> AsRef<T> for JsonLdIndexed<T> {
	#[inline(always)]
	fn as_ref(&self) -> &T {
		&self.value
	}
}

impl<T> AsMut<T> for JsonLdIndexed<T> {
	#[inline(always)]
	fn as_mut(&mut self) -> &mut T {
		&mut self.value
	}
}
