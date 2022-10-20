use crate::{id, Id, MetaValidVocabularyId, MetaVocabularyId, ValidId};
use locspan::Meta;
use rdf_types::Vocabulary;
use std::collections::HashMap;
use std::hash::Hash;

pub struct Environment<'n, M, N: Vocabulary, G> {
	vocabulary: &'n mut N,
	generator: G,
	map: HashMap<N::BlankId, MetaValidVocabularyId<N, M>>,
}

impl<'n, M, N: Vocabulary, G> Environment<'n, M, N, G> {
	pub fn new(vocabulary: &'n mut N, generator: G) -> Self {
		Self {
			vocabulary,
			generator,
			map: HashMap::new(),
		}
	}
}

impl<'n, M: Clone, V: Vocabulary, G: id::Generator<V, M>> Environment<'n, M, V, G>
where
	V::Iri: Clone,
	V::BlankId: Clone + Hash + Eq,
{
	pub fn assign(&mut self, blank_id: V::BlankId) -> Meta<ValidId<V::Iri, V::BlankId>, M> {
		use std::collections::hash_map::Entry;
		match self.map.entry(blank_id) {
			Entry::Occupied(entry) => entry.get().clone(),
			Entry::Vacant(entry) => {
				let id = self.generator.next(self.vocabulary);
				entry.insert(id.clone());
				id
			}
		}
	}

	pub fn assign_node_id(
		&mut self,
		r: Option<&MetaVocabularyId<V, M>>,
	) -> Meta<Id<V::Iri, V::BlankId>, M> {
		match r {
			Some(Meta(Id::Valid(ValidId::Blank(id)), _)) => self.assign(id.clone()).cast(),
			Some(r) => r.clone(),
			None => self.next().cast(),
		}
	}

	#[allow(clippy::should_implement_trait)]
	pub fn next(&mut self) -> Meta<ValidId<V::Iri, V::BlankId>, M> {
		self.generator.next(self.vocabulary)
	}
}
