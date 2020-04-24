use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::convert::{TryFrom, TryInto};
use json::JsonValue;
use crate::{
	Keyword,
	util::AsJson
};

pub struct Indexed<T> {
	index: Option<String>,
	value: T
}

impl<T> Indexed<T> {
	pub fn new(value: T, index: Option<String>) -> Indexed<T> {
		Indexed {
			value, index
		}
	}

	pub fn inner(&self) -> &T {
		&self.value
	}

	pub fn into_inner(self) -> T {
		self.value
	}

	pub fn index(&self) -> Option<&str> {
		match &self.index {
			Some(index) => Some(index.as_str()),
			None => None
		}
	}

	pub fn set_index(&mut self, index: Option<String>) {
		self.index = index
	}

	pub fn into_parts(self) -> (T, Option<String>) {
		(self.value, self.index)
	}

	pub fn cast<U: From<T>>(self) -> Indexed<U> {
		Indexed::new(self.value.into(), self.index)
	}

	pub fn try_cast<U: TryFrom<T>>(self) -> Result<Indexed<U>, Indexed<U::Error>> {
		match self.value.try_into() {
			Ok(value) => Ok(Indexed::new(value, self.index)),
			Err(e) => Err(Indexed::new(e, self.index))
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

impl<T: AsJson> AsJson for Indexed<T> {
	fn as_json(&self) -> JsonValue {
		let mut json = self.value.as_json();

		if let JsonValue::Object(ref mut obj) = &mut json {
			if let Some(index) = &self.index {
				obj.insert(Keyword::Index.into(), index.as_json())
			}
		}

		json
	}
}
