use super::{Multiset, Nodes};
use crate::{
	object::{InvalidExpandedJson, TryFromJson, TryFromJsonObject},
	Id, IndexedNode,
};
use contextual::WithContext;
use educe::Educe;
use indexmap::IndexMap;
use iref::IriBuf;
use json_ld_syntax::IntoJsonWithContext;
use rdf_types::{BlankIdBuf, Vocabulary, VocabularyMut};
use std::hash::{Hash, Hasher};

pub type ReversePropertyNodes<T = IriBuf, B = BlankIdBuf> = Multiset<IndexedNode<T, B>>;

/// Reverse properties of a node object, and their associated nodes.
#[derive(Educe, Debug, Clone)]
#[educe(
	PartialEq(bound = "T: Eq + Hash, B: Eq + Hash"),
	Eq(bound = "T: Eq + Hash, B: Eq + Hash")
)]
pub struct ReverseProperties<T = IriBuf, B = BlankIdBuf>(
	IndexMap<Id<T, B>, ReversePropertyNodes<T, B>>,
);

impl<T, B> Default for ReverseProperties<T, B> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T, B> ReverseProperties<T, B> {
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
	pub fn iter(&self) -> Iter<'_, T, B> {
		Iter {
			inner: self.0.iter(),
		}
	}

	/// Returns an iterator over the reverse properties with a mutable reference to their associated nodes.
	#[inline(always)]
	pub fn iter_mut(&mut self) -> IterMut<'_, T, B> {
		self.0.iter_mut()
	}

	/// Removes all reverse properties.
	#[inline(always)]
	pub fn clear(&mut self) {
		self.0.clear()
	}
}

impl<T: Eq + Hash, B: Eq + Hash> ReverseProperties<T, B> {
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
	) -> Nodes<T, B>
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
	) -> Option<&IndexedNode<T, B>>
	where
		T: 'a,
	{
		match self.0.get(prop) {
			Some(values) => values.iter().next(),
			None => None,
		}
	}

	/// Associate the given node to the given reverse property.
	#[inline(always)]
	pub fn insert(&mut self, prop: Id<T, B>, value: IndexedNode<T, B>) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			node_values.insert(value);
		} else {
			self.0.insert(prop, Multiset::singleton(value));
		}
	}

	/// Associate the given node to the given reverse property, unless it is already.
	#[inline(always)]
	pub fn insert_unique(&mut self, prop: Id<T, B>, value: IndexedNode<T, B>) {
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
	pub fn insert_all<Objects: IntoIterator<Item = IndexedNode<T, B>>>(
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

	/// Associate all the given nodes to the given reverse property, unless it is already.
	#[inline(always)]
	pub fn insert_all_unique<Nodes: IntoIterator<Item = IndexedNode<T, B>>>(
		&mut self,
		prop: Id<T, B>,
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
			let mut node_values: ReversePropertyNodes<T, B> =
				Multiset::with_capacity(values.size_hint().0);
			for value in values {
				if node_values.iter().all(|v| !v.equivalent(&value)) {
					node_values.insert(value)
				}
			}

			self.0.insert(prop, node_values);
		}
	}

	pub fn set(&mut self, prop: Id<T, B>, values: ReversePropertyNodes<T, B>) {
		self.0.insert(prop, values);
	}

	pub fn extend_unique<N>(&mut self, iter: impl IntoIterator<Item = (Id<T, B>, N)>)
	where
		N: IntoIterator<Item = IndexedNode<T, B>>,
	{
		for (prop, values) in iter {
			self.insert_all_unique(prop, values)
		}
	}

	/// Removes and returns all the values associated to the given reverse property.
	#[inline(always)]
	pub fn remove(&mut self, prop: &Id<T, B>) -> Option<ReversePropertyNodes<T, B>> {
		self.0.swap_remove(prop)
	}
}

impl<T: Eq + Hash, B: Eq + Hash, N> FromIterator<(Id<T, B>, N)> for ReverseProperties<T, B>
where
	N: IntoIterator<Item = IndexedNode<T, B>>,
{
	fn from_iter<I: IntoIterator<Item = (Id<T, B>, N)>>(iter: I) -> Self {
		let mut result = Self::default();
		for (id, values) in iter {
			result.insert_all(id, values);
		}
		result
	}
}

impl<T: Eq + Hash, B: Eq + Hash> TryFromJson<T, B> for ReverseProperties<T, B> {
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

impl<T: Eq + Hash, B: Eq + Hash> TryFromJsonObject<T, B> for ReverseProperties<T, B> {
	fn try_from_json_object_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		object: json_syntax::Object,
	) -> Result<Self, InvalidExpandedJson> {
		let mut result = Self::new();

		for entry in object {
			let prop = Id::from_string_in(vocabulary, entry.key.to_string());
			let nodes: Vec<IndexedNode<T, B>> = Vec::try_from_json_in(vocabulary, entry.value)?;
			result.insert_all(prop, nodes)
		}

		Ok(result)
	}
}

impl<T: Hash, B: Hash> Hash for ReverseProperties<T, B> {
	#[inline(always)]
	fn hash<H: Hasher>(&self, h: &mut H) {
		crate::utils::hash_map(&self.0, h)
	}
}

impl<T: Eq + Hash, B: Eq + Hash> Extend<(Id<T, B>, Vec<IndexedNode<T, B>>)>
	for ReverseProperties<T, B>
{
	fn extend<I>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (Id<T, B>, Vec<IndexedNode<T, B>>)>,
	{
		for (prop, values) in iter {
			self.insert_all(prop, values)
		}
	}
}

/// Tuple type representing a reverse binding in a node object,
/// associating a reverse property to some nodes.
pub type ReverseBinding<T, B> = (Id<T, B>, ReversePropertyNodes<T, B>);

/// Tuple type representing a reference to a reverse binding in a node object,
/// associating a reverse property to some nodes.
pub type ReverseBindingRef<'a, T, B> = (&'a Id<T, B>, &'a [IndexedNode<T, B>]);

/// Tuple type representing a mutable reference to a reverse binding in a node object,
/// associating a reverse property to some nodes, with a mutable access to the nodes.
pub type ReverseBindingMut<'a, T, B> = (&'a Id<T, B>, &'a mut ReversePropertyNodes<T, B>);

impl<T, B> IntoIterator for ReverseProperties<T, B> {
	type Item = ReverseBinding<T, B>;
	type IntoIter = IntoIter<T, B>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<'a, T, B> IntoIterator for &'a ReverseProperties<T, B> {
	type Item = ReverseBindingRef<'a, T, B>;
	type IntoIter = Iter<'a, T, B>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, T, B> IntoIterator for &'a mut ReverseProperties<T, B> {
	type Item = ReverseBindingMut<'a, T, B>;
	type IntoIter = IterMut<'a, T, B>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

/// Iterator over the reverse properties of a node.
///
/// It is created by the [`ReverseProperties::into_iter`] function.
pub type IntoIter<T, B> = indexmap::map::IntoIter<Id<T, B>, ReversePropertyNodes<T, B>>;

/// Iterator over the reverse properties of a node.
///
/// It is created by the [`ReverseProperties::iter`] function.
#[derive(Educe)]
#[educe(Clone)]
pub struct Iter<'a, T, B> {
	inner: indexmap::map::Iter<'a, Id<T, B>, ReversePropertyNodes<T, B>>,
}

impl<'a, T, B> Iterator for Iter<'a, T, B> {
	type Item = ReverseBindingRef<'a, T, B>;

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

/// Iterator over the reverse properties of a node, giving a mutable reference
/// to the associated nodes.
///
/// It is created by the [`ReverseProperties::iter_mut`] function.
pub type IterMut<'a, T, B> = indexmap::map::IterMut<'a, Id<T, B>, ReversePropertyNodes<T, B>>;

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> IntoJsonWithContext<N> for ReverseProperties<T, B> {
	fn into_json_with(self, vocabulary: &N) -> json_syntax::Value {
		let mut obj = json_syntax::Object::new();

		for (prop, nodes) in self {
			obj.insert(
				prop.with(vocabulary).to_string().into(),
				nodes.into_json_with(vocabulary),
			);
		}

		obj.into()
	}
}
