use crate::{
	compaction,
	context::{self, Loader},
	util::{AsJson, JsonFrom},
	ContextMut, Error, Id, BlankId, Indexed, Loc, Object, Warning,
};
use generic_json::{JsonClone, JsonHash};
use std::collections::{HashSet, HashMap};

/// Result of the document expansion algorithm.
///
/// It is just an alias for a set of (indexed) objects.
pub struct ExpandedDocument<J: JsonHash, T: Id> {
	objects: HashSet<Indexed<Object<J, T>>>,
	warnings: Vec<Loc<Warning, J::MetaData>>,
}

impl<J: JsonHash, T: Id> ExpandedDocument<J, T> {
	#[inline(always)]
	pub fn new(
		objects: HashSet<Indexed<Object<J, T>>>,
		warnings: Vec<Loc<Warning, J::MetaData>>,
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

	/// Find a blank node identifiers substitution that maps `self` to `other`.
	/// 
	/// If such substitution exists, then the two documents are structurally and semantically equivalents.
	pub fn blank_node_substitution(&self, other: &Self) -> Option<HashMap<BlankId, BlankId>> {
		if self.objects.len() == other.objects.len() {
			crate::util::Pairings::new(
				self.objects.iter(),
				other.objects.iter(),
				HashMap::<BlankId, BlankId>::new(),
				|substitution, a, b| {
					panic!("TODO")
				}
			).next()
		} else {
			None
		}
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
	pub fn into_parts(
		self,
	) -> (
		HashSet<Indexed<Object<J, T>>>,
		Vec<Loc<Warning, J::MetaData>>,
	) {
		(self.objects, self.warnings)
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
		let mut compacted: K = self.objects.compact_full(
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

impl<J: JsonHash + PartialEq, T: Id + PartialEq> PartialEq for ExpandedDocument<J, T> {
	/// Comparison between two expanded documents.
	/// 
	/// Warnings are not compared.
	fn eq(&self, other: &Self) -> bool {
		self.objects.eq(&other.objects)
	}
}

impl<J: JsonHash + Eq, T: Id + Eq> Eq for ExpandedDocument<J, T> {}

impl<J: JsonHash, T: Id> IntoIterator for ExpandedDocument<J, T> {
	type IntoIter = std::collections::hash_set::IntoIter<Indexed<Object<J, T>>>;
	type Item = Indexed<Object<J, T>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.objects.into_iter()
	}
}

impl<'a, J: JsonHash, T: Id> IntoIterator for &'a ExpandedDocument<J, T> {
	type IntoIter = std::collections::hash_set::Iter<'a, Indexed<Object<J, T>>>;
	type Item = &'a Indexed<Object<J, T>>;

	#[inline(always)]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<J: JsonHash + JsonClone, K: JsonFrom<J>, T: Id> AsJson<J, K> for ExpandedDocument<J, T> {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		self.objects.as_json_with(meta)
	}
}
