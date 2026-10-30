use super::JsonLdObjects;
use crate::{
	JsonLdIndexedObject, JsonLdLenient, JsonLdVisit,
	object::{JsonLdObjectMut, JsonLdObjectRef},
};
use btree_indexmap::{BTreeIndexMap, BTreeIndexMultiSet, Comparable};
use educe::Educe;
use rdf_syntax::Id;
use std::hash::Hash;

pub type PropertyObjects = BTreeIndexMultiSet<JsonLdIndexedObject>;

/// Properties of a node object, and their associated objects.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct JsonLdProperties(BTreeIndexMap<JsonLdLenient<Id>, PropertyObjects>);

impl Default for JsonLdProperties {
	fn default() -> Self {
		Self::new()
	}
}

impl JsonLdProperties {
	/// Creates an empty map.
	pub fn new() -> Self {
		Self(BTreeIndexMap::new())
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
	pub fn iter(&self) -> JsonLdPropertiesIter<'_> {
		JsonLdPropertiesIter {
			inner: self.0.iter(),
		}
	}

	/// Returns an iterator over the properties with a mutable reference to their associated objects.
	#[inline(always)]
	pub fn iter_mut(&mut self) -> JsonLdPropertiesIterMut<'_> {
		self.0.iter_mut()
	}

	/// Removes all properties.
	#[inline(always)]
	pub fn clear(&mut self) {
		self.0.clear()
	}

	/// Checks if the given property is associated to any object.
	#[inline(always)]
	pub fn contains<Q: ?Sized + Hash + Comparable<JsonLdLenient<Id>>>(&self, prop: &Q) -> bool {
		self.0.get(prop).is_some()
	}

	/// Counts the number of objects associated to the given property.
	#[inline(always)]
	pub fn count<Q: ?Sized + Hash + Comparable<JsonLdLenient<Id>>>(&self, prop: &Q) -> usize {
		self.0
			.get(prop)
			.map(BTreeIndexMultiSet::len)
			.unwrap_or_default()
	}

	/// Returns an iterator over all the objects associated to the given property.
	#[inline(always)]
	pub fn get<Q: ?Sized + Hash + Comparable<JsonLdLenient<Id>>>(
		&self,
		prop: &Q,
	) -> JsonLdObjects<'_> {
		match self.0.get(prop) {
			Some(values) => JsonLdObjects::new(Some(values.iter())),
			None => JsonLdObjects::new(None),
		}
	}

	/// Get one of the objects associated to the given property.
	///
	/// If multiple objects are found, there are no guaranties on which object will be returned.
	#[inline(always)]
	pub fn get_any<Q: ?Sized + Hash + Comparable<JsonLdLenient<Id>>>(
		&self,
		prop: &Q,
	) -> Option<&JsonLdIndexedObject> {
		match self.0.get(prop) {
			Some(values) => values.iter().next(),
			None => None,
		}
	}

	/// Associate the given object to the node through the given property with metadata.
	#[inline(always)]
	pub fn insert(&mut self, prop: impl Into<JsonLdLenient<Id>>, value: JsonLdIndexedObject) {
		let prop = prop.into();
		if let Some(node_values) = self.0.get_mut(&prop) {
			node_values.insert(value);
		} else {
			self.0.insert(prop, BTreeIndexMultiSet::singleton(value));
		}
	}

	/// Associate the given object to the node through the given property, unless it is already.
	#[inline(always)]
	pub fn insert_unique(
		&mut self,
		prop: impl Into<JsonLdLenient<Id>>,
		value: JsonLdIndexedObject,
	) {
		let prop = prop.into();
		if let Some(node_values) = self.0.get_mut(&prop) {
			if node_values.iter().all(|v| !v.equivalent(&value)) {
				node_values.insert(value);
			}
		} else {
			self.0.insert(prop, BTreeIndexMultiSet::singleton(value));
		}
	}

	/// Associate all the given objects to the node through the given property.
	#[inline(always)]
	pub fn insert_all<Objects: IntoIterator<Item = JsonLdIndexedObject>>(
		&mut self,
		prop: impl Into<JsonLdLenient<Id>>,
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
	/// The [equivalence operator](crate::JsonLdObject::equivalent) is used to remove equivalent objects.
	#[inline(always)]
	pub fn insert_all_unique<Objects: IntoIterator<Item = JsonLdIndexedObject>>(
		&mut self,
		prop: JsonLdLenient<Id>,
		values: Objects,
	) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			for value in values {
				if node_values.iter().all(|v| !v.equivalent(&value)) {
					node_values.insert(value);
				}
			}
		} else {
			let values = values.into_iter();
			let mut node_values: PropertyObjects =
				BTreeIndexMultiSet::with_capacity(values.size_hint().0);
			for value in values {
				if node_values.iter().all(|v| !v.equivalent(&value)) {
					node_values.insert(value);
				}
			}

			self.0.insert(prop, node_values);
		}
	}

	pub fn set(&mut self, prop: impl Into<JsonLdLenient<Id>>, values: PropertyObjects) {
		let prop = prop.into();
		self.0.insert(prop, values);
	}

	pub fn extend_unique<I, O>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (JsonLdLenient<Id>, O)>,
		O: IntoIterator<Item = JsonLdIndexedObject>,
	{
		for (prop, values) in iter {
			self.insert_all_unique(prop, values)
		}
	}

	/// Removes and returns all the values associated to the given property.
	#[inline(always)]
	pub fn remove<Q: ?Sized + Hash + Comparable<JsonLdLenient<Id>>>(
		&mut self,
		prop: &Q,
	) -> Option<PropertyObjects> {
		self.0.swap_remove(prop)
	}
}

impl JsonLdVisit for JsonLdProperties {
	fn visit_with(&self, f: &mut impl FnMut(JsonLdObjectRef)) {
		for (_, t) in self {
			t.visit_with(&mut *f);
		}
	}

	fn visit_mut_with(&mut self, f: &mut impl FnMut(JsonLdObjectMut)) {
		for (_, values) in self {
			values.visit_mut_with(f);
		}
	}
}

impl<O> FromIterator<(JsonLdLenient<Id>, O)> for JsonLdProperties
where
	O: IntoIterator<Item = JsonLdIndexedObject>,
{
	fn from_iter<I: IntoIterator<Item = (JsonLdLenient<Id>, O)>>(iter: I) -> Self {
		let mut result = Self::default();
		for (id, values) in iter {
			result.insert_all(id, values);
		}
		result
	}
}

impl Extend<(JsonLdLenient<Id>, Vec<JsonLdIndexedObject>)> for JsonLdProperties {
	fn extend<I>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (JsonLdLenient<Id>, Vec<JsonLdIndexedObject>)>,
	{
		for (prop, values) in iter {
			self.insert_all(prop, values)
		}
	}
}

/// Tuple type representing a binding in a node object,
/// associating a property to some objects.
pub type JsonLdBinding = (JsonLdLenient<Id>, PropertyObjects);

/// Tuple type representing a reference to a binding in a node object,
/// associating a property to some objects.
pub type JsonLdBindingRef<'a> = (&'a JsonLdLenient<Id>, &'a [JsonLdIndexedObject]);

/// Tuple type representing a mutable reference to a binding in a node object,
/// associating a property to some objects, with a mutable access to the objects.
pub type JsonLdBindingMut<'a> = (&'a JsonLdLenient<Id>, &'a mut PropertyObjects);

impl IntoIterator for JsonLdProperties {
	type Item = JsonLdBinding;
	type IntoIter = JsonLdPropertiesIntoIter;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<'a> IntoIterator for &'a JsonLdProperties {
	type Item = JsonLdBindingRef<'a>;
	type IntoIter = JsonLdPropertiesIter<'a>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a> IntoIterator for &'a mut JsonLdProperties {
	type Item = JsonLdBindingMut<'a>;
	type IntoIter = JsonLdPropertiesIterMut<'a>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

/// Iterator over the properties of a node.
///
/// It is created by the [`Properties::into_iter`] function.
pub type JsonLdPropertiesIntoIter = std::vec::IntoIter<(JsonLdLenient<Id>, PropertyObjects)>;

/// Iterator over the properties of a node.
///
/// It is created by the [`Properties::iter`] function.
#[derive(Educe)]
#[educe(Clone)]
pub struct JsonLdPropertiesIter<'a> {
	inner: btree_indexmap::map::Iter<'a, JsonLdLenient<Id>, PropertyObjects>,
}

impl<'a> Iterator for JsonLdPropertiesIter<'a> {
	type Item = JsonLdBindingRef<'a>;

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

impl<'a> ExactSizeIterator for JsonLdPropertiesIter<'a> {}

impl<'a> std::iter::FusedIterator for JsonLdPropertiesIter<'a> {}

/// Iterator over the properties of a node, giving a mutable reference
/// to the associated objects.
///
/// It is created by the [`Properties::iter_mut`] function.
pub type JsonLdPropertiesIterMut<'a> =
	btree_indexmap::map::IterMut<'a, JsonLdLenient<Id>, PropertyObjects>;
