use crate::{id, Id, Indexed, Object, StrippedIndexedObject};
use rdf_types::BlankId;
use std::collections::{BTreeSet, HashSet};

/// Result of the document expansion algorithm.
///
/// It is just an alias for a set of (indexed) objects.
pub struct ExpandedDocument<T: Id, M>(HashSet<StrippedIndexedObject<T, M>>);

impl<T: Id, M> ExpandedDocument<T, M> {
	#[inline(always)]
	pub fn new(objects: HashSet<StrippedIndexedObject<T, M>>) -> Self {
		Self(objects)
	}

	#[inline(always)]
	pub fn len(&self) -> usize {
		self.0.len()
	}

	#[inline(always)]
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	#[inline(always)]
	pub fn objects(&self) -> &HashSet<StrippedIndexedObject<T, M>> {
		&self.0
	}

	#[inline(always)]
	pub fn into_objects(self) -> HashSet<StrippedIndexedObject<T, M>> {
		self.0
	}

	#[inline(always)]
	pub fn iter(&self) -> std::collections::hash_set::Iter<'_, StrippedIndexedObject<T, M>> {
		self.0.iter()
	}

	#[inline(always)]
	pub fn identify_all<G: id::Generator<T>>(&mut self, generator: &mut G) {
		let mut objects = HashSet::new();
		std::mem::swap(&mut self.0, &mut objects);

		for mut object in objects {
			object.identify_all(generator);
			self.0.insert(object);
		}
	}

	/// Returns the set of all blank identifiers in the given document.
	pub fn blank_ids(&self) -> BTreeSet<&BlankId> {
		let mut blank_ids = BTreeSet::new();

		fn collect_reference<'a, T>(
			ids: &mut BTreeSet<&'a BlankId>,
			r: crate::reference::Ref<'a, T>,
		) {
			if let crate::reference::Ref::Blank(id) = r {
				ids.insert(id);
			}
		}

		for object in self {
			for object_ref in object.traverse() {
				match object_ref {
					crate::object::Ref::Node(node) => {
						if let Some(id) = node.id() {
							collect_reference(&mut blank_ids, id.as_ref())
						}

						for (r, _) in node.properties() {
							collect_reference(&mut blank_ids, r.as_ref())
						}

						for (r, _) in node.reverse_properties() {
							collect_reference(&mut blank_ids, r.as_ref())
						}
					}
					crate::object::Ref::Value(value) => {
						if let Some(ty) = value.typ() {
							if let Some(r) = ty.into_reference() {
								collect_reference(&mut blank_ids, r)
							}
						}
					}
					_ => (),
				}
			}
		}

		blank_ids
	}
}

impl<T: Id + PartialEq, M> PartialEq for ExpandedDocument<T, M> {
	/// Comparison between two expanded documents.
	///
	/// Warnings are not compared.
	fn eq(&self, other: &Self) -> bool {
		self.0.eq(&other.0)
	}
}

impl<T: Id + Eq, M> Eq for ExpandedDocument<T, M> {}

impl<T: Id, M> IntoIterator for ExpandedDocument<T, M> {
	type IntoIter = IntoIter<T, M>;
	type Item = Indexed<Object<T, M>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		IntoIter(self.0.into_iter())
	}
}

impl<'a, T: Id, M> IntoIterator for &'a ExpandedDocument<T, M> {
	type IntoIter = std::collections::hash_set::Iter<'a, StrippedIndexedObject<T, M>>;
	type Item = &'a StrippedIndexedObject<T, M>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct IntoIter<T: Id, M>(std::collections::hash_set::IntoIter<StrippedIndexedObject<T, M>>);

impl<T: Id, M> Iterator for IntoIter<T, M> {
	type Item = Indexed<Object<T, M>>;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|s| s.0)
	}
}

// impl<F, J: JsonHash + JsonClone, K: JsonFrom<J>, T: Id> AsJson<J, K> for ExpandedDocument<T, M> {
// 	fn as_json_with(
// 		&self,
// 		meta: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
// 	) -> K {
// 		self.0.as_json_with(meta)
// 	}
// }
