use std::hash::Hash;

use crate::{
	JsonLdIndexedObject, JsonLdVisit,
	object::{JsonLdObjectMut, JsonLdObjectRef},
};

use super::AnyJsonLdObject;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// List object.
pub struct JsonLdList {
	#[cfg_attr(feature = "serde", serde(rename = "@list"))]
	entry: Vec<JsonLdIndexedObject>,
}

impl JsonLdList {
	/// Creates a new list object.
	pub fn new(objects: Vec<JsonLdIndexedObject>) -> Self {
		Self { entry: objects }
	}

	pub fn len(&self) -> usize {
		self.entry.len()
	}

	pub fn is_empty(&self) -> bool {
		self.entry.is_empty()
	}

	/// Returns a reference to the "@list" entry of the list object.
	///
	/// Alias for `as_slice`.
	pub fn entry(&self) -> &[JsonLdIndexedObject] {
		&self.entry
	}

	pub fn entry_mut(&mut self) -> &mut Vec<JsonLdIndexedObject> {
		&mut self.entry
	}

	pub fn as_slice(&self) -> &[JsonLdIndexedObject] {
		self.entry.as_slice()
	}

	pub fn as_mut_slice(&mut self) -> &mut [JsonLdIndexedObject] {
		self.entry.as_mut_slice()
	}

	pub fn into_entry(self) -> Vec<JsonLdIndexedObject> {
		self.entry
	}

	pub fn push(&mut self, object: JsonLdIndexedObject) {
		self.entry.push(object)
	}

	pub fn pop(&mut self) -> Option<JsonLdIndexedObject> {
		self.entry.pop()
	}

	pub fn iter(&self) -> core::slice::Iter<'_, JsonLdIndexedObject> {
		self.entry.iter()
	}

	pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, JsonLdIndexedObject> {
		self.entry.iter_mut()
	}
}

impl JsonLdVisit for JsonLdList {
	fn visit_with(&self, f: &mut impl FnMut(JsonLdObjectRef)) {
		f(JsonLdObjectRef::List(self));
		for t in self {
			t.visit_with(f);
		}
	}

	fn visit_mut_with(&mut self, f: &mut impl FnMut(JsonLdObjectMut)) {
		f(JsonLdObjectMut::List(self));
		for t in &mut self.entry {
			t.visit_mut_with(f);
		}
	}
}

impl AnyJsonLdObject for JsonLdList {
	fn as_ref(&self) -> super::JsonLdObjectRef<'_> {
		super::JsonLdObjectRef::List(self)
	}
}

impl<'a> IntoIterator for &'a JsonLdList {
	type Item = &'a JsonLdIndexedObject;
	type IntoIter = core::slice::Iter<'a, JsonLdIndexedObject>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a> IntoIterator for &'a mut JsonLdList {
	type Item = &'a mut JsonLdIndexedObject;
	type IntoIter = core::slice::IterMut<'a, JsonLdIndexedObject>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

impl IntoIterator for JsonLdList {
	type Item = JsonLdIndexedObject;
	type IntoIter = std::vec::IntoIter<JsonLdIndexedObject>;

	fn into_iter(self) -> Self::IntoIter {
		self.entry.into_iter()
	}
}
