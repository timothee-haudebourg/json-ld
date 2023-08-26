use crate::object::{FragmentRef, InvalidExpandedJson, Traverse};
use crate::{id, Id, Indexed, Relabel, StrippedIndexedObject};
use crate::{IndexedObject, TryFromJson};
use hashbrown::HashMap;
use indexmap::IndexSet;
use iref::IriBuf;
use locspan::{Meta, StrippedEq, StrippedPartialEq};
use rdf_types::vocabulary::VocabularyMut;
use rdf_types::{BlankIdBuf, Vocabulary};
use std::collections::HashSet;
use std::hash::Hash;

/// Result of the document expansion algorithm.
///
/// It is just an alias for a set of (indexed) objects.
pub struct ExpandedDocument<T = IriBuf, B = BlankIdBuf, M = ()>(
	IndexSet<StrippedIndexedObject<T, B, M>>,
);

impl<T, B, M> Default for ExpandedDocument<T, B, M> {
	#[inline(always)]
	fn default() -> Self {
		Self(IndexSet::new())
	}
}

impl<T, B, M> ExpandedDocument<T, B, M> {
	#[inline(always)]
	pub fn new() -> Self {
		Self::default()
	}

	#[inline(always)]
	pub fn len(&self) -> usize {
		self.0.len()
	}

	#[inline(always)]
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	#[inline(always)]
	pub fn objects(&self) -> &IndexSet<StrippedIndexedObject<T, B, M>> {
		&self.0
	}

	#[inline(always)]
	pub fn into_objects(self) -> IndexSet<StrippedIndexedObject<T, B, M>> {
		self.0
	}

	#[inline(always)]
	pub fn iter(&self) -> indexmap::set::Iter<'_, StrippedIndexedObject<T, B, M>> {
		self.0.iter()
	}

	#[inline(always)]
	pub fn traverse(&self) -> Traverse<T, B, M> {
		Traverse::new(self.iter().map(|o| FragmentRef::IndexedObject(o)))
	}

	#[inline(always)]
	pub fn count(&self, f: impl FnMut(&FragmentRef<T, B, M>) -> bool) -> usize {
		self.traverse().filter(f).count()
	}

	/// Give an identifier (`@id`) to every nodes using the given generator to
	/// generate fresh identifiers for anonymous nodes.
	#[inline(always)]
	pub fn identify_all_with<V: Vocabulary<Iri = T, BlankId = B>, G: id::Generator<V, M>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut G,
	) where
		M: Clone,
		T: Eq + Hash,
		B: Eq + Hash,
	{
		let mut objects = std::mem::take(&mut self.0);
		for mut object in objects {
			object.identify_all_with(vocabulary, generator);
			self.0.insert(object);
		}
	}

	/// Give an identifier (`@id`) to every nodes using the given generator to
	/// generate fresh identifiers for anonymous nodes.
	#[inline(always)]
	pub fn identify_all<G: id::Generator<(), M>>(&mut self, generator: &mut G)
	where
		M: Clone,
		T: Eq + Hash,
		B: Eq + Hash,
		(): Vocabulary<Iri = T, BlankId = B>,
	{
		self.identify_all_with(&mut (), generator)
	}

	/// Give an identifier (`@id`) to every nodes and canonicalize every
	/// literals using the given generator to generate fresh identifiers for
	/// anonymous nodes.
	#[inline(always)]
	pub fn relabel_and_canonicalize_with<
		V: Vocabulary<Iri = T, BlankId = B>,
		G: id::Generator<V, M>,
	>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut G,
	) where
		M: Clone,
		T: Clone + Eq + Hash,
		B: Clone + Eq + Hash,
	{
		let mut objects = std::mem::take(&mut self.0);
		let mut relabeling = HashMap::new();
		let mut buffer = ryu_js::Buffer::new();
		for mut object in objects {
			object.relabel_with(vocabulary, generator, &mut relabeling);
			object.canonicalize_with(&mut buffer);
			self.0.insert(object);
		}
	}

	/// Give an identifier (`@id`) to every nodes and canonicalize every
	/// literals using the given generator to generate fresh identifiers for
	/// anonymous nodes.
	#[inline(always)]
	pub fn relabel_and_canonicalize<G: id::Generator<(), M>>(&mut self, generator: &mut G)
	where
		M: Clone,
		T: Clone + Eq + Hash,
		B: Clone + Eq + Hash,
		(): Vocabulary<Iri = T, BlankId = B>,
	{
		self.relabel_and_canonicalize_with(&mut (), generator)
	}

	/// Relabels nodes.
	#[inline(always)]
	pub fn relabel_with<V: Vocabulary<Iri = T, BlankId = B>, G: id::Generator<V, M>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut G,
	) where
		M: Clone,
		T: Clone + Eq + Hash,
		B: Clone + Eq + Hash,
	{
		let mut objects = std::mem::take(&mut self.0);
		let mut relabeling = HashMap::new();
		for mut object in objects {
			object.relabel_with(vocabulary, generator, &mut relabeling);
			self.0.insert(object);
		}
	}

	/// Relabels nodes.
	#[inline(always)]
	pub fn relabel<G: id::Generator<(), M>>(&mut self, generator: &mut G)
	where
		M: Clone,
		T: Clone + Eq + Hash,
		B: Clone + Eq + Hash,
		(): Vocabulary<Iri = T, BlankId = B>,
	{
		self.relabel_with(&mut (), generator)
	}

	/// Puts this document literals into canonical form using the given
	/// `buffer`.
	///
	/// The buffer is used to compute the canonical form of numbers.
	pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer)
	where
		T: Eq + Hash,
		B: Eq + Hash,
	{
		let mut objects = std::mem::take(&mut self.0);
		for mut object in objects {
			object.canonicalize_with(buffer);
			self.0.insert(object);
		}
	}

	/// Puts this document literals into canonical form.
	pub fn canonicalize(&mut self)
	where
		T: Eq + Hash,
		B: Eq + Hash,
	{
		let mut buffer = ryu_js::Buffer::new();
		self.canonicalize_with(&mut buffer)
	}

	/// Returns the set of all blank identifiers in the given document.
	pub fn blank_ids(&self) -> HashSet<&B>
	where
		B: Eq + Hash,
	{
		self.traverse()
			.filter_map(|f| f.into_id().and_then(Id::into_blank))
			.collect()
	}
}

impl<T: Hash + Eq, B: Hash + Eq, M> ExpandedDocument<T, B, M> {
	#[inline(always)]
	pub fn insert(&mut self, object: IndexedObject<T, B, M>) -> bool {
		self.0.insert(locspan::Stripped(object))
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> TryFromJson<T, B, M> for ExpandedDocument<T, B, M> {
	fn try_from_json_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidExpandedJson<M>, M>> {
		match value {
			json_syntax::Value::Array(items) => {
				let mut result = Self::new();

				for item in items {
					result.insert(Indexed::try_from_json_in(vocabulary, item)?);
				}

				Ok(Meta(result, meta))
			}
			other => Err(Meta(
				InvalidExpandedJson::Unexpected(other.kind(), json_syntax::Kind::Array),
				meta,
			)),
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> PartialEq for ExpandedDocument<T, B, M> {
	/// Comparison between two expanded documents.
	fn eq(&self, other: &Self) -> bool {
		self.0.eq(&other.0)
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> Eq for ExpandedDocument<T, B, M> {}

impl<T: Eq + Hash, B: Eq + Hash, M> StrippedPartialEq for ExpandedDocument<T, B, M> {
	/// Comparison between two expanded documents.
	fn stripped_eq(&self, other: &Self) -> bool {
		self.0.eq(&other.0)
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> StrippedEq for ExpandedDocument<T, B, M> {}

impl<T, B, M> IntoIterator for ExpandedDocument<T, B, M> {
	type IntoIter = IntoIter<T, B, M>;
	type Item = IndexedObject<T, B, M>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		IntoIter(self.0.into_iter())
	}
}

impl<'a, T, B, M> IntoIterator for &'a ExpandedDocument<T, B, M> {
	type IntoIter = indexmap::set::Iter<'a, StrippedIndexedObject<T, B, M>>;
	type Item = &'a StrippedIndexedObject<T, B, M>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}
pub struct IntoIter<T, B, M>(indexmap::set::IntoIter<StrippedIndexedObject<T, B, M>>);

impl<T, B, M> Iterator for IntoIter<T, B, M> {
	type Item = IndexedObject<T, B, M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|s| s.0)
	}
}

impl<T: Hash + Eq, B: Hash + Eq, M> FromIterator<IndexedObject<T, B, M>>
	for ExpandedDocument<T, B, M>
{
	fn from_iter<I: IntoIterator<Item = IndexedObject<T, B, M>>>(iter: I) -> Self {
		Self(iter.into_iter().map(locspan::Stripped).collect())
	}
}

impl<T: Hash + Eq, B: Hash + Eq, M> Extend<IndexedObject<T, B, M>> for ExpandedDocument<T, B, M> {
	fn extend<I: IntoIterator<Item = IndexedObject<T, B, M>>>(&mut self, iter: I) {
		self.0.extend(iter.into_iter().map(locspan::Stripped))
	}
}

impl<T, B, M> From<IndexSet<StrippedIndexedObject<T, B, M>>> for ExpandedDocument<T, B, M> {
	fn from(set: IndexSet<StrippedIndexedObject<T, B, M>>) -> Self {
		Self(set)
	}
}
