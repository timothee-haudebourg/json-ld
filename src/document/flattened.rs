use crate::{
	compaction,
	context::{self, Loader},
	util::{AsJson, JsonFrom},
	ContextMut, Error, Id, Indexed, Loc, Node, Warning,
};
use generic_json::{JsonClone, JsonHash};

/// Result of the document flattening algorithm.
///
/// It is just an alias for a set of (indexed) nodes.
pub struct FlattenedDocument<J: JsonHash, T: Id> {
	nodes: Vec<Indexed<Node<J, T>>>,
	warnings: Vec<Loc<Warning, J::MetaData>>,
}

impl<J: JsonHash, T: Id> FlattenedDocument<J, T> {
	#[inline(always)]
	pub fn new(nodes: Vec<Indexed<Node<J, T>>>, warnings: Vec<Loc<Warning, J::MetaData>>) -> Self {
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
	pub fn warnings(&self) -> &[Loc<Warning, J::MetaData>] {
		&self.warnings
	}

	#[inline(always)]
	pub fn into_warnings(self) -> Vec<Loc<Warning, J::MetaData>> {
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
	pub fn into_parts(self) -> (Vec<Indexed<Node<J, T>>>, Vec<Loc<Warning, J::MetaData>>) {
		(self.nodes, self.warnings)
	}

	pub async fn compact<'a, K: JsonFrom<J>, C: ContextMut<T>, L: Loader, M>(
		&'a self,
		context: &'a context::ProcessedOwned<K, context::Inversible<T, C>>,
		loader: &'a mut L,
		options: compaction::Options,
		meta: M
	) -> Result<K, Error>
	where
		K: Clone + JsonFrom<C::LocalContext>,
		J: compaction::JsonSrc,
		T: 'a + Sync + Send,
		C: Sync + Send,
		C::LocalContext: From<L::Output>,
		L: Sync + Send,
		M: 'a + Clone + Send + Sync + Fn(Option<&J::MetaData>) -> K::MetaData,
	{
		use compaction::Compact;
		let mut compacted: K = self.nodes.compact_full(
			context.as_ref(),
			context.as_ref(),
			None,
			loader,
			options,
			meta.clone(),
		).await?;

		use crate::Document;
		compacted.embed_context(
			context,
			options,
			|| meta(None)
		)?;

		Ok(compacted)
	}
}

impl<J: JsonHash, T: Id> IntoIterator for FlattenedDocument<J, T> {
	type IntoIter = std::vec::IntoIter<Indexed<Node<J, T>>>;
	type Item = Indexed<Node<J, T>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.nodes.into_iter()
	}
}

impl<'a, J: JsonHash, T: Id> IntoIterator for &'a FlattenedDocument<J, T> {
	type IntoIter = std::slice::Iter<'a, Indexed<Node<J, T>>>;
	type Item = &'a Indexed<Node<J, T>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<J: JsonHash + JsonClone, K: JsonFrom<J>, T: Id> AsJson<J, K> for FlattenedDocument<J, T> {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		self.nodes.as_json_with(meta)
	}
}
