use std::hash::Hash;

use crate::{
	IndexedJsonLdObject, JsonLdVisit,
	object::{JsonLdObjectMut, JsonLdObjectRef},
};

use super::AnyJsonLdObject;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// List object.
pub struct JsonLdList {
	#[cfg_attr(feature = "serde", serde(rename = "@list"))]
	entry: Vec<IndexedJsonLdObject>,
}

impl JsonLdList {
	/// Creates a new list object.
	pub fn new(objects: Vec<IndexedJsonLdObject>) -> Self {
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
	pub fn entry(&self) -> &[IndexedJsonLdObject] {
		&self.entry
	}

	pub fn entry_mut(&mut self) -> &mut Vec<IndexedJsonLdObject> {
		&mut self.entry
	}

	pub fn as_slice(&self) -> &[IndexedJsonLdObject] {
		self.entry.as_slice()
	}

	pub fn as_mut_slice(&mut self) -> &mut [IndexedJsonLdObject] {
		self.entry.as_mut_slice()
	}

	pub fn into_entry(self) -> Vec<IndexedJsonLdObject> {
		self.entry
	}

	pub fn push(&mut self, object: IndexedJsonLdObject) {
		self.entry.push(object)
	}

	pub fn pop(&mut self) -> Option<IndexedJsonLdObject> {
		self.entry.pop()
	}

	pub fn iter(&self) -> core::slice::Iter<'_, IndexedJsonLdObject> {
		self.entry.iter()
	}

	pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, IndexedJsonLdObject> {
		self.entry.iter_mut()
	}

	// /// Puts this list object literals into canonical form using the given
	// /// `buffer`.
	// ///
	// /// The buffer is used to compute the canonical form of numbers.
	// pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer) {
	// 	for object in self {
	// 		object.canonicalize_with(buffer)
	// 	}
	// }

	// /// Puts this list object literals into canonical form.
	// pub fn canonicalize(&mut self) {
	// 	let mut buffer = ryu_js::Buffer::new();
	// 	self.canonicalize_with(&mut buffer)
	// }
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
	type Item = &'a IndexedJsonLdObject;
	type IntoIter = core::slice::Iter<'a, IndexedJsonLdObject>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a> IntoIterator for &'a mut JsonLdList {
	type Item = &'a mut IndexedJsonLdObject;
	type IntoIter = core::slice::IterMut<'a, IndexedJsonLdObject>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

impl IntoIterator for JsonLdList {
	type Item = IndexedJsonLdObject;
	type IntoIter = std::vec::IntoIter<IndexedJsonLdObject>;

	fn into_iter(self) -> Self::IntoIter {
		self.entry.into_iter()
	}
}
