use super::{Multiset, Nodes};
use crate::{Id, IndexedNode};
use educe::Educe;
use indexmap::IndexMap;
use std::hash::{Hash, Hasher};

pub type ReversePropertyNodes = Multiset<IndexedNode>;

/// Reverse properties of a node object, and their associated nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReverseProperties(IndexMap<Id, ReversePropertyNodes>);

impl Default for ReverseProperties {
	fn default() -> Self {
		Self::new()
	}
}

impl ReverseProperties {
	/// Creates an empty map.
	pub fn new() -> Self {
		Self(IndexMap::new())
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
	pub fn iter(&self) -> Iter<'_> {
		Iter {
			inner: self.0.iter(),
		}
	}

	/// Returns an iterator over the reverse properties with a mutable reference to their associated nodes.
	#[inline(always)]
	pub fn iter_mut(&mut self) -> IterMut<'_> {
		self.0.iter_mut()
	}

	/// Removes all reverse properties.
	#[inline(always)]
	pub fn clear(&mut self) {
		self.0.clear()
	}
}

impl ReverseProperties {
	/// Checks if the given reverse property is associated to any node.
	#[inline(always)]
	pub fn contains<Q: ?Sized + Hash + indexmap::Equivalent<Id>>(&self, prop: &Q) -> bool {
		self.0.get(prop).is_some()
	}

	/// Returns an iterator over all the nodes associated to the given reverse property.
	#[inline(always)]
	pub fn get<'a, Q: ?Sized + Hash + indexmap::Equivalent<Id>>(&self, prop: &Q) -> Nodes {
		match self.0.get(prop) {
			Some(values) => Nodes::new(Some(values.iter())),
			None => Nodes::new(None),
		}
	}

	/// Get one of the nodes associated to the given reverse property.
	///
	/// If multiple nodes are found, there are no guaranties on which node will be returned.
	#[inline(always)]
	pub fn get_any<'a, Q: ?Sized + Hash + indexmap::Equivalent<Id>>(
		&self,
		prop: &Q,
	) -> Option<&IndexedNode> {
		match self.0.get(prop) {
			Some(values) => values.iter().next(),
			None => None,
		}
	}

	/// Associate the given node to the given reverse property.
	#[inline(always)]
	pub fn insert(&mut self, prop: Id, value: IndexedNode) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			node_values.insert(value);
		} else {
			self.0.insert(prop, Multiset::singleton(value));
		}
	}

	/// Associate the given node to the given reverse property, unless it is already.
	#[inline(always)]
	pub fn insert_unique(&mut self, prop: Id, value: IndexedNode) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			if node_values.iter().all(|v| !v.equivalent(&value)) {
				node_values.insert(value)
			}
		} else {
			self.0.insert(prop, Multiset::singleton(value));
		}
	}

	/// Associate all the given nodes to the given reverse property.
	#[inline(always)]
	pub fn insert_all<Objects: IntoIterator<Item = IndexedNode>>(
		&mut self,
		prop: Id,
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
	pub fn insert_all_unique<Nodes: IntoIterator<Item = IndexedNode>>(
		&mut self,
		prop: Id,
		values: Nodes,
	) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			for value in values {
				if node_values.iter().all(|v| !v.equivalent(&value)) {
					node_values.insert(value)
				}
			}
		} else {
			let values = values.into_iter();
			let mut node_values: ReversePropertyNodes =
				Multiset::with_capacity(values.size_hint().0);
			for value in values {
				if node_values.iter().all(|v| !v.equivalent(&value)) {
					node_values.insert(value)
				}
			}

			self.0.insert(prop, node_values);
		}
	}

	pub fn set(&mut self, prop: Id, values: ReversePropertyNodes) {
		self.0.insert(prop, values);
	}

	pub fn extend_unique<N>(&mut self, iter: impl IntoIterator<Item = (Id, N)>)
	where
		N: IntoIterator<Item = IndexedNode>,
	{
		for (prop, values) in iter {
			self.insert_all_unique(prop, values)
		}
	}

	/// Removes and returns all the values associated to the given reverse property.
	#[inline(always)]
	pub fn remove(&mut self, prop: &Id) -> Option<ReversePropertyNodes> {
		self.0.swap_remove(prop)
	}
}

impl<N> FromIterator<(Id, N)> for ReverseProperties
where
	N: IntoIterator<Item = IndexedNode>,
{
	fn from_iter<I: IntoIterator<Item = (Id, N)>>(iter: I) -> Self {
		let mut result = Self::default();
		for (id, values) in iter {
			result.insert_all(id, values);
		}
		result
	}
}

impl Hash for ReverseProperties {
	#[inline(always)]
	fn hash<H: Hasher>(&self, h: &mut H) {
		crate::utils::hash_map(&self.0, h)
	}
}

impl Extend<(Id, Vec<IndexedNode>)> for ReverseProperties {
	fn extend<I>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (Id, Vec<IndexedNode>)>,
	{
		for (prop, values) in iter {
			self.insert_all(prop, values)
		}
	}
}

/// Tuple type representing a reverse binding in a node object,
/// associating a reverse property to some nodes.
pub type ReverseBinding = (Id, ReversePropertyNodes);

/// Tuple type representing a reference to a reverse binding in a node object,
/// associating a reverse property to some nodes.
pub type ReverseBindingRef<'a> = (&'a Id, &'a [IndexedNode]);

/// Tuple type representing a mutable reference to a reverse binding in a node object,
/// associating a reverse property to some nodes, with a mutable access to the nodes.
pub type ReverseBindingMut<'a> = (&'a Id, &'a mut ReversePropertyNodes);

impl IntoIterator for ReverseProperties {
	type Item = ReverseBinding;
	type IntoIter = IntoIter;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<'a> IntoIterator for &'a ReverseProperties {
	type Item = ReverseBindingRef<'a>;
	type IntoIter = Iter<'a>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a> IntoIterator for &'a mut ReverseProperties {
	type Item = ReverseBindingMut<'a>;
	type IntoIter = IterMut<'a>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

/// Iterator over the reverse properties of a node.
///
/// It is created by the [`ReverseProperties::into_iter`] function.
pub type IntoIter = indexmap::map::IntoIter<Id, ReversePropertyNodes>;

/// Iterator over the reverse properties of a node.
///
/// It is created by the [`ReverseProperties::iter`] function.
#[derive(Educe)]
#[educe(Clone)]
pub struct Iter<'a> {
	inner: indexmap::map::Iter<'a, Id, ReversePropertyNodes>,
}

impl<'a> Iterator for Iter<'a> {
	type Item = ReverseBindingRef<'a>;

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

impl<'a> ExactSizeIterator for Iter<'a> {}

impl<'a> std::iter::FusedIterator for Iter<'a> {}

/// Iterator over the reverse properties of a node, giving a mutable reference
/// to the associated nodes.
///
/// It is created by the [`ReverseProperties::iter_mut`] function.
pub type IterMut<'a> = indexmap::map::IterMut<'a, Id, ReversePropertyNodes>;
