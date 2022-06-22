use crate::{
	id,
	utils::{AsJson, JsonFrom},
	Id, Indexed, Loc, Node, Warning,
};
use generic_json::{Json, JsonClone, JsonHash};

/// Result of the document flattening algorithm.
///
/// It is just an alias for a set of (indexed) nodes.
pub struct FlattenedDocument<F, J: JsonHash, T: Id> {
	nodes: Vec<Indexed<Node<J, T>>>,
	warnings: Vec<Loc<Warning, F, J::MetaData>>,
}

impl<F, J: JsonHash, T: Id> FlattenedDocument<F, J, T> {
	#[inline(always)]
	pub fn new(
		nodes: Vec<Indexed<Node<J, T>>>,
		warnings: Vec<Loc<Warning, F, J::MetaData>>,
	) -> Self {
		Self { nodes, warnings }
	}

	#[inline(always)]
	pub fn len(&self) -> usize {
		self.nodes.len()
	}

	#[inline(always)]
	pub fn is_empty(&self) -> bool {
		self.nodes.is_empty()
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
	pub fn nodes(&self) -> &[Indexed<Node<J, T>>] {
		&self.nodes
	}

	#[inline(always)]
	pub fn into_nodes(self) -> Vec<Indexed<Node<J, T>>> {
		self.nodes
	}

	#[inline(always)]
	pub fn iter(&self) -> std::slice::Iter<'_, Indexed<Node<J, T>>> {
		self.nodes.iter()
	}

	#[inline(always)]
	pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Indexed<Node<J, T>>> {
		self.nodes.iter_mut()
	}

	#[inline(always)]
	#[allow(clippy::type_complexity)]
	pub fn into_parts(self) -> (Vec<Indexed<Node<J, T>>>, Vec<Loc<Warning, F, J::MetaData>>) {
		(self.nodes, self.warnings)
	}

	#[inline(always)]
	pub fn identify_all<G: id::Generator<T>>(&mut self, mut generator: G) {
		for node in &mut self.nodes {
			node.identify_all(&mut generator)
		}
	}
}

impl<F, J: JsonHash, T: Id> IntoIterator for FlattenedDocument<F, J, T> {
	type IntoIter = std::vec::IntoIter<Indexed<Node<J, T>>>;
	type Item = Indexed<Node<J, T>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.nodes.into_iter()
	}
}

impl<'a, F, J: JsonHash, T: Id> IntoIterator for &'a FlattenedDocument<F, J, T> {
	type IntoIter = std::slice::Iter<'a, Indexed<Node<J, T>>>;
	type Item = &'a Indexed<Node<J, T>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, F, J: JsonHash, T: Id> IntoIterator for &'a mut FlattenedDocument<F, J, T> {
	type IntoIter = std::slice::IterMut<'a, Indexed<Node<J, T>>>;
	type Item = &'a mut Indexed<Node<J, T>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

impl<F, J: JsonHash + JsonClone, K: JsonFrom<J>, T: Id> AsJson<J, K>
	for FlattenedDocument<F, J, T>
{
	fn as_json_with(
		&self,
		meta: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
	) -> K {
		self.nodes.as_json_with(meta)
	}
}
