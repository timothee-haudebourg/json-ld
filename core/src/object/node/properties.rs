use super::{Multiset, Objects};
use crate::{
	object::{InvalidExpandedJson, TryFromJson, TryFromJsonObject},
	Id, Indexed, IndexedObject, Object, StrippedIndexedObject,
};
use educe::Educe;
use indexmap::IndexMap;
use locspan::{BorrowStripped, Meta, Stripped};
use locspan_derive::{StrippedEq, StrippedHash, StrippedPartialEq};
use rdf_types::VocabularyMut;
use std::{
	hash::{Hash, Hasher},
	ops,
};

/// Property set entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, StrippedPartialEq, StrippedEq, StrippedHash)]
#[locspan(ignore(M))]
pub struct Entry<T, M> {
	/// Property key metadata.
	#[locspan(ignore)]
	pub key_metadata: M,

	/// Property value.
	pub value: T,
}

impl<T, M> Entry<T, M> {
	pub fn new(key_metadata: M, value: T) -> Self {
		Self {
			key_metadata,
			value,
		}
	}

	pub fn borrow(&self) -> Entry<&T, &M> {
		Entry {
			key_metadata: &self.key_metadata,
			value: &self.value,
		}
	}

	pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Entry<U, M> {
		Entry {
			key_metadata: self.key_metadata,
			value: f(self.value),
		}
	}
}

impl<T, M> ops::Deref for Entry<T, M> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<T, M> ops::DerefMut for Entry<T, M> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.value
	}
}

pub type PropertyObjects<T, B, M = ()> = Multiset<StrippedIndexedObject<T, B, M>>;

/// Properties of a node object, and their associated objects.
#[derive(Educe, Debug, Clone)]
#[educe(
	PartialEq(bound = "T: Eq + Hash, B: Eq + Hash, M: PartialEq"),
	Eq(bound = "T: Eq + Hash, B: Eq + Hash, M: Eq")
)]
pub struct Properties<T, B, M = ()>(IndexMap<Id<T, B>, PropertyEntry<T, B, M>>);

impl<T, B, M> Default for Properties<T, B, M> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T, B, M> Properties<T, B, M> {
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
	pub fn iter(&self) -> Iter<'_, T, B, M> {
		Iter {
			inner: self.0.iter(),
		}
	}

	/// Returns an iterator over the properties with a mutable reference to their associated objects.
	#[inline(always)]
	pub fn iter_mut(&mut self) -> IterMut<'_, T, B, M> {
		IterMut {
			inner: self.0.iter_mut(),
		}
	}

	/// Removes all properties.
	#[inline(always)]
	pub fn clear(&mut self) {
		self.0.clear()
	}
}

impl<T: Eq + Hash, B: Eq + Hash> Properties<T, B> {
	/// Associate the given object to the node through the given property.
	#[inline(always)]
	pub fn insert(&mut self, prop: Id<T, B>, value: Indexed<Object<T, B>>) {
		self.insert_with(Meta::none(prop), Meta::none(value))
	}

	/// Associate the given object to the node through the given property, unless it is already.
	#[inline(always)]
	pub fn insert_unique(&mut self, prop: Id<T, B>, value: Indexed<Object<T, B>>) {
		self.insert_unique_with(Meta::none(prop), Meta::none(value))
	}

	/// Associate all the given objects to the node through the given property.
	#[inline(always)]
	pub fn insert_all<Objects: IntoIterator<Item = Indexed<Object<T, B>>>>(
		&mut self,
		prop: Id<T, B>,
		values: Objects,
	) {
		self.insert_all_with(Meta::none(prop), values.into_iter().map(Meta::none))
	}

	/// Associate all the given objects to the node through the given property, unless it is already.
	///
	/// The [equivalence operator](crate::Object::equivalent) is used to remove equivalent objects.
	#[inline(always)]
	pub fn insert_all_unique<Objects: IntoIterator<Item = Indexed<Object<T, B>>>>(
		&mut self,
		prop: Id<T, B>,
		values: Objects,
	) {
		self.insert_all_unique_stripped_with(
			Meta::none(prop),
			values.into_iter().map(|v| Stripped(Meta::none(v))),
		)
	}

	pub fn set(&mut self, prop: Id<T, B>, values: PropertyObjects<T, B>) {
		self.set_with(Meta::none(prop), values)
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> Properties<T, B, M> {
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
	) -> Objects<T, B, M> {
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
	) -> Option<&IndexedObject<T, B, M>> {
		match self.0.get(prop) {
			Some(values) => values.iter().next().map(|o| &o.0),
			None => None,
		}
	}

	/// Associate the given object to the node through the given property with metadata.
	#[inline(always)]
	pub fn insert_with(
		&mut self,
		Meta(prop, meta): Meta<Id<T, B>, M>,
		value: IndexedObject<T, B, M>,
	) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			node_values.insert(Stripped(value));
		} else {
			self.0
				.insert(prop, Entry::new(meta, Multiset::singleton(Stripped(value))));
		}
	}

	/// Associate the given object to the node through the given property, unless it is already.
	#[inline(always)]
	pub fn insert_unique_with(
		&mut self,
		Meta(prop, meta): Meta<Id<T, B>, M>,
		value: IndexedObject<T, B, M>,
	) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			if node_values.iter().all(|v| !v.equivalent(&value)) {
				node_values.insert(Stripped(value))
			}
		} else {
			self.0
				.insert(prop, Entry::new(meta, Multiset::singleton(Stripped(value))));
		}
	}

	/// Associate all the given objects to the node through the given property.
	#[inline(always)]
	pub fn insert_all_with<Objects: IntoIterator<Item = IndexedObject<T, B, M>>>(
		&mut self,
		Meta(prop, meta): Meta<Id<T, B>, M>,
		values: Objects,
	) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			node_values.extend(values.into_iter().map(Stripped));
		} else {
			self.0.insert(
				prop,
				Entry::new(meta, values.into_iter().map(Stripped).collect()),
			);
		}
	}

	/// Associate all the given objects to the node through the given property, unless it is already.
	///
	/// The [equivalence operator](crate::Object::equivalent) is used to remove equivalent objects.
	#[inline(always)]
	pub fn insert_all_unique_stripped_with<
		Objects: IntoIterator<Item = StrippedIndexedObject<T, B, M>>,
	>(
		&mut self,
		Meta(prop, meta): Meta<Id<T, B>, M>,
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
			let mut node_values: PropertyObjects<T, B, M> =
				Multiset::with_capacity(values.size_hint().0);
			for value in values {
				if node_values.iter().all(|v| !v.equivalent(&value)) {
					node_values.insert(value)
				}
			}

			self.0.insert(prop, Entry::new(meta, node_values));
		}
	}

	/// Associate all the given objects to the node through the given property, unless it is already.
	///
	/// The [equivalence operator](crate::Object::equivalent) is used to remove equivalent objects.
	#[inline(always)]
	pub fn insert_all_unique_with<Objects: IntoIterator<Item = IndexedObject<T, B, M>>>(
		&mut self,
		prop: Meta<Id<T, B>, M>,
		values: Objects,
	) {
		self.insert_all_unique_stripped_with(prop, values.into_iter().map(Stripped))
	}

	pub fn set_with(
		&mut self,
		Meta(prop, meta): Meta<Id<T, B>, M>,
		values: PropertyObjects<T, B, M>,
	) {
		self.0
			.entry(prop)
			.or_insert_with(|| Entry::new(meta, Multiset::new()))
			.value = values
	}

	pub fn extend_unique<I, O>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (Meta<Id<T, B>, M>, O)>,
		O: IntoIterator<Item = IndexedObject<T, B, M>>,
	{
		for (prop, values) in iter {
			self.insert_all_unique_with(prop, values)
		}
	}

	pub fn extend_unique_stripped<I, O>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (Meta<Id<T, B>, M>, O)>,
		O: IntoIterator<Item = StrippedIndexedObject<T, B, M>>,
	{
		for (prop, values) in iter {
			self.insert_all_unique_stripped_with(prop, values)
		}
	}

	/// Removes and returns all the values associated to the given property.
	#[inline(always)]
	pub fn remove<Q: ?Sized + Hash + indexmap::Equivalent<Id<T, B>>>(
		&mut self,
		prop: &Q,
	) -> Option<PropertyEntry<T, B, M>> {
		self.0.remove(prop)
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> TryFromJson<T, B, M> for Properties<T, B, M> {
	fn try_from_json_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidExpandedJson<M>, M>> {
		match value {
			json_syntax::Value::Object(object) => {
				Self::try_from_json_object_in(vocabulary, Meta(object, meta))
			}
			_ => Err(Meta(InvalidExpandedJson::InvalidObject, meta)),
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> TryFromJsonObject<T, B, M> for Properties<T, B, M> {
	fn try_from_json_object_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		Meta(object, meta): Meta<json_syntax::Object<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidExpandedJson<M>, M>> {
		let mut result = Self::new();

		for entry in object {
			let Meta(key, key_meta) = entry.key;
			let prop = Id::from_string_in(vocabulary, key.to_string());
			let objects: Vec<IndexedObject<T, B, M>> =
				Vec::try_from_json_in(vocabulary, entry.value)?.into_value();
			result.insert_all_with(Meta(prop, key_meta), objects)
		}

		Ok(Meta(result, meta))
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> locspan::StrippedPartialEq for Properties<T, B, M> {
	#[inline(always)]
	fn stripped_eq(&self, other: &Self) -> bool {
		self.0.stripped() == other.0.stripped()
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> locspan::StrippedEq for Properties<T, B, M> {}

impl<T: Hash, B: Hash, M> locspan::StrippedHash for Properties<T, B, M> {
	#[inline(always)]
	fn stripped_hash<H: Hasher>(&self, h: &mut H) {
		crate::utils::hash_map_stripped(&self.0, h)
	}
}

impl<T: Hash, B: Hash, M: Hash> Hash for Properties<T, B, M> {
	#[inline(always)]
	fn hash<H: Hasher>(&self, h: &mut H) {
		crate::utils::hash_map(&self.0, h)
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> Extend<(Meta<Id<T, B>, M>, Vec<IndexedObject<T, B, M>>)>
	for Properties<T, B, M>
{
	fn extend<I>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (Meta<Id<T, B>, M>, Vec<IndexedObject<T, B, M>>)>,
	{
		for (prop, values) in iter {
			self.insert_all_with(prop, values)
		}
	}
}

/// Tuple type representing a binding in a node object,
/// associating a property to some objects.
pub type Binding<T, B, M> = (Meta<Id<T, B>, M>, PropertyObjects<T, B, M>);

/// Tuple type representing a reference to a binding in a node object,
/// associating a property to some objects.
pub type BindingRef<'a, T, B, M> = (
	Meta<&'a Id<T, B>, &'a M>,
	&'a [StrippedIndexedObject<T, B, M>],
);

/// Tuple type representing a mutable reference to a binding in a node object,
/// associating a property to some objects, with a mutable access to the objects.
pub type BindingMut<'a, T, B, M> = (
	Meta<&'a Id<T, B>, &'a mut M>,
	&'a mut PropertyObjects<T, B, M>,
);

impl<T, B, M> IntoIterator for Properties<T, B, M> {
	type Item = Binding<T, B, M>;
	type IntoIter = IntoIter<T, B, M>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			inner: self.0.into_iter(),
		}
	}
}

impl<'a, T, B, M> IntoIterator for &'a Properties<T, B, M> {
	type Item = BindingRef<'a, T, B, M>;
	type IntoIter = Iter<'a, T, B, M>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, T, B, M> IntoIterator for &'a mut Properties<T, B, M> {
	type Item = BindingMut<'a, T, B, M>;
	type IntoIter = IterMut<'a, T, B, M>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

/// Iterator over the properties of a node.
///
/// It is created by the [`Properties::into_iter`] function.
pub struct IntoIter<T, B, M> {
	inner: indexmap::map::IntoIter<Id<T, B>, PropertyEntry<T, B, M>>,
}

impl<T, B, M> Iterator for IntoIter<T, B, M> {
	type Item = Binding<T, B, M>;

	#[inline(always)]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.inner.size_hint()
	}

	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item> {
		self.inner
			.next()
			.map(|(k, v)| (Meta(k, v.key_metadata), v.value))
	}
}

impl<T, B, M> ExactSizeIterator for IntoIter<T, B, M> {}

impl<T, B, M> std::iter::FusedIterator for IntoIter<T, B, M> {}

/// Iterator over the properties of a node.
///
/// It is created by the [`Properties::iter`] function.
#[derive(Educe)]
#[educe(Clone)]
pub struct Iter<'a, T, B, M> {
	inner: indexmap::map::Iter<'a, Id<T, B>, PropertyEntry<T, B, M>>,
}

impl<'a, T, B, M> Iterator for Iter<'a, T, B, M> {
	type Item = BindingRef<'a, T, B, M>;

	#[inline(always)]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.inner.size_hint()
	}

	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|(property, objects)| {
			(
				Meta(property, &objects.key_metadata),
				objects.value.as_slice(),
			)
		})
	}
}

impl<'a, T, B, M> ExactSizeIterator for Iter<'a, T, B, M> {}

impl<'a, T, B, M> std::iter::FusedIterator for Iter<'a, T, B, M> {}

pub type PropertyEntry<T, B, M> = Entry<PropertyObjects<T, B, M>, M>;

/// Iterator over the properties of a node, giving a mutable reference
/// to the associated objects.
///
/// It is created by the [`Properties::iter_mut`] function.
pub struct IterMut<'a, T, B, M> {
	inner: indexmap::map::IterMut<'a, Id<T, B>, PropertyEntry<T, B, M>>,
}

impl<'a, T, B, M> Iterator for IterMut<'a, T, B, M> {
	type Item = BindingMut<'a, T, B, M>;

	#[inline(always)]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.inner.size_hint()
	}

	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item> {
		self.inner
			.next()
			.map(|(k, v)| (Meta(k, &mut v.key_metadata), &mut v.value))
	}
}

impl<'a, T, B, M> ExactSizeIterator for IterMut<'a, T, B, M> {}

impl<'a, T, B, M> std::iter::FusedIterator for IterMut<'a, T, B, M> {}
