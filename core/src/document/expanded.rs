use crate::{id, utils::JsonFrom, Id, Indexed, Loc, Object, Warning};
use generic_json::{JsonClone, JsonHash};
use rdf_types::BlankId;
use std::collections::{BTreeSet, HashSet};

/// Result of the document expansion algorithm.
///
/// It is just an alias for a set of (indexed) objects.
pub struct ExpandedDocument<F, J: JsonHash, T: Id> {
	objects: HashSet<Indexed<Object<J, T>>>,
	warnings: Vec<Loc<Warning, F, J::MetaData>>,
}

impl<F, J: JsonHash, T: Id> ExpandedDocument<F, J, T> {
	#[inline(always)]
	pub fn new(
		objects: HashSet<Indexed<Object<J, T>>>,
		warnings: Vec<Loc<Warning, F, J::MetaData>>,
	) -> Self {
		Self { objects, warnings }
	}

	#[inline(always)]
	pub fn len(&self) -> usize {
		self.objects.len()
	}

	#[inline(always)]
	pub fn is_empty(&self) -> bool {
		self.objects.is_empty()
	}

	#[inline(always)]
	pub fn warnings(&self) -> &[Loc<Warning, F, J::MetaData>] {
		&self.warnings
	}

	#[inline(always)]
	pub fn into_warnings(self) -> Vec<Loc<Warning, F, J::MetaData>> {
		self.warnings
	}

	#[inline(always)]
	pub fn objects(&self) -> &HashSet<Indexed<Object<J, T>>> {
		&self.objects
	}

	#[inline(always)]
	pub fn into_objects(self) -> HashSet<Indexed<Object<J, T>>> {
		self.objects
	}

	#[inline(always)]
	pub fn iter(&self) -> std::collections::hash_set::Iter<'_, Indexed<Object<J, T>>> {
		self.objects.iter()
	}

	#[inline(always)]
	pub fn identify_all<G: id::Generator<T>>(&mut self, generator: &mut G) {
		let mut objects = HashSet::new();
		std::mem::swap(&mut self.objects, &mut objects);

		for mut object in objects {
			object.identify_all(generator);
			self.objects.insert(object);
		}
	}

	#[inline(always)]
	#[allow(clippy::type_complexity)]
	pub fn into_parts(
		self,
	) -> (
		HashSet<Indexed<Object<J, T>>>,
		Vec<Loc<Warning, F, J::MetaData>>,
	) {
		(self.objects, self.warnings)
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

impl<F, J: JsonHash + PartialEq, T: Id + PartialEq> PartialEq for ExpandedDocument<F, J, T> {
	/// Comparison between two expanded documents.
	///
	/// Warnings are not compared.
	fn eq(&self, other: &Self) -> bool {
		self.objects.eq(&other.objects)
	}
}

impl<F, J: JsonHash + Eq, T: Id + Eq> Eq for ExpandedDocument<F, J, T> {}

impl<F, J: JsonHash, T: Id> IntoIterator for ExpandedDocument<F, J, T> {
	type IntoIter = std::collections::hash_set::IntoIter<Indexed<Object<J, T>>>;
	type Item = Indexed<Object<J, T>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.objects.into_iter()
	}
}

impl<'a, F, J: JsonHash, T: Id> IntoIterator for &'a ExpandedDocument<F, J, T> {
	type IntoIter = std::collections::hash_set::Iter<'a, Indexed<Object<J, T>>>;
	type Item = &'a Indexed<Object<J, T>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

// impl<F, J: JsonHash + JsonClone, K: JsonFrom<J>, T: Id> AsJson<J, K> for ExpandedDocument<F, J, T> {
// 	fn as_json_with(
// 		&self,
// 		meta: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
// 	) -> K {
// 		self.objects.as_json_with(meta)
// 	}
// }
