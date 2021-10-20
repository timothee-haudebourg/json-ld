use crate::{
	syntax::Keyword,
	util::{AsJson, JsonFrom},
};
use generic_json::JsonClone;
use std::convert::{TryFrom, TryInto};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

/// Indexed objects.
///
/// Nodes and value objects may be indexed by a string in JSON-LD.
/// This type is a wrapper around any kind of indexable data.
///
/// It is a pointer type that `Deref` into the underlying value.
pub struct Indexed<T> {
	/// Index.
	index: Option<String>,

	/// Value.
	value: T,
}

impl<T> Indexed<T> {
	/// Create a new (maybe) indexed value.
	pub fn new(value: T, index: Option<String>) -> Indexed<T> {
		Indexed { value, index }
	}

	/// Get a reference to the inner value.
	pub fn inner(&self) -> &T {
		&self.value
	}

	/// Drop the index and return the underlying value.
	pub fn into_inner(self) -> T {
		self.value
	}

	/// Get the index, if any.
	pub fn index(&self) -> Option<&str> {
		match &self.index {
			Some(index) => Some(index.as_str()),
			None => None,
		}
	}

	/// Set the value index.
	pub fn set_index(&mut self, index: Option<String>) {
		self.index = index
	}

	/// Turn this indexed value into its components: inner value and index.
	pub fn into_parts(self) -> (T, Option<String>) {
		(self.value, self.index)
	}

	/// Cast the inner value.
	pub fn cast<U: From<T>>(self) -> Indexed<U> {
		Indexed::new(self.value.into(), self.index)
	}

	/// Try to cast the inner value.
	pub fn try_cast<U: TryFrom<T>>(self) -> Result<Indexed<U>, Indexed<U::Error>> {
		match self.value.try_into() {
			Ok(value) => Ok(Indexed::new(value, self.index)),
			Err(e) => Err(Indexed::new(e, self.index)),
		}
	}
}

impl<T: Hash> Hash for Indexed<T> {
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.value.hash(h);
		self.index.hash(h)
	}
}

impl<T: PartialEq> PartialEq for Indexed<T> {
	fn eq(&self, other: &Self) -> bool {
		self.index == other.index && self.value == other.value
	}
}

impl<T: Eq> Eq for Indexed<T> {}

impl<T: Clone> Clone for Indexed<T> {
	fn clone(&self) -> Self {
		Indexed::new(self.value.clone(), self.index.clone())
	}
}

impl<T> From<T> for Indexed<T> {
	fn from(value: T) -> Indexed<T> {
		Indexed::new(value, None)
	}
}

impl<T> Deref for Indexed<T> {
	type Target = T;

	fn deref(&self) -> &T {
		&self.value
	}
}

impl<T> DerefMut for Indexed<T> {
	fn deref_mut(&mut self) -> &mut T {
		&mut self.value
	}
}

impl<T> AsRef<T> for Indexed<T> {
	fn as_ref(&self) -> &T {
		&self.value
	}
}

impl<T> AsMut<T> for Indexed<T> {
	fn as_mut(&mut self) -> &mut T {
		&mut self.value
	}
}

impl<J: JsonClone, K: JsonFrom<J>, T: AsJson<J, K>> AsJson<J, K> for Indexed<T> {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		let mut json = self.value.as_json_with(meta.clone());

		if let Some(obj) = json.as_object_mut() {
			if let Some(index) = &self.index {
				obj.insert(
					K::new_key(Keyword::Index.into_str(), meta(None)),
					index.as_json_with(meta)
				);
			}
		}

		json
	}
}
