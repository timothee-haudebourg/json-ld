use crate::{
	id,
	Id, Indexed, Loc, Node, Warning,
};
use locspan::Span;

/// Result of the document flattening algorithm.
///
/// It is just an alias for a set of (indexed) nodes.
pub struct FlattenedDocument<T: Id, S, P=Span> {
	nodes: Vec<Indexed<Node<T>>>,
	warnings: Vec<Loc<Warning, S, P>>,
}

impl<T: Id, S, P> FlattenedDocument<T, S, P> {
	#[inline(always)]
	pub fn new(
		nodes: Vec<Indexed<Node<T>>>,
		warnings: Vec<Loc<Warning, S, P>>,
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
	pub fn warnings(&self) -> &[Loc<Warning, S, P>] {
		&self.warnings
	}

	#[inline(always)]
	pub fn into_warnings(self) -> Vec<Loc<Warning, S, P>> {
		self.warnings
	}

	#[inline(always)]
	pub fn nodes(&self) -> &[Indexed<Node<T>>] {
		&self.nodes
	}

	#[inline(always)]
	pub fn into_nodes(self) -> Vec<Indexed<Node<T>>> {
		self.nodes
	}

	#[inline(always)]
	pub fn iter(&self) -> std::slice::Iter<'_, Indexed<Node<T>>> {
		self.nodes.iter()
	}

	#[inline(always)]
	pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Indexed<Node<T>>> {
		self.nodes.iter_mut()
	}

	#[inline(always)]
	#[allow(clippy::type_complexity)]
	pub fn into_parts(self) -> (Vec<Indexed<Node<T>>>, Vec<Loc<Warning, S, P>>) {
		(self.nodes, self.warnings)
	}

	#[inline(always)]
	pub fn identify_all<G: id::Generator<T>>(&mut self, mut generator: G) {
		for node in &mut self.nodes {
			node.identify_all(&mut generator)
		}
	}
}

impl<T: Id, S, P> IntoIterator for FlattenedDocument<T, S, P> {
	type IntoIter = std::vec::IntoIter<Indexed<Node<T>>>;
	type Item = Indexed<Node<T>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.nodes.into_iter()
	}
}

impl<'a, T: Id, S, P> IntoIterator for &'a FlattenedDocument<T, S, P> {
	type IntoIter = std::slice::Iter<'a, Indexed<Node<T>>>;
	type Item = &'a Indexed<Node<T>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, T: Id, S, P> IntoIterator for &'a mut FlattenedDocument<T, S, P> {
	type IntoIter = std::slice::IterMut<'a, Indexed<Node<T>>>;
	type Item = &'a mut Indexed<Node<T>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

// impl<F, J: JsonHash + JsonClone, K: JsonFrom<J>, T: Id> AsJson<J, K>
// 	for FlattenedDocument<S, P>
// {
// 	fn as_json_with(
// 		&self,
// 		meta: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
// 	) -> K {
// 		self.nodes.as_json_with(meta)
// 	}
// }
