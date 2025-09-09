use crate::{Id, ValidId};
use rdf_types::{BlankId, BlankIdBuf, Generator};
use std::collections::HashMap;

#[derive(Default)]
struct Flattener<G> {
	generator: G,
	map: HashMap<BlankIdBuf, ValidId>,
}

impl<G> Flattener<G> {
	pub fn new(generator: G) -> Self {
		Self::default()
	}
}

impl<G: Generator> Flattener<G> {
	pub fn assign(&mut self, blank_id: &BlankId) -> ValidId {
		match self.map.get(blank_id) {
			Some(id) => id,
			None => {
				let id = self.generator.next_term();
			}
		}
	}

	pub fn assign_node_id(&mut self, r: Option<&Id>) -> Id {
		match r {
			Some(Id::Valid(ValidId::Blank(id))) => self.assign(id.clone()).into(),
			Some(r) => r.clone(),
			None => self.next().into(),
		}
	}

	#[allow(clippy::should_implement_trait)]
	pub fn next(&mut self) -> ValidId {
		self.generator.next(self.vocabulary)
	}
}
