use super::{Multiset, Objects};
use crate::{
	object::{InvalidExpandedJson, TryFromJson, TryFromJsonObject},
	Id, IndexedObject,
};
use educe::Educe;
use indexmap::IndexMap;
use rdf_types::VocabularyMut;
use std::hash::{Hash, Hasher};

pub type PropertyObjects<T, B> = Multiset<IndexedObject<T, B>>;

/// Properties of a node object, and their associated objects.
#[derive(Educe, Debug, Clone)]
#[educe(
	PartialEq(bound = "T: Eq + Hash, B: Eq + Hash"),
	Eq(bound = "T: Eq + Hash, B: Eq + Hash")
)]
pub struct Properties<T, B>(IndexMap<Id<T, B>, PropertyObjects<T, B>>);

impl<T, B> Default for Properties<T, B> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T, B> Properties<T, B> {
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
	pub fn iter(&self) -> Iter<'_, T, B> {
		Iter {
			inner: self.0.iter(),
		}
	}

	/// Returns an iterator over the properties with a mutable reference to their associated objects.
	#[inline(always)]
	pub fn iter_mut(&mut self) -> IterMut<'_, T, B> {
		self.0.iter_mut()
	}

	/// Removes all properties.
	#[inline(always)]
	pub fn clear(&mut self) {
		self.0.clear()
	}
}

impl<T: Eq + Hash, B: Eq + Hash> Properties<T, B> {
	/// Checks if the given property is associated to any object.
	#[inline(always)]
	pub fn contains<Q: ?Sized + Hash + indexmap::Equivalent<Id<T, B>>>(&self, prop: &Q) -> bool {
		self.0.get(prop).is_some()
	}

	/// Returns an iterator over all the objects associated to the given property.
	#[inline(always)]
	pub fn get<Q: ?Sized + Hash + indexmap::Equivalent<Id<T, B>>>(
		&self,
		prop: &Q,
	) -> Objects<T, B> {
		match self.0.get(prop) {
			Some(values) => Objects::new(Some(values.iter())),
			None => Objects::new(None),
		}
	}

	/// Get one of the objects associated to the given property.
	///
	/// If multiple objects are found, there are no guaranties on which object will be returned.
	#[inline(always)]
	pub fn get_any<Q: ?Sized + Hash + indexmap::Equivalent<Id<T, B>>>(
		&self,
		prop: &Q,
	) -> Option<&IndexedObject<T, B>> {
		match self.0.get(prop) {
			Some(values) => values.iter().next(),
			None => None,
		}
	}

	/// Associate the given object to the node through the given property with metadata.
	#[inline(always)]
	pub fn insert(&mut self, prop: Id<T, B>, value: IndexedObject<T, B>) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			node_values.insert(value);
		} else {
			self.0.insert(prop, Multiset::singleton(value));
		}
	}

	/// Associate the given object to the node through the given property, unless it is already.
	#[inline(always)]
	pub fn insert_unique(&mut self, prop: Id<T, B>, value: IndexedObject<T, B>) {
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
	pub fn insert_all<Objects: IntoIterator<Item = IndexedObject<T, B>>>(
		&mut self,
		prop: Id<T, B>,
		values: Objects,
	) {
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
	pub fn insert_all_unique<Objects: IntoIterator<Item = IndexedObject<T, B>>>(
		&mut self,
		prop: Id<T, B>,
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
			let mut node_values: PropertyObjects<T, B> =
				Multiset::with_capacity(values.size_hint().0);
			for value in values {
				if node_values.iter().all(|v| !v.equivalent(&value)) {
					node_values.insert(value)
				}
			}

			self.0.insert(prop, node_values);
		}
	}

	pub fn set(&mut self, prop: Id<T, B>, values: PropertyObjects<T, B>) {
		self.0.insert(prop, values);
	}

	pub fn extend_unique<I, O>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (Id<T, B>, O)>,
		O: IntoIterator<Item = IndexedObject<T, B>>,
	{
		for (prop, values) in iter {
			self.insert_all_unique(prop, values)
		}
	}

	/// Removes and returns all the values associated to the given property.
	#[inline(always)]
	pub fn remove<Q: ?Sized + Hash + indexmap::Equivalent<Id<T, B>>>(
		&mut self,
		prop: &Q,
	) -> Option<PropertyObjects<T, B>> {
		self.0.swap_remove(prop)
	}
}

impl<T: Eq + Hash, B: Eq + Hash, O> FromIterator<(Id<T, B>, O)> for Properties<T, B>
where
	O: IntoIterator<Item = IndexedObject<T, B>>,
{
	fn from_iter<I: IntoIterator<Item = (Id<T, B>, O)>>(iter: I) -> Self {
		let mut result = Self::default();
		for (id, values) in iter {
			result.insert_all(id, values);
		}
		result
	}
}

impl<T: Eq + Hash, B: Eq + Hash> TryFromJson<T, B> for Properties<T, B> {
	fn try_from_json_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		value: json_syntax::Value,
	) -> Result<Self, InvalidExpandedJson> {
		match value {
			json_syntax::Value::Object(object) => Self::try_from_json_object_in(vocabulary, object),
			_ => Err(InvalidExpandedJson::InvalidObject),
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash> TryFromJsonObject<T, B> for Properties<T, B> {
	fn try_from_json_object_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		object: json_syntax::Object,
	) -> Result<Self, InvalidExpandedJson> {
		let mut result = Self::new();

		for entry in object {
			let prop = Id::from_string_in(vocabulary, entry.key.to_string());
			let objects: Vec<IndexedObject<T, B>> = Vec::try_from_json_in(vocabulary, entry.value)?;
			result.insert_all(prop, objects)
		}

		Ok(result)
	}
}

impl<T: Hash, B: Hash> Hash for Properties<T, B> {
	#[inline(always)]
	fn hash<H: Hasher>(&self, h: &mut H) {
		crate::utils::hash_map(&self.0, h)
	}
}

impl<T: Eq + Hash, B: Eq + Hash> Extend<(Id<T, B>, Vec<IndexedObject<T, B>>)> for Properties<T, B> {
	fn extend<I>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (Id<T, B>, Vec<IndexedObject<T, B>>)>,
	{
		for (prop, values) in iter {
			self.insert_all(prop, values)
		}
	}
}

/// Tuple type representing a binding in a node object,
/// associating a property to some objects.
pub type Binding<T, B> = (Id<T, B>, PropertyObjects<T, B>);

/// Tuple type representing a reference to a binding in a node object,
/// associating a property to some objects.
pub type BindingRef<'a, T, B> = (&'a Id<T, B>, &'a [IndexedObject<T, B>]);

/// Tuple type representing a mutable reference to a binding in a node object,
/// associating a property to some objects, with a mutable access to the objects.
pub type BindingMut<'a, T, B> = (&'a Id<T, B>, &'a mut PropertyObjects<T, B>);

impl<T, B> IntoIterator for Properties<T, B> {
	type Item = Binding<T, B>;
	type IntoIter = IntoIter<T, B>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<'a, T, B> IntoIterator for &'a Properties<T, B> {
	type Item = BindingRef<'a, T, B>;
	type IntoIter = Iter<'a, T, B>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, T, B> IntoIterator for &'a mut Properties<T, B> {
	type Item = BindingMut<'a, T, B>;
	type IntoIter = IterMut<'a, T, B>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

/// Iterator over the properties of a node.
///
/// It is created by the [`Properties::into_iter`] function.
pub type IntoIter<T, B> = indexmap::map::IntoIter<Id<T, B>, PropertyObjects<T, B>>;

/// Iterator over the properties of a node.
///
/// It is created by the [`Properties::iter`] function.
#[derive(Educe)]
#[educe(Clone)]
pub struct Iter<'a, T, B> {
	inner: indexmap::map::Iter<'a, Id<T, B>, PropertyObjects<T, B>>,
}

impl<'a, T, B> Iterator for Iter<'a, T, B> {
	type Item = BindingRef<'a, T, B>;

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

impl<'a, T, B> ExactSizeIterator for Iter<'a, T, B> {}

impl<'a, T, B> std::iter::FusedIterator for Iter<'a, T, B> {}

/// Iterator over the properties of a node, giving a mutable reference
/// to the associated objects.
///
/// It is created by the [`Properties::iter_mut`] function.
pub type IterMut<'a, T, B> = indexmap::map::IterMut<'a, Id<T, B>, PropertyObjects<T, B>>;
