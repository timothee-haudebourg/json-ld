use crate::{id, Id, Reference, ValidReference};
use generic_json::JsonClone;
use rdf_types::BlankIdBuf;
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct Namespace<T, G> {
	id: PhantomData<T>,
	generator: G,
	map: HashMap<BlankIdBuf, ValidReference<T>>,
}

impl<T, G> Namespace<T, G> {
	pub fn new(generator: G) -> Self {
		Self {
			id: PhantomData,
			generator,
			map: HashMap::new(),
		}
	}
}

impl<T: Id, G: id::Generator<T>> Namespace<T, G> {
	pub fn assign(&mut self, blank_id: BlankIdBuf) -> ValidReference<T> {
		use std::collections::hash_map::Entry;
		match self.map.entry(blank_id) {
			Entry::Occupied(entry) => entry.get().clone(),
			Entry::Vacant(entry) => {
				let id = self.generator.next();
				entry.insert(id.clone());
				id
			}
		}
	}

	pub fn assign_node_id(&mut self, r: Option<&Reference<T>>) -> Reference<T> {
		match r {
			Some(Reference::Blank(id)) => self.assign(id.clone()).into(),
			Some(r) => r.clone(),
			None => self.next().into(),
		}
	}

	#[allow(clippy::should_implement_trait)]
	pub fn next(&mut self) -> ValidReference<T> {
		self.generator.next()
	}
}
