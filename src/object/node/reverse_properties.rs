use super::{Node, Nodes};
use crate::{Id, Indexed, Reference, ToReference};
use generic_json::JsonHash;
use std::{
	borrow::Borrow,
	collections::HashMap,
	hash::{Hash, Hasher},
};

/// Reverse properties of a node object, and their associated nodes.
#[derive(PartialEq, Eq)]
pub struct ReverseProperties<J: JsonHash, T: Id>(HashMap<Reference<T>, Vec<Indexed<Node<J, T>>>>);

impl<J: JsonHash, T: Id> ReverseProperties<J, T> {
	/// Creates an empty map.
	pub(crate) fn new() -> Self {
		Self(HashMap::new())
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

	/// Checks if the given reverse property is associated to any node.
	#[inline(always)]
	pub fn contains<Q: ToReference<T>>(&self, prop: Q) -> bool {
		self.0.get(prop.to_ref().borrow()).is_some()
	}

	/// Returns an iterator over all the nodes associated to the given reverse property.
	#[inline(always)]
	pub fn get<'a, Q: ToReference<T>>(&self, prop: Q) -> Nodes<J, T>
	where
		T: 'a,
	{
		match self.0.get(prop.to_ref().borrow()) {
			Some(values) => Nodes::new(Some(values.iter())),
			None => Nodes::new(None),
		}
	}

	/// Get one of the nodes associated to the given reverse property.
	///
	/// If multiple nodes are found, there are no guaranties on which node will be returned.
	#[inline(always)]
	pub fn get_any<'a, Q: ToReference<T>>(&self, prop: Q) -> Option<&Indexed<Node<J, T>>>
	where
		T: 'a,
	{
		match self.0.get(prop.to_ref().borrow()) {
			Some(values) => values.iter().next(),
			None => None,
		}
	}

	/// Associate the given node to the given reverse property.
	#[inline(always)]
	pub fn insert(&mut self, prop: Reference<T>, value: Indexed<Node<J, T>>) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			node_values.push(value);
		} else {
			let node_values = vec![value];
			self.0.insert(prop, node_values);
		}
	}

	/// Associate all the given nodes to the given reverse property.
	#[inline(always)]
	pub fn insert_all<Objects: IntoIterator<Item = Indexed<Node<J, T>>>>(
		&mut self,
		prop: Reference<T>,
		values: Objects,
	) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			node_values.extend(values);
		} else {
			self.0.insert(prop, values.into_iter().collect());
		}
	}

	/// Returns an iterator over the reverse properties and their associated nodes.
	#[inline(always)]
	pub fn iter(&self) -> Iter<'_, J, T> {
		Iter {
			inner: self.0.iter(),
		}
	}

	/// Returns an iterator over the reverse properties with a mutable reference to their associated nodes.
	#[inline(always)]
	pub fn iter_mut(&mut self) -> IterMut<'_, J, T> {
		IterMut {
			inner: self.0.iter_mut(),
		}
	}
}

impl<J: JsonHash, T: Id> Hash for ReverseProperties<J, T> {
	#[inline(always)]
	fn hash<H: Hasher>(&self, h: &mut H) {
		crate::util::hash_map(&self.0, h)
	}
}

impl<J: JsonHash, T: Id> Extend<(Reference<T>, Vec<Indexed<Node<J, T>>>)> for ReverseProperties<J, T> {
	fn extend<I>(&mut self, iter: I)
    where
		I: IntoIterator<Item = (Reference<T>, Vec<Indexed<Node<J, T>>>)>
	{
		for (prop, values) in iter {
			self.insert_all(prop, values)
		}
	}
}

/// Tuple type representing a reverse binding in a node object,
/// associating a reverse property to some nodes.
pub type ReverseBinding<J, T> = (Reference<T>, Vec<Indexed<Node<J, T>>>);

/// Tuple type representing a reference to a reverse binding in a node object,
/// associating a reverse property to some nodes.
pub type ReverseBindingRef<'a, J, T> = (&'a Reference<T>, &'a [Indexed<Node<J, T>>]);

/// Tuple type representing a mutable reference to a reverse binding in a node object,
/// associating a reverse property to some nodes, with a mutable access to the nodes.
pub type ReverseBindingMut<'a, J, T> = (&'a Reference<T>, &'a mut Vec<Indexed<Node<J, T>>>);

impl<J: JsonHash, T: Id> IntoIterator for ReverseProperties<J, T> {
	type Item = ReverseBinding<J, T>;
	type IntoIter = IntoIter<J, T>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			inner: self.0.into_iter()
		}
	}
}

impl<'a, J: JsonHash, T: Id> IntoIterator for &'a ReverseProperties<J, T> {
	type Item = ReverseBindingRef<'a, J, T>;
	type IntoIter = Iter<'a, J, T>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, J: JsonHash, T: Id> IntoIterator for &'a mut ReverseProperties<J, T> {
	type Item = ReverseBindingMut<'a, J, T>;
	type IntoIter = IterMut<'a, J, T>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

/// Iterator over the reverse properties of a node.
///
/// It is created by the [`ReverseProperties::into_iter`] function.
pub struct IntoIter<J: JsonHash, T: Id> {
	inner: std::collections::hash_map::IntoIter<Reference<T>, Vec<Indexed<Node<J, T>>>>,
}

impl<J: JsonHash, T: Id> Iterator for IntoIter<J, T> {
	type Item = ReverseBinding<J, T>;

	#[inline(always)]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.inner.size_hint()
	}

	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next()
	}
}

impl<J: JsonHash, T: Id> ExactSizeIterator for IntoIter<J, T> {}

impl<J: JsonHash, T: Id> std::iter::FusedIterator for IntoIter<J, T> {}

/// Iterator over the reverse properties of a node.
///
/// It is created by the [`ReverseProperties::iter`] function.
pub struct Iter<'a, J: JsonHash, T: Id> {
	inner: std::collections::hash_map::Iter<'a, Reference<T>, Vec<Indexed<Node<J, T>>>>,
}

impl<'a, J: JsonHash, T: Id> Iterator for Iter<'a, J, T> {
	type Item = ReverseBindingRef<'a, J, T>;

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

impl<'a, J: JsonHash, T: Id> ExactSizeIterator for Iter<'a, J, T> {}

impl<'a, J: JsonHash, T: Id> std::iter::FusedIterator for Iter<'a, J, T> {}

/// Iterator over the reverse properties of a node, giving a mutable reference
/// to the associated nodes.
///
/// It is created by the [`ReverseProperties::iter_mut`] function.
pub struct IterMut<'a, J: JsonHash, T: Id> {
	inner: std::collections::hash_map::IterMut<'a, Reference<T>, Vec<Indexed<Node<J, T>>>>,
}

impl<'a, J: JsonHash, T: Id> Iterator for IterMut<'a, J, T> {
	type Item = ReverseBindingMut<'a, J, T>;

	#[inline(always)]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.inner.size_hint()
	}

	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next()
	}
}

impl<'a, J: JsonHash, T: Id> ExactSizeIterator for IterMut<'a, J, T> {}

impl<'a, J: JsonHash, T: Id> std::iter::FusedIterator for IterMut<'a, J, T> {}
