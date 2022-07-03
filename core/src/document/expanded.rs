use crate::{id, Id, Indexed, Loc, Object, Warning};
use rdf_types::BlankId;
use std::collections::{BTreeSet, HashSet};
use locspan::Span;

/// Result of the document expansion algorithm.
///
/// It is just an alias for a set of (indexed) objects.
pub struct ExpandedDocument<T: Id, S, P=Span> {
	objects: HashSet<Indexed<Object<T>>>,
	warnings: Vec<Loc<Warning, S, P>>,
}

impl<T: Id, S, P> ExpandedDocument<T, S, P> {
	#[inline(always)]
	pub fn new(
		objects: HashSet<Indexed<Object<T>>>,
		warnings: Vec<Loc<Warning, S, P>>,
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
	pub fn warnings(&self) -> &[Loc<Warning, S, P>] {
		&self.warnings
	}

	#[inline(always)]
	pub fn into_warnings(self) -> Vec<Loc<Warning, S, P>> {
		self.warnings
	}

	#[inline(always)]
	pub fn objects(&self) -> &HashSet<Indexed<Object<T>>> {
		&self.objects
	}

	#[inline(always)]
	pub fn into_objects(self) -> HashSet<Indexed<Object<T>>> {
		self.objects
	}

	#[inline(always)]
	pub fn iter(&self) -> std::collections::hash_set::Iter<'_, Indexed<Object<T>>> {
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
		HashSet<Indexed<Object<T>>>,
		Vec<Loc<Warning, S, P>>,
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

impl<T: Id + PartialEq, S, P> PartialEq for ExpandedDocument<T, S, P> {
	/// Comparison between two expanded documents.
	///
	/// Warnings are not compared.
	fn eq(&self, other: &Self) -> bool {
		self.objects.eq(&other.objects)
	}
}

impl<T: Id + Eq, S, P> Eq for ExpandedDocument<T, S, P> {}

impl<T: Id, S, P> IntoIterator for ExpandedDocument<T, S, P> {
	type IntoIter = std::collections::hash_set::IntoIter<Indexed<Object<T>>>;
	type Item = Indexed<Object<T>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.objects.into_iter()
	}
}

impl<'a, T: Id, S, P> IntoIterator for &'a ExpandedDocument<T, S, P> {
	type IntoIter = std::collections::hash_set::Iter<'a, Indexed<Object<T>>>;
	type Item = &'a Indexed<Object<T>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

// impl<F, J: JsonHash + JsonClone, K: JsonFrom<J>, T: Id> AsJson<J, K> for ExpandedDocument<T, S, P> {
// 	fn as_json_with(
// 		&self,
// 		meta: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
// 	) -> K {
// 		self.objects.as_json_with(meta)
// 	}
// }
