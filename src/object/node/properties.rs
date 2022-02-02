use super::Objects;
use crate::{Id, Indexed, Object, Reference, ToReference};
use generic_json::JsonHash;
use std::{
	borrow::Borrow,
	collections::HashMap,
	hash::{Hash, Hasher},
};

/// Properties of a node object, and their associated objects.
///
/// ## Example
///
/// ```rust
/// use json_ld::{context, Document, NoLoader};
/// use serde_json::Value;
/// let doc: Value = serde_json::from_str(
///   r#"
///   {
///      "@context": {
///        "name": "http://xmlns.com/foaf/0.1/name"
///      },
///      "@id": "https://www.rust-lang.org",
///      "name": "Rust Programming Language"
///    }
/// "#,
/// )
/// .unwrap();
///
/// let mut loader = NoLoader::<Value>::new();
///
/// let rt  = tokio::runtime::Runtime::new().unwrap();
/// let expanded_doc = rt.block_on(doc
///   .expand::<context::Json<Value>, _>(&mut loader)).unwrap();
///
/// let node = expanded_doc.into_iter().next().unwrap().into_indexed_node().unwrap();
///
/// for (property, objects) in node.properties() {
///   assert_eq!(property, "http://xmlns.com/foaf/0.1/name");
///   for object in objects {
///     // do something
///   }
/// }
/// ```
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

	/// Associate the given object to the node through the given property, unless it is already.
	#[inline(always)]
	pub fn insert_unique(&mut self, prop: Reference<T>, value: Indexed<Object<J, T>>) {
		if let Some(node_values) = self.0.get_mut(&prop) {
			if node_values.iter().all(|v| !v.equivalent(&value)) {
				node_values.push(value)
			}
		} else {
			let node_values = vec![value];
			self.0.insert(prop, node_values);
		}
	}

	/// Associate all the given objects to the node through the given property.
	#[inline(always)]
	pub fn insert_all<Objects: IntoIterator<Item = Indexed<Object<J, T>>>>(
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

	/// Associate all the given objects to the node through the given property, unless it is already.
	///
	/// The [equivalence operator](Object::equivalent) is used to remove equivalent objects.
	#[inline(always)]
	pub fn insert_all_unique<Objects: IntoIterator<Item = Indexed<Object<J, T>>>>(
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
			let mut node_values: Vec<Indexed<Object<J, T>>> =
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
		I: IntoIterator<Item = (Reference<T>, Vec<Indexed<Object<J, T>>>)>,
	{
		for (prop, values) in iter {
			self.insert_all_unique(prop, values)
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

	/// Removes and returns all the values associated to the given property.
	#[inline(always)]
	pub fn remove(&mut self, prop: &Reference<T>) -> Option<Vec<Indexed<Object<J, T>>>> {
		self.0.remove(prop)
	}

	/// Removes all properties.
	#[inline(always)]
	pub fn clear(&mut self) {
		self.0.clear()
	}

	#[inline(always)]
	pub fn traverse(&self) -> Traverse<J, T> {
		Traverse {
			current_object: None,
			current_property: None,
			iter: self.0.iter(),
		}
	}
}

impl<J: JsonHash, T: Id> Hash for Properties<J, T> {
	#[inline(always)]
	fn hash<H: Hasher>(&self, h: &mut H) {
		crate::util::hash_map(&self.0, h)
	}
}

impl<J: JsonHash, T: Id> Extend<(Reference<T>, Vec<Indexed<Object<J, T>>>)> for Properties<J, T> {
	fn extend<I>(&mut self, iter: I)
	where
		I: IntoIterator<Item = (Reference<T>, Vec<Indexed<Object<J, T>>>)>,
	{
		for (prop, values) in iter {
			self.insert_all(prop, values)
		}
	}
}

/// Tuple type representing a binding in a node object,
/// associating a property to some objects.
pub type Binding<J, T> = (Reference<T>, Vec<Indexed<Object<J, T>>>);

/// Tuple type representing a reference to a binding in a node object,
/// associating a property to some objects.
pub type BindingRef<'a, J, T> = (&'a Reference<T>, &'a [Indexed<Object<J, T>>]);

/// Tuple type representing a mutable reference to a binding in a node object,
/// associating a property to some objects, with a mutable access to the objects.
pub type BindingMut<'a, J, T> = (&'a Reference<T>, &'a mut Vec<Indexed<Object<J, T>>>);

impl<J: JsonHash, T: Id> IntoIterator for Properties<J, T> {
	type Item = Binding<J, T>;
	type IntoIter = IntoIter<J, T>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			inner: self.0.into_iter(),
		}
	}
}

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
/// It is created by the [`Properties::into_iter`] function.
pub struct IntoIter<J: JsonHash, T: Id> {
	inner: std::collections::hash_map::IntoIter<Reference<T>, Vec<Indexed<Object<J, T>>>>,
}

impl<J: JsonHash, T: Id> Iterator for IntoIter<J, T> {
	type Item = Binding<J, T>;

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

pub struct Traverse<'a, J: JsonHash, T: Id> {
	current_object: Option<Box<crate::object::Traverse<'a, J, T>>>,
	current_property: Option<std::slice::Iter<'a, Indexed<Object<J, T>>>>,
	iter: std::collections::hash_map::Iter<'a, Reference<T>, Vec<Indexed<Object<J, T>>>>,
}

impl<'a, J: JsonHash, T: Id> Iterator for Traverse<'a, J, T> {
	type Item = crate::object::Ref<'a, J, T>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match &mut self.current_object {
				Some(current_object) => match current_object.next() {
					Some(next) => break Some(next),
					None => self.current_object = None,
				},
				None => match &mut self.current_property {
					Some(current_property) => match current_property.next() {
						Some(object) => self.current_object = Some(Box::new(object.traverse())),
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
