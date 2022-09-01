use crate::{id, Reference, ValidReference};
use locspan::Meta;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

pub struct Environment<'n, T, B, M, N, G> {
	id: PhantomData<T>,
	vocabulary: &'n mut N,
	generator: G,
	map: HashMap<B, Meta<ValidReference<T, B>, M>>,
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
	pub fn assign(&mut self, blank_id: B) -> Meta<ValidReference<T, B>, M> {
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
		r: Option<&Meta<Reference<T, B>, M>>,
	) -> Meta<Reference<T, B>, M> {
		match r {
			Some(Meta(Reference::Valid(ValidReference::Blank(id)), _)) => {
				self.assign(id.clone()).cast()
			}
			Some(r) => r.clone(),
			None => self.next().cast(),
		}
	}

	#[allow(clippy::should_implement_trait)]
	pub fn next(&mut self) -> Meta<ValidReference<T, B>, M> {
		self.generator.next(self.vocabulary)
	}
}
