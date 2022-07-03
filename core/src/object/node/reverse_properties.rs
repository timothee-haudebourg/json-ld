use super::{Node, Nodes};
use crate::{Id, Indexed, Reference, ToReference};
use std::{
	borrow::Borrow,
	collections::HashMap,
	hash::{Hash, Hasher},
};

/// Reverse properties of a node object, and their associated nodes.
#[derive(PartialEq, Eq)]
pub struct ReverseProperties<T: Id>(HashMap<Reference<T>, Vec<Indexed<Node<T>>>>);

impl<T: Id> ReverseProperties<T> {
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
	pub fn get<'a, Q: ToReference<T>>(&self, prop: Q) -> Nodes<T>
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
	pub fn get_any<'a, Q: ToReference<T>>(&self, prop: Q) -> Option<&Indexed<Node<T>>>
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
	pub fn insert(&mut self, prop: Reference<T>, value: Indexed<Node<T>>) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			node_values.push(value);
		} else {
			let node_values = vec![value];
			self.0.insert(prop, node_values);
		}
	}

	/// Associate the given node to the given reverse property, unless it is already.
	#[inline(always)]
	pub fn insert_unique(&mut self, prop: Reference<T>, value: Indexed<Node<T>>) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			if node_values.iter().all(|v| !v.equivalent(&value)) {
				node_values.push(value)
			}
		} else {
			let node_values = vec![value];
			self.0.insert(prop, node_values);
		}
	}

	/// Associate all the given nodes to the given reverse property.
	#[inline(always)]
	pub fn insert_all<Objects: IntoIterator<Item = Indexed<Node<T>>>>(
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

	/// Associate all the given nodes to the given reverse property, unless it is already.
	#[inline(always)]
	pub fn insert_all_unique<Objects: IntoIterator<Item = Indexed<Node<T>>>>(
		&mut self,
		prop: Reference<T>,
		values: Objects,
	) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			for value in values {
				if node_values.iter().all(|v| !v.equivalent(&value)) {
					node_values.push(value)
				}
			}
		} else {
			let values = values.into_iter();
			let mut node_values: Vec<Indexed<Node<T>>> =
				Vec::with_capacity(values.size_hint().0);
			for value in values {
				if node_values.iter().all(|v| !v.equivalent(&value)) {
					node_values.push(value)
				}
			}

			self.0.insert(prop, node_values);
		}
	}

	pub fn extend_unique<I>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (Reference<T>, Vec<Indexed<Node<T>>>)>,
	{
		for (prop, values) in iter {
			self.insert_all_unique(prop, values)
		}
	}

	/// Returns an iterator over the reverse properties and their associated nodes.
	#[inline(always)]
	pub fn iter(&self) -> Iter<'_, T> {
		Iter {
			inner: self.0.iter(),
		}
	}

	/// Returns an iterator over the reverse properties with a mutable reference to their associated nodes.
	#[inline(always)]
	pub fn iter_mut(&mut self) -> IterMut<'_, T> {
		IterMut {
			inner: self.0.iter_mut(),
		}
	}

	/// Removes and returns all the values associated to the given reverse property.
	#[inline(always)]
	pub fn remove(&mut self, prop: &Reference<T>) -> Option<Vec<Indexed<Node<T>>>> {
		self.0.remove(prop)
	}

	/// Removes all reverse properties.
	#[inline(always)]
	pub fn clear(&mut self) {
		self.0.clear()
	}

	#[inline(always)]
	pub fn traverse(&self) -> Traverse<T> {
		Traverse {
			current_node: None,
			current_property: None,
			iter: self.0.iter(),
		}
	}
}

impl<T: Id> Hash for ReverseProperties<T> {
	#[inline(always)]
	fn hash<H: Hasher>(&self, h: &mut H) {
		crate::utils::hash_map(&self.0, h)
	}
}

impl<T: Id> Extend<(Reference<T>, Vec<Indexed<Node<T>>>)>
	for ReverseProperties<T>
{
	fn extend<I>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (Reference<T>, Vec<Indexed<Node<T>>>)>,
	{
		for (prop, values) in iter {
			self.insert_all(prop, values)
		}
	}
}

/// Tuple type representing a reverse binding in a node object,
/// associating a reverse property to some nodes.
pub type ReverseBinding<T> = (Reference<T>, Vec<Indexed<Node<T>>>);

/// Tuple type representing a reference to a reverse binding in a node object,
/// associating a reverse property to some nodes.
pub type ReverseBindingRef<'a, T> = (&'a Reference<T>, &'a [Indexed<Node<T>>]);

/// Tuple type representing a mutable reference to a reverse binding in a node object,
/// associating a reverse property to some nodes, with a mutable access to the nodes.
pub type ReverseBindingMut<'a, T> = (&'a Reference<T>, &'a mut Vec<Indexed<Node<T>>>);

impl<T: Id> IntoIterator for ReverseProperties<T> {
	type Item = ReverseBinding<T>;
	type IntoIter = IntoIter<T>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			inner: self.0.into_iter(),
		}
	}
}

impl<'a, T: Id> IntoIterator for &'a ReverseProperties<T> {
	type Item = ReverseBindingRef<'a, T>;
	type IntoIter = Iter<'a, T>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, T: Id> IntoIterator for &'a mut ReverseProperties<T> {
	type Item = ReverseBindingMut<'a, T>;
	type IntoIter = IterMut<'a, T>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

/// Iterator over the reverse properties of a node.
///
/// It is created by the [`ReverseProperties::into_iter`] function.
pub struct IntoIter<T: Id> {
	inner: std::collections::hash_map::IntoIter<Reference<T>, Vec<Indexed<Node<T>>>>,
}

impl<T: Id> Iterator for IntoIter<T> {
	type Item = ReverseBinding<T>;

	#[inline(always)]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.inner.size_hint()
	}

	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next()
	}
}

impl<T: Id> ExactSizeIterator for IntoIter<T> {}

impl<T: Id> std::iter::FusedIterator for IntoIter<T> {}

/// Iterator over the reverse properties of a node.
///
/// It is created by the [`ReverseProperties::iter`] function.
pub struct Iter<'a, T: Id> {
	inner: std::collections::hash_map::Iter<'a, Reference<T>, Vec<Indexed<Node<T>>>>,
}

impl<'a, T: Id> Iterator for Iter<'a, T> {
	type Item = ReverseBindingRef<'a, T>;

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

impl<'a, T: Id> ExactSizeIterator for Iter<'a, T> {}

impl<'a, T: Id> std::iter::FusedIterator for Iter<'a, T> {}

/// Iterator over the reverse properties of a node, giving a mutable reference
/// to the associated nodes.
///
/// It is created by the [`ReverseProperties::iter_mut`] function.
pub struct IterMut<'a, T: Id> {
	inner: std::collections::hash_map::IterMut<'a, Reference<T>, Vec<Indexed<Node<T>>>>,
}

impl<'a, T: Id> Iterator for IterMut<'a, T> {
	type Item = ReverseBindingMut<'a, T>;

	#[inline(always)]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.inner.size_hint()
	}

	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next()
	}
}

impl<'a, T: Id> ExactSizeIterator for IterMut<'a, T> {}

impl<'a, T: Id> std::iter::FusedIterator for IterMut<'a, T> {}

pub struct Traverse<'a, T: Id> {
	current_node: Option<Box<super::Traverse<'a, T>>>,
	current_property: Option<std::slice::Iter<'a, Indexed<Node<T>>>>,
	iter: std::collections::hash_map::Iter<'a, Reference<T>, Vec<Indexed<Node<T>>>>,
}

impl<'a, T: Id> Iterator for Traverse<'a, T> {
	type Item = crate::object::Ref<'a, T>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match &mut self.current_node {
				Some(current_node) => match current_node.next() {
					Some(next) => break Some(next),
					None => self.current_node = None,
				},
				None => match &mut self.current_property {
					Some(current_property) => match current_property.next() {
						Some(object) => self.current_node = Some(Box::new(object.traverse())),
						None => self.current_property = None,
					},
					None => match self.iter.next() {
						Some((_, property)) => self.current_property = Some(property.iter()),
						None => break None,
					},
				},
			}
		}
	}
}
