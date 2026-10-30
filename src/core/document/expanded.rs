use btree_indexmap::BTreeIndexSet;

use crate::{
	IndexedJsonLdObject, JsonLdIndexed, JsonLdNode, JsonLdObject, JsonLdVisit,
	object::{JsonLdObjectMut, JsonLdObjectRef},
};

/// Result of the document expansion algorithm.
///
/// It is just an alias for a set of (indexed) objects.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JsonLdExpandedDocument(BTreeIndexSet<IndexedJsonLdObject>);

impl JsonLdExpandedDocument {
	#[inline(always)]
	pub fn new() -> Self {
		Self::default()
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
	pub fn objects(&self) -> &BTreeIndexSet<IndexedJsonLdObject> {
		&self.0
	}

	#[inline(always)]
	pub fn into_objects(self) -> BTreeIndexSet<IndexedJsonLdObject> {
		self.0
	}

	#[inline(always)]
	pub fn iter(&self) -> btree_indexmap::set::Iter<'_, IndexedJsonLdObject> {
		self.0.iter()
	}

	/// Returns the main node object of the document, if any.
	///
	/// The main node is the unique top level (root) node object. If multiple
	/// node objects are on the root, `None` is returned.
	pub fn main_node(&self) -> Option<&JsonLdNode> {
		let mut result = None;

		for object in self {
			if let JsonLdObject::Node(node) = object.inner() {
				if result.is_some() {
					return None;
				}

				result = Some(&**node)
			}
		}

		result
	}

	/// Consumes the document and returns its main node object, if any.
	///
	/// The main node is the unique top level (root) node object. If multiple
	/// node objects are on the root, `None` is returned.
	pub fn into_main_node(self) -> Option<JsonLdNode> {
		let mut result = None;

		for object in self {
			if let JsonLdObject::Node(node) = object.into_inner() {
				if result.is_some() {
					return None;
				}

				result = Some(*node)
			}
		}

		result
	}

	#[inline(always)]
	pub fn insert(&mut self, object: IndexedJsonLdObject) -> bool {
		self.0.insert(object)
	}
}

#[cfg(feature = "serde")]
impl JsonLdExpandedDocument {
	/// Returns the JSON representation of this expanded document, that is a
	/// JSON array of its expanded objects.
	pub fn to_json(&self) -> json_syntax::JsonValue {
		// An expanded document is, by definition, a valid JSON value: it only
		// contains JSON-LD objects, which are made of strings, numbers,
		// booleans, arrays and objects with string keys. Serializing it to a
		// JSON value therefore cannot fail.
		json_syntax::to_value(self).expect("an expanded document is a valid JSON value")
	}
}

#[cfg(feature = "serde")]
impl From<JsonLdExpandedDocument> for json_syntax::JsonValue {
	fn from(document: JsonLdExpandedDocument) -> Self {
		document.to_json()
	}
}

impl JsonLdVisit for JsonLdExpandedDocument {
	fn visit_with(&self, f: &mut impl FnMut(JsonLdObjectRef)) {
		self.0.visit_with(f);
	}

	fn visit_mut_with(&mut self, f: &mut impl FnMut(JsonLdObjectMut)) {
		self.0.visit_mut_with(f);
	}
}

impl From<JsonLdIndexed<JsonLdNode>> for JsonLdExpandedDocument {
	fn from(value: JsonLdIndexed<JsonLdNode>) -> Self {
		let mut result = Self::default();

		result.insert(value.map_inner(JsonLdObject::node));

		result
	}
}

impl IntoIterator for JsonLdExpandedDocument {
	type IntoIter = IntoIter;
	type Item = IndexedJsonLdObject;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		IntoIter(self.0.into_iter())
	}
}

impl<'a> IntoIterator for &'a JsonLdExpandedDocument {
	type IntoIter = btree_indexmap::set::Iter<'a, IndexedJsonLdObject>;
	type Item = &'a IndexedJsonLdObject;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}
pub struct IntoIter(btree_indexmap::set::IntoIter<IndexedJsonLdObject>);

impl Iterator for IntoIter {
	type Item = IndexedJsonLdObject;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

impl FromIterator<IndexedJsonLdObject> for JsonLdExpandedDocument {
	fn from_iter<I: IntoIterator<Item = IndexedJsonLdObject>>(iter: I) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl Extend<IndexedJsonLdObject> for JsonLdExpandedDocument {
	fn extend<I: IntoIterator<Item = IndexedJsonLdObject>>(&mut self, iter: I) {
		self.0.extend(iter)
	}
}

impl From<BTreeIndexSet<IndexedJsonLdObject>> for JsonLdExpandedDocument {
	fn from(set: BTreeIndexSet<IndexedJsonLdObject>) -> Self {
		Self(set)
	}
}

#[cfg(all(test, feature = "serde"))]
mod tests {
	use json_syntax::JsonValue;
	use rdf_syntax::{Id, iri};

	use super::*;
	use crate::JsonLdLenient;

	#[test]
	fn empty_document_to_json() {
		assert_eq!(
			JsonLdExpandedDocument::new().to_json(),
			JsonValue::Array(Vec::new())
		);
	}

	#[test]
	fn node_document_to_json() {
		let id = JsonLdLenient::Valid(Id::Iri(iri!("https://example.org/a").to_owned()));
		let document = JsonLdExpandedDocument::from(JsonLdIndexed::new(
			JsonLdNode::new_with_id(Some(id)),
			None,
		));

		let json = document.to_json();
		let JsonValue::Array(objects) = &json else {
			panic!("an expanded document must be a JSON array")
		};
		assert_eq!(objects.len(), 1);

		assert_eq!(JsonValue::from(document), json);
	}
}
