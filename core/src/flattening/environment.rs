use crate::{id, Id, ValidId};
use locspan::Meta;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

pub struct Environment<'n, T, B, M, N, G> {
	id: PhantomData<T>,
	vocabulary: &'n mut N,
	generator: G,
	map: HashMap<B, Meta<ValidId<T, B>, M>>,
}

impl<'n, T, B, M, N, G> Environment<'n, T, B, M, N, G> {
	pub fn new(vocabulary: &'n mut N, generator: G) -> Self {
		Self {
			id: PhantomData,
			vocabulary,
			generator,
			map: HashMap::new(),
		}
	}
}

impl<'n, T: Clone, B: Clone + Hash + Eq, M: Clone, N, G: id::Generator<T, B, M, N>>
	Environment<'n, T, B, M, N, G>
{
	pub fn assign(&mut self, blank_id: B) -> Meta<ValidId<T, B>, M> {
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
		r: Option<&Meta<Id<T, B>, M>>,
	) -> Meta<Id<T, B>, M> {
		match r {
			Some(Meta(Id::Valid(ValidId::Blank(id)), _)) => {
				self.assign(id.clone()).cast()
			}
			Some(r) => r.clone(),
			None => self.next().cast(),
		}
	}

	#[allow(clippy::should_implement_trait)]
	pub fn next(&mut self) -> Meta<ValidId<T, B>, M> {
		self.generator.next(self.vocabulary)
	}
}
