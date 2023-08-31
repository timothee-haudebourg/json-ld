use super::{Multiset, Nodes};
use crate::{
	object::{InvalidExpandedJson, TryFromJson, TryFromJsonObject},
	Id, Indexed, IndexedNode, Node, StrippedIndexedNode,
};
use contextual::WithContext;
use derivative::Derivative;
use indexmap::IndexMap;
use iref::IriBuf;
use json_ld_syntax::IntoJsonWithContextMeta;
use locspan::{BorrowStripped, Meta, Stripped};
use rdf_types::{BlankIdBuf, Vocabulary, VocabularyMut};
use std::hash::{Hash, Hasher};

pub use super::properties::Entry;

pub type ReversePropertyNodes<T = IriBuf, B = BlankIdBuf, M = ()> =
	Multiset<Stripped<IndexedNode<T, B, M>>>;

/// Reverse properties of a node object, and their associated nodes.
#[derive(Derivative, Clone)]
#[derivative(
	PartialEq(bound = "T: Eq + Hash, B: Eq + Hash, M: PartialEq"),
	Eq(bound = "T: Eq + Hash, B: Eq + Hash, M: Eq")
)]
pub struct ReverseProperties<T = IriBuf, B = BlankIdBuf, M = ()>(
	IndexMap<Id<T, B>, ReversePropertyEntry<T, B, M>>,
);

impl<T, B, M> Default for ReverseProperties<T, B, M> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T, B, M> ReverseProperties<T, B, M> {
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
	pub fn iter(&self) -> Iter<'_, T, B, M> {
		Iter {
			inner: self.0.iter(),
		}
	}

	/// Returns an iterator over the reverse properties with a mutable reference to their associated nodes.
	#[inline(always)]
	pub fn iter_mut(&mut self) -> IterMut<'_, T, B, M> {
		IterMut {
			inner: self.0.iter_mut(),
		}
	}

	/// Removes all reverse properties.
	#[inline(always)]
	pub fn clear(&mut self) {
		self.0.clear()
	}
}

impl<T: Eq + Hash, B: Eq + Hash> ReverseProperties<T, B> {
	/// Associate the given node the reverse property `prop`.
	#[inline(always)]
	pub fn insert(&mut self, prop: Id<T, B>, value: Indexed<Node<T, B>>) {
		self.insert_with(Meta::none(prop), Meta::none(value))
	}

	/// Associate the given node reverse property `prop`, unless it is already.
	#[inline(always)]
	pub fn insert_unique(&mut self, prop: Id<T, B>, value: Indexed<Node<T, B>>) {
		self.insert_unique_with(Meta::none(prop), Meta::none(value))
	}

	/// Associate all the given nodes reverse property `prop`.
	#[inline(always)]
	pub fn insert_all<Objects: IntoIterator<Item = Indexed<Node<T, B>>>>(
		&mut self,
		prop: Id<T, B>,
		values: Objects,
	) {
		self.insert_all_with(Meta::none(prop), values.into_iter().map(Meta::none))
	}

	/// Associate all the given nodes to the reverse property `prop`, unless it
	/// is already.
	///
	/// The [equivalence operator](crate::Node::equivalent) is used to remove
	/// equivalent nodes.
	#[inline(always)]
	pub fn insert_all_unique<Nodes: IntoIterator<Item = Indexed<Node<T, B>>>>(
		&mut self,
		prop: Id<T, B>,
		values: Nodes,
	) {
		self.insert_all_unique_stripped_with(
			Meta::none(prop),
			values.into_iter().map(|v| Stripped(Meta::none(v))),
		)
	}

	pub fn set(&mut self, prop: Id<T, B>, values: ReversePropertyNodes<T, B>) {
		self.set_with(Meta::none(prop), values)
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> ReverseProperties<T, B, M> {
	/// Checks if the given reverse property is associated to any node.
	#[inline(always)]
	pub fn contains<Q: ?Sized + Hash + indexmap::Equivalent<Id<T, B>>>(&self, prop: &Q) -> bool {
		self.0.get(prop).is_some()
	}

	/// Returns an iterator over all the nodes associated to the given reverse property.
	#[inline(always)]
	pub fn get<'a, Q: ?Sized + Hash + indexmap::Equivalent<Id<T, B>>>(
		&self,
		prop: &Q,
	) -> Nodes<T, B, M>
	where
		T: 'a,
	{
		match self.0.get(prop) {
			Some(values) => Nodes::new(Some(values.iter())),
			None => Nodes::new(None),
		}
	}

	/// Get one of the nodes associated to the given reverse property.
	///
	/// If multiple nodes are found, there are no guaranties on which node will be returned.
	#[inline(always)]
	pub fn get_any<'a, Q: ?Sized + Hash + indexmap::Equivalent<Id<T, B>>>(
		&self,
		prop: &Q,
	) -> Option<&IndexedNode<T, B, M>>
	where
		T: 'a,
	{
		match self.0.get(prop) {
			Some(values) => values.iter().next().map(|n| &n.0),
			None => None,
		}
	}

	/// Associate the given node to the given reverse property.
	#[inline(always)]
	pub fn insert_with(
		&mut self,
		Meta(prop, meta): Meta<Id<T, B>, M>,
		value: IndexedNode<T, B, M>,
	) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			node_values.insert(Stripped(value));
		} else {
			self.0
				.insert(prop, Entry::new(meta, Multiset::singleton(Stripped(value))));
		}
	}

	/// Associate the given node to the given reverse property, unless it is already.
	#[inline(always)]
	pub fn insert_unique_with(
		&mut self,
		Meta(prop, meta): Meta<Id<T, B>, M>,
		value: IndexedNode<T, B, M>,
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

	/// Associate all the given nodes to the given reverse property.
	#[inline(always)]
	pub fn insert_all_with<Objects: IntoIterator<Item = IndexedNode<T, B, M>>>(
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

	/// Associate all the given nodes to the given reverse property, unless it is already.
	#[inline(always)]
	pub fn insert_all_unique_stripped_with<
		Nodes: IntoIterator<Item = Stripped<IndexedNode<T, B, M>>>,
	>(
		&mut self,
		Meta(prop, meta): Meta<Id<T, B>, M>,
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
			let mut node_values: ReversePropertyNodes<T, B, M> =
				Multiset::with_capacity(values.size_hint().0);
			for value in values {
				if node_values.iter().all(|v| !v.equivalent(&value)) {
					node_values.insert(value)
				}
			}

			self.0.insert(prop, Entry::new(meta, node_values));
		}
	}

	/// Associate all the given nodes to the given reverse property, unless it is already.
	#[inline(always)]
	pub fn insert_all_unique_with<Nodes: IntoIterator<Item = IndexedNode<T, B, M>>>(
		&mut self,
		prop: Meta<Id<T, B>, M>,
		values: Nodes,
	) {
		self.insert_all_unique_stripped_with(prop, values.into_iter().map(Stripped))
	}

	pub fn set_with(
		&mut self,
		Meta(prop, meta): Meta<Id<T, B>, M>,
		values: ReversePropertyNodes<T, B, M>,
	) {
		self.0
			.entry(prop)
			.or_insert_with(|| Entry::new(meta, Multiset::new()))
			.value = values
	}

	pub fn extend_unique_with<I, N>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (Meta<Id<T, B>, M>, N)>,
		N: IntoIterator<Item = IndexedNode<T, B, M>>,
	{
		for (prop, values) in iter {
			self.insert_all_unique_with(prop, values)
		}
	}

	pub fn extend_unique_stripped<I, N>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (Meta<Id<T, B>, M>, N)>,
		N: IntoIterator<Item = Stripped<IndexedNode<T, B, M>>>,
	{
		for (prop, values) in iter {
			self.insert_all_unique_stripped_with(prop, values)
		}
	}

	/// Removes and returns all the values associated to the given reverse property.
	#[inline(always)]
	pub fn remove(&mut self, prop: &Id<T, B>) -> Option<ReversePropertyEntry<T, B, M>> {
		self.0.remove(prop)
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> TryFromJson<T, B, M> for ReverseProperties<T, B, M> {
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

impl<T: Eq + Hash, B: Eq + Hash, M> TryFromJsonObject<T, B, M> for ReverseProperties<T, B, M> {
	fn try_from_json_object_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		Meta(object, meta): Meta<json_syntax::Object<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidExpandedJson<M>, M>> {
		let mut result = Self::new();

		for entry in object {
			let Meta(key, key_meta) = entry.key;
			let prop = Id::from_string_in(vocabulary, key.to_string());
			let nodes: Vec<IndexedNode<T, B, M>> =
				Vec::try_from_json_in(vocabulary, entry.value)?.into_value();
			result.insert_all_with(Meta(prop, key_meta), nodes)
		}

		Ok(Meta(result, meta))
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> locspan::StrippedPartialEq for ReverseProperties<T, B, M> {
	#[inline(always)]
	fn stripped_eq(&self, other: &Self) -> bool {
		self.0.stripped() == other.0.stripped()
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> locspan::StrippedEq for ReverseProperties<T, B, M> {}

impl<T: Hash, B: Hash, M> locspan::StrippedHash for ReverseProperties<T, B, M> {
	#[inline(always)]
	fn stripped_hash<H: Hasher>(&self, h: &mut H) {
		crate::utils::hash_map_stripped(&self.0, h)
	}
}

impl<T: Hash, B: Hash, M: Hash> Hash for ReverseProperties<T, B, M> {
	#[inline(always)]
	fn hash<H: Hasher>(&self, h: &mut H) {
		crate::utils::hash_map(&self.0, h)
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> Extend<(Meta<Id<T, B>, M>, Vec<IndexedNode<T, B, M>>)>
	for ReverseProperties<T, B, M>
{
	fn extend<I>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (Meta<Id<T, B>, M>, Vec<IndexedNode<T, B, M>>)>,
	{
		for (prop, values) in iter {
			self.insert_all_with(prop, values)
		}
	}
}

/// Tuple type representing a reverse binding in a node object,
/// associating a reverse property to some nodes.
pub type ReverseBinding<T, B, M> = (Meta<Id<T, B>, M>, ReversePropertyNodes<T, B, M>);

/// Tuple type representing a reference to a reverse binding in a node object,
/// associating a reverse property to some nodes.
pub type ReverseBindingRef<'a, T, B, M> = (
	Meta<&'a Id<T, B>, &'a M>,
	&'a [StrippedIndexedNode<T, B, M>],
);

/// Tuple type representing a mutable reference to a reverse binding in a node object,
/// associating a reverse property to some nodes, with a mutable access to the nodes.
pub type ReverseBindingMut<'a, T, B, M> = (
	Meta<&'a Id<T, B>, &'a mut M>,
	&'a mut ReversePropertyNodes<T, B, M>,
);

impl<T, B, M> IntoIterator for ReverseProperties<T, B, M> {
	type Item = ReverseBinding<T, B, M>;
	type IntoIter = IntoIter<T, B, M>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			inner: self.0.into_iter(),
		}
	}
}

impl<'a, T, B, M> IntoIterator for &'a ReverseProperties<T, B, M> {
	type Item = ReverseBindingRef<'a, T, B, M>;
	type IntoIter = Iter<'a, T, B, M>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, T, B, M> IntoIterator for &'a mut ReverseProperties<T, B, M> {
	type Item = ReverseBindingMut<'a, T, B, M>;
	type IntoIter = IterMut<'a, T, B, M>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

/// Iterator over the reverse properties of a node.
///
/// It is created by the [`ReverseProperties::into_iter`] function.
pub struct IntoIter<T, B, M> {
	inner: indexmap::map::IntoIter<Id<T, B>, ReversePropertyEntry<T, B, M>>,
}

impl<T, B, M> Iterator for IntoIter<T, B, M> {
	type Item = ReverseBinding<T, B, M>;

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

/// Iterator over the reverse properties of a node.
///
/// It is created by the [`ReverseProperties::iter`] function.
#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub struct Iter<'a, T, B, M> {
	inner: indexmap::map::Iter<'a, Id<T, B>, ReversePropertyEntry<T, B, M>>,
}

impl<'a, T, B, M> Iterator for Iter<'a, T, B, M> {
	type Item = ReverseBindingRef<'a, T, B, M>;

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

pub type ReversePropertyEntry<T, B, M> = Entry<ReversePropertyNodes<T, B, M>, M>;

/// Iterator over the reverse properties of a node, giving a mutable reference
/// to the associated nodes.
///
/// It is created by the [`ReverseProperties::iter_mut`] function.
pub struct IterMut<'a, T, B, M> {
	inner: indexmap::map::IterMut<'a, Id<T, B>, ReversePropertyEntry<T, B, M>>,
}

impl<'a, T, B, M> Iterator for IterMut<'a, T, B, M> {
	type Item = ReverseBindingMut<'a, T, B, M>;

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

impl<T, B, M: Clone, N: Vocabulary<Iri = T, BlankId = B>> IntoJsonWithContextMeta<M, N>
	for ReverseProperties<T, B, M>
{
	fn into_json_meta_with(self, meta: M, vocabulary: &N) -> Meta<json_syntax::Value<M>, M> {
		let mut obj = json_syntax::Object::new();

		for (Meta(prop, meta), nodes) in self {
			obj.insert(
				Meta(prop.with(vocabulary).to_string().into(), meta.clone()),
				nodes.into_json_meta_with(meta, vocabulary),
			);
		}

		Meta(obj.into(), meta)
	}
}
