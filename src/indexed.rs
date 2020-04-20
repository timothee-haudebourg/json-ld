use std::hash::Hash;
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

	pub fn index(&self) -> Option<&'str> {
		match &self.index {
			Some(index) => Some(index.as_str()),
			None => None
		}
	}

	pub fn set_index(&mut self, index: Option<String>) {
		self.index = index
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

impl<T: AsJson> AsJson for Indexed<T> {
	fn as_json(&self) -> JsonValue {
		let mut json = self.value.as_json();

		if let JsonValue::Object(ref mut obj) = &mut json {
			obj.insert(Keyword::Index.into(), self.index.into())
		}

		json
	}
}
