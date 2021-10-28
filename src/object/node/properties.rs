use super::Objects;
use crate::{Id, Indexed, Object, Reference, ToReference};
use generic_json::JsonHash;
use std::{
	borrow::Borrow,
	collections::HashMap,
	hash::{Hash, Hasher},
};

/// Properties of a node object, and their associated objects.
#[derive(PartialEq, Eq)]
pub struct Properties<J: JsonHash, T: Id>(HashMap<Reference<T>, Vec<Indexed<Object<J, T>>>>);

impl<J: JsonHash, T: Id> Properties<J, T> {
	/// Creates an empty map.
	pub(crate) fn new() -> Self {
		Self(HashMap::new())
	}

	/// Returns the number of properties.
	#[inline(always)]
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Checks if there are no defined properties.
	#[inline(always)]
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	/// Checks if the given property is associated to any object.
	#[inline(always)]
	pub fn contains<Q: ToReference<T>>(&self, prop: Q) -> bool {
		self.0.get(prop.to_ref().borrow()).is_some()
	}

	/// Returns an iterator over all the objects associated to the given property.
	#[inline(always)]
	pub fn get<'a, Q: ToReference<T>>(&self, prop: Q) -> Objects<J, T>
	where
		T: 'a,
	{
		match self.0.get(prop.to_ref().borrow()) {
			Some(values) => Objects::new(Some(values.iter())),
			None => Objects::new(None),
		}
	}

	/// Get one of the objects associated to the given property.
	///
	/// If multiple objects are found, there are no guaranties on which object will be returned.
	#[inline(always)]
	pub fn get_any<'a, Q: ToReference<T>>(&self, prop: Q) -> Option<&Indexed<Object<J, T>>>
	where
		T: 'a,
	{
		match self.0.get(prop.to_ref().borrow()) {
			Some(values) => values.iter().next(),
			None => None,
		}
	}

	/// Associate the given object to the node through the given property.
	#[inline(always)]
	pub fn insert(&mut self, prop: Reference<T>, value: Indexed<Object<J, T>>) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			node_values.push(value);
		} else {
			let node_values = vec![value];
			self.0.insert(prop, node_values);
		}
	}

	/// Associate all the given objects to the node through the given property.
	#[inline(always)]
	pub fn insert_all<Objects: Iterator<Item = Indexed<Object<J, T>>>>(
		&mut self,
		prop: Reference<T>,
		values: Objects,
	) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			node_values.extend(values);
		} else {
			self.0.insert(prop, values.collect());
		}
	}

	/// Returns an iterator over the properties and their associated objects.
	#[inline(always)]
	pub fn iter(&self) -> Iter<'_, J, T> {
		Iter {
			inner: self.0.iter(),
		}
	}

	/// Returns an iterator over the properties with a mutable reference to their associated objects.
	#[inline(always)]
	pub fn iter_mut(&mut self) -> IterMut<'_, J, T> {
		IterMut {
			inner: self.0.iter_mut(),
		}
	}
}

impl<J: JsonHash, T: Id> Hash for Properties<J, T> {
	#[inline(always)]
	fn hash<H: Hasher>(&self, h: &mut H) {
		crate::util::hash_map(&self.0, h)
	}
}

/// Tuple type representing a reference to a binding in a node object,
/// associating a property to some objects.
pub type BindingRef<'a, J, T> = (&'a Reference<T>, &'a [Indexed<Object<J, T>>]);

/// Tuple type representing a mutable reference to a binding in a node object,
/// associating a property to some objects, with a mutable access to the objects.
pub type BindingMut<'a, J, T> = (&'a Reference<T>, &'a mut Vec<Indexed<Object<J, T>>>);

impl<'a, J: JsonHash, T: Id> IntoIterator for &'a Properties<J, T> {
	type Item = BindingRef<'a, J, T>;
	type IntoIter = Iter<'a, J, T>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, J: JsonHash, T: Id> IntoIterator for &'a mut Properties<J, T> {
	type Item = BindingMut<'a, J, T>;
	type IntoIter = IterMut<'a, J, T>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

/// Iterator over the properties of a node.
///
/// It is created by the [`Properties::iter`] function.
pub struct Iter<'a, J: JsonHash, T: Id> {
	inner: std::collections::hash_map::Iter<'a, Reference<T>, Vec<Indexed<Object<J, T>>>>,
}

impl<'a, J: JsonHash, T: Id> Iterator for Iter<'a, J, T> {
	type Item = BindingRef<'a, J, T>;

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

/// Iterator over the properties of a node, giving a mutable reference
/// to the associated objects.
///
/// It is created by the [`Properties::iter_mut`] function.
pub struct IterMut<'a, J: JsonHash, T: Id> {
	inner: std::collections::hash_map::IterMut<'a, Reference<T>, Vec<Indexed<Object<J, T>>>>,
}

impl<'a, J: JsonHash, T: Id> Iterator for IterMut<'a, J, T> {
	type Item = BindingMut<'a, J, T>;

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
