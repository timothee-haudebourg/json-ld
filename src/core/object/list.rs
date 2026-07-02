use std::hash::Hash;

use crate::{
	IndexedObject, VisitJsonLd,
	object::{ObjectMut, ObjectRef},
};

use super::AnyObject;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// List object.
pub struct ListObject {
	#[cfg_attr(feature = "serde", serde(rename = "@list"))]
	entry: Vec<IndexedObject>,
}

impl ListObject {
	/// Creates a new list object.
	pub fn new(objects: Vec<IndexedObject>) -> Self {
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
	pub fn entry(&self) -> &[IndexedObject] {
		&self.entry
	}

	pub fn entry_mut(&mut self) -> &mut Vec<IndexedObject> {
		&mut self.entry
	}

	pub fn as_slice(&self) -> &[IndexedObject] {
		self.entry.as_slice()
	}

	pub fn as_mut_slice(&mut self) -> &mut [IndexedObject] {
		self.entry.as_mut_slice()
	}

	pub fn into_entry(self) -> Vec<IndexedObject> {
		self.entry
	}

	pub fn push(&mut self, object: IndexedObject) {
		self.entry.push(object)
	}

	pub fn pop(&mut self) -> Option<IndexedObject> {
		self.entry.pop()
	}

	pub fn iter(&self) -> core::slice::Iter<'_, IndexedObject> {
		self.entry.iter()
	}

	pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, IndexedObject> {
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

impl VisitJsonLd for ListObject {
	fn visit_with(&self, f: &mut impl FnMut(ObjectRef)) {
		f(ObjectRef::List(self));
		for t in self {
			t.visit_with(f);
		}
	}

	fn visit_mut_with(&mut self, f: &mut impl FnMut(ObjectMut)) {
		f(ObjectMut::List(self));
		for t in &mut self.entry {
			t.visit_mut_with(f);
		}
	}
}

impl AnyObject for ListObject {
	fn as_ref(&self) -> super::ObjectRef<'_> {
		super::ObjectRef::List(self)
	}
}

impl<'a> IntoIterator for &'a ListObject {
	type Item = &'a IndexedObject;
	type IntoIter = core::slice::Iter<'a, IndexedObject>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a> IntoIterator for &'a mut ListObject {
	type Item = &'a mut IndexedObject;
	type IntoIter = core::slice::IterMut<'a, IndexedObject>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

impl IntoIterator for ListObject {
	type Item = IndexedObject;
	type IntoIter = std::vec::IntoIter<IndexedObject>;

	fn into_iter(self) -> Self::IntoIter {
		self.entry.into_iter()
	}
}
