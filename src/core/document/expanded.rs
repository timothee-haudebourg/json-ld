use crate::{Indexed, IndexedObject, Node, Object};
use indexmap::IndexSet;

/// Result of the document expansion algorithm.
///
/// It is just an alias for a set of (indexed) objects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpandedDocument(IndexSet<IndexedObject>);

impl Default for ExpandedDocument {
	#[inline(always)]
	fn default() -> Self {
		Self(IndexSet::new())
	}
}

impl ExpandedDocument {
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
	pub fn objects(&self) -> &IndexSet<IndexedObject> {
		&self.0
	}

	#[inline(always)]
	pub fn into_objects(self) -> IndexSet<IndexedObject> {
		self.0
	}

	#[inline(always)]
	pub fn iter(&self) -> indexmap::set::Iter<'_, IndexedObject> {
		self.0.iter()
	}

	// /// Give an identifier (`@id`) to every nodes using the given generator to
	// /// generate fresh identifiers for anonymous nodes.
	// #[inline(always)]
	// pub fn identify_all_with<V: Vocabulary<Iri = T, BlankId = B>, G: Generator<V>>(
	// 	&mut self,
	// 	vocabulary: &mut V,
	// 	generator: &mut G,
	// ) where
	// 	T: Eq + Hash,
	// 	B: Eq + Hash,
	// {
	// 	let objects = std::mem::take(&mut self.0);
	// 	for mut object in objects {
	// 		object.identify_all_with(vocabulary, generator);
	// 		self.0.insert(object);
	// 	}
	// }

	// /// Give an identifier (`@id`) to every nodes using the given generator to
	// /// generate fresh identifiers for anonymous nodes.
	// #[inline(always)]
	// pub fn identify_all<G: Generator>(&mut self, generator: &mut G)
	// where
	// 	T: Eq + Hash,
	// 	B: Eq + Hash,
	// 	(): Vocabulary<Iri = T, BlankId = B>,
	// {
	// 	self.identify_all_with(&mut (), generator)
	// }

	// /// Give an identifier (`@id`) to every nodes and canonicalize every
	// /// literals using the given generator to generate fresh identifiers for
	// /// anonymous nodes.
	// #[inline(always)]
	// pub fn relabel_and_canonicalize_with<V: Vocabulary<Iri = T, BlankId = B>, G: Generator<V>>(
	// 	&mut self,
	// 	vocabulary: &mut V,
	// 	generator: &mut G,
	// ) where
	// 	T: Clone + Eq + Hash,
	// 	B: Clone + Eq + Hash,
	// {
	// 	let objects = std::mem::take(&mut self.0);
	// 	let mut relabeling = HashMap::new();
	// 	let mut buffer = ryu_js::Buffer::new();
	// 	for mut object in objects {
	// 		object.relabel_with(vocabulary, generator, &mut relabeling);
	// 		object.canonicalize_with(&mut buffer);
	// 		self.0.insert(object);
	// 	}
	// }

	// /// Give an identifier (`@id`) to every nodes and canonicalize every
	// /// literals using the given generator to generate fresh identifiers for
	// /// anonymous nodes.
	// #[inline(always)]
	// pub fn relabel_and_canonicalize<G: Generator>(&mut self, generator: &mut G)
	// where
	// 	T: Clone + Eq + Hash,
	// 	B: Clone + Eq + Hash,
	// 	(): Vocabulary<Iri = T, BlankId = B>,
	// {
	// 	self.relabel_and_canonicalize_with(&mut (), generator)
	// }

	// /// Relabels nodes.
	// #[inline(always)]
	// pub fn relabel_with<V: Vocabulary<Iri = T, BlankId = B>, G: Generator<V>>(
	// 	&mut self,
	// 	vocabulary: &mut V,
	// 	generator: &mut G,
	// ) where
	// 	T: Clone + Eq + Hash,
	// 	B: Clone + Eq + Hash,
	// {
	// 	let objects = std::mem::take(&mut self.0);
	// 	let mut relabeling = HashMap::new();
	// 	for mut object in objects {
	// 		object.relabel_with(vocabulary, generator, &mut relabeling);
	// 		self.0.insert(object);
	// 	}
	// }

	// /// Relabels nodes.
	// #[inline(always)]
	// pub fn relabel<G: Generator>(&mut self, generator: &mut G)
	// where
	// 	T: Clone + Eq + Hash,
	// 	B: Clone + Eq + Hash,
	// 	(): Vocabulary<Iri = T, BlankId = B>,
	// {
	// 	self.relabel_with(&mut (), generator)
	// }

	// /// Puts this document literals into canonical form using the given
	// /// `buffer`.
	// ///
	// /// The buffer is used to compute the canonical form of numbers.
	// pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer)
	// where
	// 	T: Eq + Hash,
	// 	B: Eq + Hash,
	// {
	// 	let objects = std::mem::take(&mut self.0);
	// 	for mut object in objects {
	// 		object.canonicalize_with(buffer);
	// 		self.0.insert(object);
	// 	}
	// }

	// /// Puts this document literals into canonical form.
	// pub fn canonicalize(&mut self)
	// where
	// 	T: Eq + Hash,
	// 	B: Eq + Hash,
	// {
	// 	let mut buffer = ryu_js::Buffer::new();
	// 	self.canonicalize_with(&mut buffer)
	// }

	// /// Map the identifiers present in this expanded document (recursively).
	// pub fn map_ids(
	// 	self,
	// 	mut map_iri: impl FnMut(IriBuf) -> IriBuf,
	// 	mut map_id: impl FnMut(Id) -> Id<U, C>,
	// ) -> ExpandedDocument<U, C>
	// where
	// 	U: Eq + Hash,
	// 	C: Eq + Hash,
	// {
	// 	ExpandedDocument(
	// 		self.0
	// 			.into_iter()
	// 			.map(|i| i.map_inner(|o| o.map_ids(&mut map_iri, &mut map_id)))
	// 			.collect(),
	// 	)
	// }

	// /// Returns the set of all blank identifiers in the given document.
	// pub fn blank_ids(&self) -> HashSet<&BlankId> {
	// 	self.traverse()
	// 		.filter_map(|f| f.into_id().and_then(Id::into_blank))
	// 		.collect()
	// }

	/// Returns the main node object of the document, if any.
	///
	/// The main node is the unique top level (root) node object. If multiple
	/// node objects are on the root, `None` is returned.
	pub fn main_node(&self) -> Option<&Node> {
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
	pub fn into_main_node(self) -> Option<Node> {
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

	#[inline(always)]
	pub fn insert(&mut self, object: IndexedObject) -> bool {
		self.0.insert(object)
	}
}

impl From<Indexed<Node>> for ExpandedDocument {
	fn from(value: Indexed<Node>) -> Self {
		let mut result = Self::default();

		result.insert(value.map_inner(Object::node));

		result
	}
}

impl IntoIterator for ExpandedDocument {
	type IntoIter = IntoIter;
	type Item = IndexedObject;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		IntoIter(self.0.into_iter())
	}
}

impl<'a> IntoIterator for &'a ExpandedDocument {
	type IntoIter = indexmap::set::Iter<'a, IndexedObject>;
	type Item = &'a IndexedObject;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}
pub struct IntoIter(indexmap::set::IntoIter<IndexedObject>);

impl Iterator for IntoIter {
	type Item = IndexedObject;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

impl FromIterator<IndexedObject> for ExpandedDocument {
	fn from_iter<I: IntoIterator<Item = IndexedObject>>(iter: I) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl Extend<IndexedObject> for ExpandedDocument {
	fn extend<I: IntoIterator<Item = IndexedObject>>(&mut self, iter: I) {
		self.0.extend(iter)
	}
}

impl From<IndexSet<IndexedObject>> for ExpandedDocument {
	fn from(set: IndexSet<IndexedObject>) -> Self {
		Self(set)
	}
}
