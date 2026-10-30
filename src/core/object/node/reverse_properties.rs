use super::Nodes;
use crate::{
	IndexedJsonLdNode, JsonLdLenient, JsonLdVisit,
	object::{JsonLdObjectMut, JsonLdObjectRef},
};
use btree_indexmap::{BTreeIndexMap, BTreeIndexMultiSet, Comparable};
use educe::Educe;
use rdf_syntax::Id;
use std::hash::Hash;

pub type JsonLdReversePropertyNodes = BTreeIndexMultiSet<IndexedJsonLdNode>;

/// Reverse properties of a node object, and their associated nodes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct JsonLdReverseProperties(BTreeIndexMap<JsonLdLenient<Id>, JsonLdReversePropertyNodes>);

impl Default for JsonLdReverseProperties {
	fn default() -> Self {
		Self::new()
	}
}

impl JsonLdReverseProperties {
	/// Creates an empty map.
	pub fn new() -> Self {
		Self(BTreeIndexMap::new())
	}

	/// Returns the number of reverse properties.
	#[inline(always)]
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Checks if there are no defined reverse properties.
	#[inline(always)]
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	/// Returns an iterator over the reverse properties and their associated nodes.
	#[inline(always)]
	pub fn iter(&self) -> JsonLdReversePropertiesIter<'_> {
		JsonLdReversePropertiesIter {
			inner: self.0.iter(),
		}
	}

	/// Returns an iterator over the reverse properties with a mutable reference to their associated nodes.
	#[inline(always)]
	pub fn iter_mut(&mut self) -> JsonLdReversePropertiesIterMut<'_> {
		self.0.iter_mut()
	}

	/// Removes all reverse properties.
	#[inline(always)]
	pub fn clear(&mut self) {
		self.0.clear()
	}

	/// Checks if the given reverse property is associated to any node.
	#[inline(always)]
	pub fn contains<Q: ?Sized + Hash + Comparable<JsonLdLenient<Id>>>(&self, prop: &Q) -> bool {
		self.0.get(prop).is_some()
	}

	/// Returns an iterator over all the nodes associated to the given reverse property.
	#[inline(always)]
	pub fn get<Q: ?Sized + Hash + Comparable<JsonLdLenient<Id>>>(&self, prop: &Q) -> Nodes<'_> {
		match self.0.get(prop) {
			Some(values) => Nodes::new(Some(values.iter())),
			None => Nodes::new(None),
		}
	}

	/// Get one of the nodes associated to the given reverse property.
	///
	/// If multiple nodes are found, there are no guaranties on which node will be returned.
	#[inline(always)]
	pub fn get_any<Q: ?Sized + Hash + Comparable<JsonLdLenient<Id>>>(
		&self,
		prop: &Q,
	) -> Option<&IndexedJsonLdNode> {
		match self.0.get(prop) {
			Some(values) => values.iter().next(),
			None => None,
		}
	}

	/// Associate the given node to the given reverse property.
	#[inline(always)]
	pub fn insert(&mut self, prop: JsonLdLenient<Id>, value: IndexedJsonLdNode) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			node_values.insert(value);
		} else {
			self.0.insert(prop, BTreeIndexMultiSet::singleton(value));
		}
	}

	/// Associate the given node to the given reverse property, unless it is already.
	#[inline(always)]
	pub fn insert_unique(&mut self, prop: JsonLdLenient<Id>, value: IndexedJsonLdNode) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			if node_values.iter().all(|v| !v.equivalent(&value)) {
				node_values.insert(value);
			}
		} else {
			self.0.insert(prop, BTreeIndexMultiSet::singleton(value));
		}
	}

	/// Associate all the given nodes to the given reverse property.
	#[inline(always)]
	pub fn insert_all<Objects: IntoIterator<Item = IndexedJsonLdNode>>(
		&mut self,
		prop: JsonLdLenient<Id>,
		values: Objects,
	) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			node_values.extend(values);
		} else {
			self.0.insert(prop, values.into_iter().collect());
		}
	}

	/// Associate all the given nodes to the given reverse property, unless it is already.
	#[inline(always)]
	pub fn insert_all_unique<Nodes: IntoIterator<Item = IndexedJsonLdNode>>(
		&mut self,
		prop: JsonLdLenient<Id>,
		values: Nodes,
	) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			for value in values {
				if node_values.iter().all(|v| !v.equivalent(&value)) {
					node_values.insert(value);
				}
			}
		} else {
			let values = values.into_iter();
			let mut node_values: JsonLdReversePropertyNodes =
				BTreeIndexMultiSet::with_capacity(values.size_hint().0);
			for value in values {
				if node_values.iter().all(|v| !v.equivalent(&value)) {
					node_values.insert(value);
				}
			}

			self.0.insert(prop, node_values);
		}
	}

	pub fn set(&mut self, prop: JsonLdLenient<Id>, values: JsonLdReversePropertyNodes) {
		self.0.insert(prop, values);
	}

	pub fn extend_unique<N>(&mut self, iter: impl IntoIterator<Item = (JsonLdLenient<Id>, N)>)
	where
		N: IntoIterator<Item = IndexedJsonLdNode>,
	{
		for (prop, values) in iter {
			self.insert_all_unique(prop, values)
		}
	}

	/// Removes and returns all the values associated to the given reverse property.
	#[inline(always)]
	pub fn remove(&mut self, prop: &JsonLdLenient<Id>) -> Option<JsonLdReversePropertyNodes> {
		self.0.swap_remove(prop)
	}
}

impl JsonLdVisit for JsonLdReverseProperties {
	fn visit_with(&self, f: &mut impl FnMut(JsonLdObjectRef)) {
		for (_, t) in self {
			t.visit_with(f);
		}
	}

	fn visit_mut_with(&mut self, f: &mut impl FnMut(JsonLdObjectMut)) {
		for (_, t) in self {
			t.visit_mut_with(f);
		}
	}
}

impl<N> FromIterator<(JsonLdLenient<Id>, N)> for JsonLdReverseProperties
where
	N: IntoIterator<Item = IndexedJsonLdNode>,
{
	fn from_iter<I: IntoIterator<Item = (JsonLdLenient<Id>, N)>>(iter: I) -> Self {
		let mut result = Self::default();
		for (id, values) in iter {
			result.insert_all(id, values);
		}
		result
	}
}

impl Extend<(JsonLdLenient<Id>, Vec<IndexedJsonLdNode>)> for JsonLdReverseProperties {
	fn extend<I>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (JsonLdLenient<Id>, Vec<IndexedJsonLdNode>)>,
	{
		for (prop, values) in iter {
			self.insert_all(prop, values)
		}
	}
}

/// Tuple type representing a reverse binding in a node object,
/// associating a reverse property to some nodes.
pub type JsonLdReverseBinding = (JsonLdLenient<Id>, JsonLdReversePropertyNodes);

/// Tuple type representing a reference to a reverse binding in a node object,
/// associating a reverse property to some nodes.
pub type JsonLdReverseBindingRef<'a> = (&'a JsonLdLenient<Id>, &'a [IndexedJsonLdNode]);

/// Tuple type representing a mutable reference to a reverse binding in a node object,
/// associating a reverse property to some nodes, with a mutable access to the nodes.
pub type JsonLdReverseBindingMut<'a> = (&'a JsonLdLenient<Id>, &'a mut JsonLdReversePropertyNodes);

impl IntoIterator for JsonLdReverseProperties {
	type Item = JsonLdReverseBinding;
	type IntoIter = JsonLdReversePropertiesIntoIter;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<'a> IntoIterator for &'a JsonLdReverseProperties {
	type Item = JsonLdReverseBindingRef<'a>;
	type IntoIter = JsonLdReversePropertiesIter<'a>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a> IntoIterator for &'a mut JsonLdReverseProperties {
	type Item = JsonLdReverseBindingMut<'a>;
	type IntoIter = JsonLdReversePropertiesIterMut<'a>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

/// Iterator over the reverse properties of a node.
///
/// It is created by the [`ReverseProperties::into_iter`] function.
pub type JsonLdReversePropertiesIntoIter =
	btree_indexmap::map::IntoIter<JsonLdLenient<Id>, JsonLdReversePropertyNodes>;

/// Iterator over the reverse properties of a node.
///
/// It is created by the [`ReverseProperties::iter`] function.
#[derive(Educe)]
#[educe(Clone)]
pub struct JsonLdReversePropertiesIter<'a> {
	inner: btree_indexmap::map::Iter<'a, JsonLdLenient<Id>, JsonLdReversePropertyNodes>,
}

impl<'a> Iterator for JsonLdReversePropertiesIter<'a> {
	type Item = JsonLdReverseBindingRef<'a>;

	#[inline(always)]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.inner.size_hint()
	}

	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item> {
		self.inner
			.next()
			.map(|(property, objects)| (property, objects.as_slice()))
	}
}

impl<'a> ExactSizeIterator for JsonLdReversePropertiesIter<'a> {}

impl<'a> std::iter::FusedIterator for JsonLdReversePropertiesIter<'a> {}

/// Iterator over the reverse properties of a node, giving a mutable reference
/// to the associated nodes.
///
/// It is created by the [`ReverseProperties::iter_mut`] function.
pub type JsonLdReversePropertiesIterMut<'a> =
	btree_indexmap::map::IterMut<'a, JsonLdLenient<Id>, JsonLdReversePropertyNodes>;
