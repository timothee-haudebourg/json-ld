use crate::object::{FragmentRef, InvalidExpandedJson, Traverse};
use crate::{id, Indexed, Reference, StrippedIndexedObject};
use crate::{IndexedObject, TryFromJson};
use locspan::{Location, Meta, StrippedEq, StrippedPartialEq};
use rdf_types::vocabulary::{Index, VocabularyMut};
use std::collections::HashSet;
use std::hash::Hash;

/// Result of the document expansion algorithm.
///
/// It is just an alias for a set of (indexed) objects.
pub struct ExpandedDocument<T = Index, B = Index, M = Location<T>>(
	HashSet<StrippedIndexedObject<T, B, M>>,
);

impl<T, B, M> Default for ExpandedDocument<T, B, M> {
	#[inline(always)]
	fn default() -> Self {
		Self(HashSet::new())
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
	pub fn objects(&self) -> &HashSet<StrippedIndexedObject<T, B, M>> {
		&self.0
	}

	#[inline(always)]
	pub fn into_objects(self) -> HashSet<StrippedIndexedObject<T, B, M>> {
		self.0
	}

	#[inline(always)]
	pub fn iter(&self) -> std::collections::hash_set::Iter<'_, StrippedIndexedObject<T, B, M>> {
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

	#[inline(always)]
	pub fn identify_all_in<N, G: id::Generator<T, B, M, N>>(
		&mut self,
		vocabulary: &mut N,
		generator: &mut G,
	) where
		M: Clone,
		T: Eq + Hash,
		B: Eq + Hash,
	{
		let mut objects = HashSet::new();
		std::mem::swap(&mut self.0, &mut objects);

		for mut object in objects {
			object.identify_all_in(vocabulary, generator);
			self.0.insert(object);
		}
	}

	#[inline(always)]
	pub fn identify_all<G: id::Generator<T, B, M, ()>>(&mut self, generator: &mut G)
	where
		M: Clone,
		T: Eq + Hash,
		B: Eq + Hash,
	{
		self.identify_all_in(&mut (), generator)
	}

	/// Returns the set of all blank identifiers in the given document.
	pub fn blank_ids(&self) -> HashSet<&B>
	where
		B: Eq + Hash,
	{
		self.traverse()
			.filter_map(|f| f.into_id().and_then(Reference::into_blank))
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
		vocabulary: &mut impl VocabularyMut<Iri=T, BlankId=B>,
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
	type IntoIter = std::collections::hash_set::Iter<'a, StrippedIndexedObject<T, B, M>>;
	type Item = &'a StrippedIndexedObject<T, B, M>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}
pub struct IntoIter<T, B, M>(std::collections::hash_set::IntoIter<StrippedIndexedObject<T, B, M>>);

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

impl<T, B, M> From<HashSet<StrippedIndexedObject<T, B, M>>> for ExpandedDocument<T, B, M> {
	fn from(set: HashSet<StrippedIndexedObject<T, B, M>>) -> Self {
		Self(set)
	}
}

// impl<F, J: JsonHash + JsonClone, K: JsonFrom<J>, T: Id> AsJson<J, K> for ExpandedDocument<T, B, M> {
// 	fn as_json_with(
// 		&self,
// 		meta: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
// 	) -> K {
// 		self.0.as_json_with(meta)
// 	}
// }
