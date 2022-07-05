use crate::{
	id,
	Id, Indexed, Node, Warning,
};
use locspan::Meta;

/// Result of the document flattening algorithm.
///
/// It is just an alias for a set of (indexed) nodes.
pub struct FlattenedDocument<T: Id, M> {
	nodes: Vec<Indexed<Node<T, M>>>,
	warnings: Vec<Meta<Warning, M>>,
}

impl<T: Id, M> FlattenedDocument<T, M> {
	#[inline(always)]
	pub fn new(
		nodes: Vec<Indexed<Node<T, M>>>,
		warnings: Vec<Meta<Warning, M>>,
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
	pub fn warnings(&self) -> &[Meta<Warning, M>] {
		&self.warnings
	}

	#[inline(always)]
	pub fn into_warnings(self) -> Vec<Meta<Warning, M>> {
		self.warnings
	}

	#[inline(always)]
	pub fn nodes(&self) -> &[Indexed<Node<T, M>>] {
		&self.nodes
	}

	#[inline(always)]
	pub fn into_nodes(self) -> Vec<Indexed<Node<T, M>>> {
		self.nodes
	}

	#[inline(always)]
	pub fn iter(&self) -> std::slice::Iter<'_, Indexed<Node<T, M>>> {
		self.nodes.iter()
	}

	#[inline(always)]
	pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Indexed<Node<T, M>>> {
		self.nodes.iter_mut()
	}

	#[inline(always)]
	#[allow(clippy::type_complexity)]
	pub fn into_parts(self) -> (Vec<Indexed<Node<T, M>>>, Vec<Meta<Warning, M>>) {
		(self.nodes, self.warnings)
	}

	#[inline(always)]
	pub fn identify_all<G: id::Generator<T>>(&mut self, mut generator: G) {
		for node in &mut self.nodes {
			node.identify_all(&mut generator)
		}
	}
}

impl<T: Id, M> IntoIterator for FlattenedDocument<T, M> {
	type IntoIter = std::vec::IntoIter<Indexed<Node<T, M>>>;
	type Item = Indexed<Node<T, M>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.nodes.into_iter()
	}
}

impl<'a, T: Id, M> IntoIterator for &'a FlattenedDocument<T, M> {
	type IntoIter = std::slice::Iter<'a, Indexed<Node<T, M>>>;
	type Item = &'a Indexed<Node<T, M>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, T: Id, M> IntoIterator for &'a mut FlattenedDocument<T, M> {
	type IntoIter = std::slice::IterMut<'a, Indexed<Node<T, M>>>;
	type Item = &'a mut Indexed<Node<T, M>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

// impl<F, J: JsonHash + JsonClone, K: JsonFrom<J>, T: Id> AsJson<J, K>
// 	for FlattenedDocument<M>
// {
// 	fn as_json_with(
// 		&self,
// 		meta: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
// 	) -> K {
// 		self.nodes.as_json_with(meta)
// 	}
// }
