use super::{Any, InvalidExpandedJson, MappedEq};
use crate::{IndexedObject, Relabel, TryFromJson};
use contextual::WithContext;
use derivative::Derivative;
use json_ld_syntax::{Entry, IntoJson, IntoJsonWithContextMeta};
use locspan::{Meta, StrippedEq, StrippedPartialEq};
use locspan_derive::StrippedHash;
use rdf_types::{Subject, Vocabulary, VocabularyMut};
use std::hash::Hash;

#[allow(clippy::derive_hash_xor_eq)]
#[derive(Derivative, Clone, Hash, StrippedHash)]
#[derivative(
	PartialEq(bound = "T: Eq + Hash, B: Eq + Hash, M: PartialEq"),
	Eq(bound = "T: Eq + Hash, B: Eq + Hash, M: Eq")
)]
#[locspan(ignore(M))]
#[locspan(stripped(T, B))]
/// List object.
pub struct List<T, B, M> {
	entry: Entry<Vec<IndexedObject<T, B, M>>, M>,
}

impl<T, B, M> List<T, B, M> {
	pub fn new(key_metadata: M, value: Meta<Vec<IndexedObject<T, B, M>>, M>) -> Self {
		Self {
			entry: Entry::new(key_metadata, value),
		}
	}

	pub fn len(&self) -> usize {
		self.entry.value.len()
	}

	pub fn is_empty(&self) -> bool {
		self.entry.value.is_empty()
	}

	/// Returns a reference to the "@list" entry of the list object.
	pub fn entry(&self) -> &Entry<Vec<IndexedObject<T, B, M>>, M> {
		&self.entry
	}

	pub fn entry_mut(&mut self) -> &mut Entry<Vec<IndexedObject<T, B, M>>, M> {
		&mut self.entry
	}

	pub fn into_entry(self) -> Entry<Vec<IndexedObject<T, B, M>>, M> {
		self.entry
	}

	pub fn push(&mut self, object: IndexedObject<T, B, M>) {
		self.entry.push(object)
	}

	pub fn pop(&mut self) -> Option<IndexedObject<T, B, M>> {
		self.entry.pop()
	}

	pub fn iter(&self) -> core::slice::Iter<IndexedObject<T, B, M>> {
		self.entry.iter()
	}

	pub fn iter_mut(&mut self) -> core::slice::IterMut<IndexedObject<T, B, M>> {
		self.entry.iter_mut()
	}

	pub fn as_slice(&self) -> &[IndexedObject<T, B, M>] {
		self.entry.as_slice()
	}

	pub fn as_mut_slice(&mut self) -> &mut [IndexedObject<T, B, M>] {
		self.entry.as_mut_slice()
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
}

impl<T, B, M> Relabel<T, B, M> for List<T, B, M> {
	fn relabel_with<N: Vocabulary<Iri = T, BlankId = B>, G: rdf_types::MetaGenerator<N, M>>(
		&mut self,
		vocabulary: &mut N,
		generator: &mut G,
		relabeling: &mut hashbrown::HashMap<B, Meta<Subject<T, B>, M>>,
	) where
		M: Clone,
		T: Clone + Eq + Hash,
		B: Clone + Eq + Hash,
	{
		for object in self {
			object.relabel_with(vocabulary, generator, relabeling)
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> List<T, B, M> {
	pub(crate) fn try_from_json_object_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		object: json_syntax::Object<M>,
		list_entry: json_syntax::object::Entry<M>,
	) -> Result<Self, Meta<InvalidExpandedJson<M>, M>> {
		let list = Vec::try_from_json_in(vocabulary, list_entry.value)?;

		match object.into_iter().next() {
			Some(unexpected_entry) => Err(Meta(
				InvalidExpandedJson::UnexpectedEntry,
				unexpected_entry.key.into_metadata(),
			)),
			None => Ok(Self::new(list_entry.key.into_metadata(), list)),
		}
	}
}

impl<T, B, M> Any<T, B, M> for List<T, B, M> {
	fn as_ref(&self) -> super::Ref<T, B, M> {
		super::Ref::List(self)
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> StrippedPartialEq for List<T, B, M> {
	fn stripped_eq(&self, other: &Self) -> bool {
		self.entry.stripped_eq(&other.entry)
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> StrippedEq for List<T, B, M> {}

impl<T: Eq + Hash, B: Eq + Hash, M> MappedEq for List<T, B, M> {
	type BlankId = B;

	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a B) -> &'b B>(&'a self, other: &Self, f: F) -> bool
	where
		B: 'a + 'b,
	{
		self.entry.mapped_eq(&other.entry, f)
	}
}

impl<'a, T, B, M> IntoIterator for &'a List<T, B, M> {
	type Item = &'a IndexedObject<T, B, M>;
	type IntoIter = core::slice::Iter<'a, IndexedObject<T, B, M>>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, T, B, M> IntoIterator for &'a mut List<T, B, M> {
	type Item = &'a mut IndexedObject<T, B, M>;
	type IntoIter = core::slice::IterMut<'a, IndexedObject<T, B, M>>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

impl<T, B, M> IntoIterator for List<T, B, M> {
	type Item = IndexedObject<T, B, M>;
	type IntoIter = std::vec::IntoIter<IndexedObject<T, B, M>>;

	fn into_iter(self) -> Self::IntoIter {
		self.entry.value.into_value().into_iter()
	}
}

pub type EntryRef<'a, T, B, M> = &'a Entry<Vec<IndexedObject<T, B, M>>, M>;

pub type EntryValueRef<'a, T, B, M> = &'a [IndexedObject<T, B, M>];

/// List object fragment.
pub enum FragmentRef<'a, T, B, M> {
	/// "@list" entry.
	Entry(EntryRef<'a, T, B, M>),

	/// "@list" entry key.
	Key(&'a M),

	/// "@list" value.
	Value(EntryValueRef<'a, T, B, M>),
}

impl<T, B, M: Clone, N: Vocabulary<Iri = T, BlankId = B>> IntoJsonWithContextMeta<M, N>
	for List<T, B, M>
{
	fn into_json_meta_with(self, meta: M, vocabulary: &N) -> Meta<json_syntax::Value<M>, M> {
		let mut obj = json_syntax::Object::new();

		obj.insert(
			Meta("@list".into(), self.entry.key_metadata),
			self.entry.value.into_with(vocabulary).into_json(),
		);

		Meta(obj.into(), meta)
	}
}
