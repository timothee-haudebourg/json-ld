use std::hash::Hash;

use rdf_types::BlankId;

use crate::IndexedObject;

use super::{AnyObject, MappedEq};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// List object.
pub struct List {
	entry: Vec<IndexedObject>,
}

impl List {
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

	pub fn iter(&self) -> core::slice::Iter<IndexedObject> {
		self.entry.iter()
	}

	pub fn iter_mut(&mut self) -> core::slice::IterMut<IndexedObject> {
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

// impl Relabel for List {
// 	fn relabel_with<N: Vocabulary<Iri = T, BlankId = B>, G: Generator<N>>(
// 		&mut self,
// 		vocabulary: &mut N,
// 		generator: &mut G,
// 		relabeling: &mut hashbrown::HashMap<B, Subject>,
// 	) where
// 		T: Clone + Eq + Hash,
// 		B: Clone + Eq + Hash,
// 	{
// 		for object in self {
// 			object.relabel_with(vocabulary, generator, relabeling)
// 		}
// 	}
// }

impl AnyObject for List {
	fn as_ref(&self) -> super::Ref {
		super::Ref::List(self)
	}
}

impl MappedEq for List {
	fn mapped_eq(&self, other: &Self, f: impl Clone + Fn(&BlankId) -> &BlankId) -> bool {
		self.entry.mapped_eq(&other.entry, f)
	}
}

impl<'a> IntoIterator for &'a List {
	type Item = &'a IndexedObject;
	type IntoIter = core::slice::Iter<'a, IndexedObject>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a> IntoIterator for &'a mut List {
	type Item = &'a mut IndexedObject;
	type IntoIter = core::slice::IterMut<'a, IndexedObject>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

impl IntoIterator for List {
	type Item = IndexedObject;
	type IntoIter = std::vec::IntoIter<IndexedObject>;

	fn into_iter(self) -> Self::IntoIter {
		self.entry.into_iter()
	}
}
