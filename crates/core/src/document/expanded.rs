use crate::object::{FragmentRef, InvalidExpandedJson, Traverse};
use crate::{Id, Indexed, IndexedObject, Node, Object, Relabel, TryFromJson};
use hashbrown::HashMap;
use indexmap::IndexSet;
use iref::IriBuf;
use rdf_types::vocabulary::VocabularyMut;
use rdf_types::{BlankIdBuf, Generator, Vocabulary};
use std::collections::HashSet;
use std::hash::Hash;

/// Result of the document expansion algorithm.
///
/// It is just an alias for a set of (indexed) objects.
#[derive(Debug, Clone)]
pub struct ExpandedDocument<T = IriBuf, B = BlankIdBuf>(IndexSet<IndexedObject<T, B>>);

impl<T, B> Default for ExpandedDocument<T, B> {
	#[inline(always)]
	fn default() -> Self {
		Self(IndexSet::new())
	}
}

impl<T, B> ExpandedDocument<T, B> {
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
	pub fn objects(&self) -> &IndexSet<IndexedObject<T, B>> {
		&self.0
	}

	#[inline(always)]
	pub fn into_objects(self) -> IndexSet<IndexedObject<T, B>> {
		self.0
	}

	#[inline(always)]
	pub fn iter(&self) -> indexmap::set::Iter<'_, IndexedObject<T, B>> {
		self.0.iter()
	}

	#[inline(always)]
	pub fn traverse(&self) -> Traverse<T, B> {
		Traverse::new(self.iter().map(|o| FragmentRef::IndexedObject(o)))
	}

	#[inline(always)]
	pub fn count(&self, f: impl FnMut(&FragmentRef<T, B>) -> bool) -> usize {
		self.traverse().filter(f).count()
	}

	/// Give an identifier (`@id`) to every nodes using the given generator to
	/// generate fresh identifiers for anonymous nodes.
	#[inline(always)]
	pub fn identify_all_with<V: Vocabulary<Iri = T, BlankId = B>, G: Generator<V>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut G,
	) where
		T: Eq + Hash,
		B: Eq + Hash,
	{
		let objects = std::mem::take(&mut self.0);
		for mut object in objects {
			object.identify_all_with(vocabulary, generator);
			self.0.insert(object);
		}
	}

	/// Give an identifier (`@id`) to every nodes using the given generator to
	/// generate fresh identifiers for anonymous nodes.
	#[inline(always)]
	pub fn identify_all<G: Generator>(&mut self, generator: &mut G)
	where
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
	pub fn relabel_and_canonicalize_with<V: Vocabulary<Iri = T, BlankId = B>, G: Generator<V>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut G,
	) where
		T: Clone + Eq + Hash,
		B: Clone + Eq + Hash,
	{
		let objects = std::mem::take(&mut self.0);
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
	pub fn relabel_and_canonicalize<G: Generator>(&mut self, generator: &mut G)
	where
		T: Clone + Eq + Hash,
		B: Clone + Eq + Hash,
		(): Vocabulary<Iri = T, BlankId = B>,
	{
		self.relabel_and_canonicalize_with(&mut (), generator)
	}

	/// Relabels nodes.
	#[inline(always)]
	pub fn relabel_with<V: Vocabulary<Iri = T, BlankId = B>, G: Generator<V>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut G,
	) where
		T: Clone + Eq + Hash,
		B: Clone + Eq + Hash,
	{
		let objects = std::mem::take(&mut self.0);
		let mut relabeling = HashMap::new();
		for mut object in objects {
			object.relabel_with(vocabulary, generator, &mut relabeling);
			self.0.insert(object);
		}
	}

	/// Relabels nodes.
	#[inline(always)]
	pub fn relabel<G: Generator>(&mut self, generator: &mut G)
	where
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
		let objects = std::mem::take(&mut self.0);
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

	/// Map the identifiers present in this expanded document (recursively).
	pub fn map_ids<U, C>(
		self,
		mut map_iri: impl FnMut(T) -> U,
		mut map_id: impl FnMut(Id<T, B>) -> Id<U, C>,
	) -> ExpandedDocument<U, C>
	where
		U: Eq + Hash,
		C: Eq + Hash,
	{
		ExpandedDocument(
			self.0
				.into_iter()
				.map(|i| i.map_inner(|o| o.map_ids(&mut map_iri, &mut map_id)))
				.collect(),
		)
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

	/// Returns the main node object of the document, if any.
	///
	/// The main node is the unique top level (root) node object. If multiple
	/// node objects are on the root, `None` is returned.
	pub fn main_node(&self) -> Option<&Node<T, B>> {
		let mut result = None;

		for object in self {
			if let Object::Node(node) = object.inner() {
				if result.is_some() {
					return None;
				}

				result = Some(&**node)
			}
		}

		result
	}

	/// Consumes the document and returns its main node object, if any.
	///
	/// The main node is the unique top level (root) node object. If multiple
	/// node objects are on the root, `None` is returned.
	pub fn into_main_node(self) -> Option<Node<T, B>> {
		let mut result = None;

		for object in self {
			if let Object::Node(node) = object.into_inner() {
				if result.is_some() {
					return None;
				}

				result = Some(*node)
			}
		}

		result
	}
}

impl<T: Hash + Eq, B: Hash + Eq> ExpandedDocument<T, B> {
	#[inline(always)]
	pub fn insert(&mut self, object: IndexedObject<T, B>) -> bool {
		self.0.insert(object)
	}
}

impl<T: Eq + Hash, B: Eq + Hash> From<Indexed<Node<T, B>>> for ExpandedDocument<T, B> {
	fn from(value: Indexed<Node<T, B>>) -> Self {
		let mut result = Self::default();

		result.insert(value.map_inner(Object::node));

		result
	}
}

impl<T: Eq + Hash, B: Eq + Hash> TryFromJson<T, B> for ExpandedDocument<T, B> {
	fn try_from_json_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		value: json_syntax::Value,
	) -> Result<Self, InvalidExpandedJson> {
		match value {
			json_syntax::Value::Array(items) => {
				let mut result = Self::new();

				for item in items {
					result.insert(Indexed::try_from_json_in(vocabulary, item)?);
				}

				Ok(result)
			}
			other => Err(InvalidExpandedJson::Unexpected(
				other.kind(),
				json_syntax::Kind::Array,
			)),
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash> PartialEq for ExpandedDocument<T, B> {
	/// Comparison between two expanded documents.
	fn eq(&self, other: &Self) -> bool {
		self.0.eq(&other.0)
	}
}

impl<T: Eq + Hash, B: Eq + Hash> Eq for ExpandedDocument<T, B> {}

impl<T, B> IntoIterator for ExpandedDocument<T, B> {
	type IntoIter = IntoIter<T, B>;
	type Item = IndexedObject<T, B>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		IntoIter(self.0.into_iter())
	}
}

impl<'a, T, B> IntoIterator for &'a ExpandedDocument<T, B> {
	type IntoIter = indexmap::set::Iter<'a, IndexedObject<T, B>>;
	type Item = &'a IndexedObject<T, B>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}
pub struct IntoIter<T, B>(indexmap::set::IntoIter<IndexedObject<T, B>>);

impl<T, B> Iterator for IntoIter<T, B> {
	type Item = IndexedObject<T, B>;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

impl<T: Hash + Eq, B: Hash + Eq> FromIterator<IndexedObject<T, B>> for ExpandedDocument<T, B> {
	fn from_iter<I: IntoIterator<Item = IndexedObject<T, B>>>(iter: I) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl<T: Hash + Eq, B: Hash + Eq> Extend<IndexedObject<T, B>> for ExpandedDocument<T, B> {
	fn extend<I: IntoIterator<Item = IndexedObject<T, B>>>(&mut self, iter: I) {
		self.0.extend(iter)
	}
}

impl<T, B> From<IndexSet<IndexedObject<T, B>>> for ExpandedDocument<T, B> {
	fn from(set: IndexSet<IndexedObject<T, B>>) -> Self {
		Self(set)
	}
}
