use super::{Multiset, Objects};
use crate::{Id, IndexedObject};
use educe::Educe;
use indexmap::IndexMap;
use std::hash::{Hash, Hasher};

pub type PropertyObjects = Multiset<IndexedObject>;

/// Properties of a node object, and their associated objects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Properties(IndexMap<Id, PropertyObjects>);

impl Default for Properties {
	fn default() -> Self {
		Self::new()
	}
}

impl Properties {
	/// Creates an empty map.
	pub fn new() -> Self {
		Self(IndexMap::new())
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

	/// Returns an iterator over the properties and their associated objects.
	#[inline(always)]
	pub fn iter(&self) -> Iter<'_> {
		Iter {
			inner: self.0.iter(),
		}
	}

	/// Returns an iterator over the properties with a mutable reference to their associated objects.
	#[inline(always)]
	pub fn iter_mut(&mut self) -> IterMut<'_> {
		self.0.iter_mut()
	}

	/// Removes all properties.
	#[inline(always)]
	pub fn clear(&mut self) {
		self.0.clear()
	}
}

impl Properties {
	/// Checks if the given property is associated to any object.
	#[inline(always)]
	pub fn contains<Q: ?Sized + Hash + indexmap::Equivalent<Id>>(&self, prop: &Q) -> bool {
		self.0.get(prop).is_some()
	}

	/// Counts the number of objects associated to the given property.
	#[inline(always)]
	pub fn count<Q: ?Sized + Hash + indexmap::Equivalent<Id>>(&self, prop: &Q) -> usize {
		self.0.get(prop).map(Multiset::len).unwrap_or_default()
	}

	/// Returns an iterator over all the objects associated to the given property.
	#[inline(always)]
	pub fn get<Q: ?Sized + Hash + indexmap::Equivalent<Id>>(&self, prop: &Q) -> Objects {
		match self.0.get(prop) {
			Some(values) => Objects::new(Some(values.iter())),
			None => Objects::new(None),
		}
	}

	/// Get one of the objects associated to the given property.
	///
	/// If multiple objects are found, there are no guaranties on which object will be returned.
	#[inline(always)]
	pub fn get_any<Q: ?Sized + Hash + indexmap::Equivalent<Id>>(
		&self,
		prop: &Q,
	) -> Option<&IndexedObject> {
		match self.0.get(prop) {
			Some(values) => values.iter().next(),
			None => None,
		}
	}

	/// Associate the given object to the node through the given property with metadata.
	#[inline(always)]
	pub fn insert(&mut self, prop: impl Into<Id>, value: IndexedObject) {
		let prop = prop.into();
		if let Some(node_values) = self.0.get_mut(&prop) {
			node_values.insert(value);
		} else {
			self.0.insert(prop, Multiset::singleton(value));
		}
	}

	/// Associate the given object to the node through the given property, unless it is already.
	#[inline(always)]
	pub fn insert_unique(&mut self, prop: impl Into<Id>, value: IndexedObject) {
		let prop = prop.into();
		if let Some(node_values) = self.0.get_mut(&prop) {
			if node_values.iter().all(|v| !v.equivalent(&value)) {
				node_values.insert(value)
			}
		} else {
			self.0.insert(prop, Multiset::singleton(value));
		}
	}

	/// Associate all the given objects to the node through the given property.
	#[inline(always)]
	pub fn insert_all<Objects: IntoIterator<Item = IndexedObject>>(
		&mut self,
		prop: impl Into<Id>,
		values: Objects,
	) {
		let prop = prop.into();
		if let Some(node_values) = self.0.get_mut(&prop) {
			node_values.extend(values);
		} else {
			self.0.insert(prop, values.into_iter().collect());
		}
	}

	/// Associate all the given objects to the node through the given property, unless it is already.
	///
	/// The [equivalence operator](crate::Object::equivalent) is used to remove equivalent objects.
	#[inline(always)]
	pub fn insert_all_unique<Objects: IntoIterator<Item = IndexedObject>>(
		&mut self,
		prop: Id,
		values: Objects,
	) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			for value in values {
				if node_values.iter().all(|v| !v.equivalent(&value)) {
					node_values.insert(value)
				}
			}
		} else {
			let values = values.into_iter();
			let mut node_values: PropertyObjects = Multiset::with_capacity(values.size_hint().0);
			for value in values {
				if node_values.iter().all(|v| !v.equivalent(&value)) {
					node_values.insert(value)
				}
			}

			self.0.insert(prop, node_values);
		}
	}

	pub fn set(&mut self, prop: impl Into<Id>, values: PropertyObjects) {
		let prop = prop.into();
		self.0.insert(prop, values);
	}

	pub fn extend_unique<I, O>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (Id, O)>,
		O: IntoIterator<Item = IndexedObject>,
	{
		for (prop, values) in iter {
			self.insert_all_unique(prop, values)
		}
	}

	/// Removes and returns all the values associated to the given property.
	#[inline(always)]
	pub fn remove<Q: ?Sized + Hash + indexmap::Equivalent<Id>>(
		&mut self,
		prop: &Q,
	) -> Option<PropertyObjects> {
		self.0.swap_remove(prop)
	}
}

impl<O> FromIterator<(Id, O)> for Properties
where
	O: IntoIterator<Item = IndexedObject>,
{
	fn from_iter<I: IntoIterator<Item = (Id, O)>>(iter: I) -> Self {
		let mut result = Self::default();
		for (id, values) in iter {
			result.insert_all(id, values);
		}
		result
	}
}

impl Hash for Properties {
	#[inline(always)]
	fn hash<H: Hasher>(&self, h: &mut H) {
		crate::utils::hash_map(&self.0, h)
	}
}

impl Extend<(Id, Vec<IndexedObject>)> for Properties {
	fn extend<I>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (Id, Vec<IndexedObject>)>,
	{
		for (prop, values) in iter {
			self.insert_all(prop, values)
		}
	}
}

/// Tuple type representing a binding in a node object,
/// associating a property to some objects.
pub type Binding = (Id, PropertyObjects);

/// Tuple type representing a reference to a binding in a node object,
/// associating a property to some objects.
pub type BindingRef<'a> = (&'a Id, &'a [IndexedObject]);

/// Tuple type representing a mutable reference to a binding in a node object,
/// associating a property to some objects, with a mutable access to the objects.
pub type BindingMut<'a> = (&'a Id, &'a mut PropertyObjects);

impl IntoIterator for Properties {
	type Item = Binding;
	type IntoIter = IntoIter;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<'a> IntoIterator for &'a Properties {
	type Item = BindingRef<'a>;
	type IntoIter = Iter<'a>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a> IntoIterator for &'a mut Properties {
	type Item = BindingMut<'a>;
	type IntoIter = IterMut<'a>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

/// Iterator over the properties of a node.
///
/// It is created by the [`Properties::into_iter`] function.
pub type IntoIter = indexmap::map::IntoIter<Id, PropertyObjects>;

/// Iterator over the properties of a node.
///
/// It is created by the [`Properties::iter`] function.
#[derive(Educe)]
#[educe(Clone)]
pub struct Iter<'a> {
	inner: indexmap::map::Iter<'a, Id, PropertyObjects>,
}

impl<'a> Iterator for Iter<'a> {
	type Item = BindingRef<'a>;

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

/// Iterator over the properties of a node, giving a mutable reference
/// to the associated objects.
///
/// It is created by the [`Properties::iter_mut`] function.
pub type IterMut<'a> = indexmap::map::IterMut<'a, Id, PropertyObjects>;
