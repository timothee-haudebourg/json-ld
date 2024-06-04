use super::{Any, InvalidExpandedJson, MappedEq};
use crate::{Id, IndexedObject, Relabel, TryFromJson};
use contextual::WithContext;
use educe::Educe;
use json_ld_syntax::{IntoJson, IntoJsonWithContext};
use rdf_types::{Generator, Subject, Vocabulary, VocabularyMut};
use std::hash::Hash;

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Educe, Debug, Clone, Hash)]
#[educe(
	PartialEq(bound = "T: Eq + Hash, B: Eq + Hash"),
	Eq(bound = "T: Eq + Hash, B: Eq + Hash")
)]
/// List object.
pub struct List<T, B> {
	entry: Vec<IndexedObject<T, B>>,
}

impl<T, B> List<T, B> {
	/// Creates a new list object.
	pub fn new(objects: Vec<IndexedObject<T, B>>) -> Self {
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
	pub fn entry(&self) -> &[IndexedObject<T, B>] {
		&self.entry
	}

	pub fn entry_mut(&mut self) -> &mut Vec<IndexedObject<T, B>> {
		&mut self.entry
	}

	pub fn as_slice(&self) -> &[IndexedObject<T, B>] {
		self.entry.as_slice()
	}

	pub fn as_mut_slice(&mut self) -> &mut [IndexedObject<T, B>] {
		self.entry.as_mut_slice()
	}

	pub fn into_entry(self) -> Vec<IndexedObject<T, B>> {
		self.entry
	}

	pub fn push(&mut self, object: IndexedObject<T, B>) {
		self.entry.push(object)
	}

	pub fn pop(&mut self) -> Option<IndexedObject<T, B>> {
		self.entry.pop()
	}

	pub fn iter(&self) -> core::slice::Iter<IndexedObject<T, B>> {
		self.entry.iter()
	}

	pub fn iter_mut(&mut self) -> core::slice::IterMut<IndexedObject<T, B>> {
		self.entry.iter_mut()
	}

	/// Puts this list object literals into canonical form using the given
	/// `buffer`.
	///
	/// The buffer is used to compute the canonical form of numbers.
	pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer) {
		for object in self {
			object.canonicalize_with(buffer)
		}
	}

	/// Puts this list object literals into canonical form.
	pub fn canonicalize(&mut self) {
		let mut buffer = ryu_js::Buffer::new();
		self.canonicalize_with(&mut buffer)
	}

	/// Map the identifiers present in this list (recursively).
	pub fn map_ids<U, C>(
		self,
		mut map_iri: impl FnMut(T) -> U,
		mut map_id: impl FnMut(Id<T, B>) -> Id<U, C>,
	) -> List<U, C>
	where
		U: Eq + Hash,
		C: Eq + Hash,
	{
		self.map_ids_with(&mut map_iri, &mut map_id)
	}

	pub(crate) fn map_ids_with<U, C>(
		self,
		map_iri: &mut impl FnMut(T) -> U,
		map_id: &mut impl FnMut(Id<T, B>) -> Id<U, C>,
	) -> List<U, C>
	where
		U: Eq + Hash,
		C: Eq + Hash,
	{
		List::new(
			self.entry
				.into_iter()
				.map(|indexed_object| {
					indexed_object.map_inner(|object| object.map_ids_with(map_iri, map_id))
				})
				.collect(),
		)
	}
}

impl<T, B> Relabel<T, B> for List<T, B> {
	fn relabel_with<N: Vocabulary<Iri = T, BlankId = B>, G: Generator<N>>(
		&mut self,
		vocabulary: &mut N,
		generator: &mut G,
		relabeling: &mut hashbrown::HashMap<B, Subject<T, B>>,
	) where
		T: Clone + Eq + Hash,
		B: Clone + Eq + Hash,
	{
		for object in self {
			object.relabel_with(vocabulary, generator, relabeling)
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash> List<T, B> {
	pub(crate) fn try_from_json_object_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		object: json_syntax::Object,
		list_entry: json_syntax::object::Entry,
	) -> Result<Self, InvalidExpandedJson> {
		let list = Vec::try_from_json_in(vocabulary, list_entry.value)?;

		match object.into_iter().next() {
			Some(_) => Err(InvalidExpandedJson::UnexpectedEntry),
			None => Ok(Self::new(list)),
		}
	}
}

impl<T, B> Any<T, B> for List<T, B> {
	fn as_ref(&self) -> super::Ref<T, B> {
		super::Ref::List(self)
	}
}

impl<T: Eq + Hash, B: Eq + Hash> MappedEq for List<T, B> {
	type BlankId = B;

	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a B) -> &'b B>(&'a self, other: &Self, f: F) -> bool
	where
		B: 'a + 'b,
	{
		self.entry.mapped_eq(&other.entry, f)
	}
}

impl<'a, T, B> IntoIterator for &'a List<T, B> {
	type Item = &'a IndexedObject<T, B>;
	type IntoIter = core::slice::Iter<'a, IndexedObject<T, B>>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, T, B> IntoIterator for &'a mut List<T, B> {
	type Item = &'a mut IndexedObject<T, B>;
	type IntoIter = core::slice::IterMut<'a, IndexedObject<T, B>>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

impl<T, B> IntoIterator for List<T, B> {
	type Item = IndexedObject<T, B>;
	type IntoIter = std::vec::IntoIter<IndexedObject<T, B>>;

	fn into_iter(self) -> Self::IntoIter {
		self.entry.into_iter()
	}
}

/// List object fragment.
pub enum FragmentRef<'a, T, B> {
	/// "@list" entry.
	Entry(&'a [IndexedObject<T, B>]),

	/// "@list" entry key.
	Key,

	/// "@list" value.
	Value(&'a [IndexedObject<T, B>]),
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> IntoJsonWithContext<N> for List<T, B> {
	fn into_json_with(self, vocabulary: &N) -> json_syntax::Value {
		let mut obj = json_syntax::Object::new();

		obj.insert("@list".into(), self.entry.into_with(vocabulary).into_json());

		obj.into()
	}
}
