use crate::{id, Indexed, Node};
use locspan::Meta;

/// Result of the document flattening algorithm.
///
/// It is just an alias for a set of (indexed) nodes.
pub struct FlattenedDocument<T, B, M>(Vec<Meta<Indexed<Node<T, B, M>>, M>>);

impl<T, B, M> FlattenedDocument<T, B, M> {
	#[inline(always)]
	pub fn new(nodes: Vec<Meta<Indexed<Node<T, B, M>>, M>>) -> Self {
		Self(nodes)
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
	pub fn nodes(&self) -> &[Meta<Indexed<Node<T, B, M>>, M>] {
		&self.0
	}

	#[inline(always)]
	pub fn into_nodes(self) -> Vec<Meta<Indexed<Node<T, B, M>>, M>> {
		self.0
	}

	#[inline(always)]
	pub fn iter(&self) -> std::slice::Iter<'_, Meta<Indexed<Node<T, B, M>>, M>> {
		self.0.iter()
	}

	#[inline(always)]
	pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Meta<Indexed<Node<T, B, M>>, M>> {
		self.0.iter_mut()
	}

	#[inline(always)]
	pub fn identify_all_in<N, G: id::Generator<T, B, M, N>>(
		&mut self,
		namespace: &mut N,
		mut generator: G,
	) where
		M: Clone,
	{
		for node in &mut self.0 {
			node.identify_all_in(namespace, &mut generator)
		}
	}

	#[inline(always)]
	pub fn identify_all<G: id::Generator<T, B, M, ()>>(&mut self, generator: G)
	where
		M: Clone,
	{
		self.identify_all_in(&mut (), generator)
	}
}

impl<T, B, M> IntoIterator for FlattenedDocument<T, B, M> {
	type IntoIter = std::vec::IntoIter<Meta<Indexed<Node<T, B, M>>, M>>;
	type Item = Meta<Indexed<Node<T, B, M>>, M>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<'a, T, B, M> IntoIterator for &'a FlattenedDocument<T, B, M> {
	type IntoIter = std::slice::Iter<'a, Meta<Indexed<Node<T, B, M>>, M>>;
	type Item = &'a Meta<Indexed<Node<T, B, M>>, M>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, T, B, M> IntoIterator for &'a mut FlattenedDocument<T, B, M> {
	type IntoIter = std::slice::IterMut<'a, Meta<Indexed<Node<T, B, M>>, M>>;
	type Item = &'a mut Meta<Indexed<Node<T, B, M>>, M>;

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
// 		self.0.as_json_with(meta)
// 	}
// }
