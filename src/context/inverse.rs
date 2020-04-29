use std::cmp::Ordering;
use std::collections::HashMap;
use crate::{
	Id,
	syntax::{
		Term
	}
};
use super::{
	Context
};

pub struct Container {
	language: (),
	typ: (),
	any: ()
}

pub struct InverseDefinition {
	container: Option<Container>
}

impl InverseDefinition {
	fn new() -> InverseDefinition {
		InverseDefinition {
			container: None
		}
	}
}

pub struct InverseContext<T: Id> {
	map: HashMap<Term<T>, InverseDefinition>
}

impl<T: Id> InverseContext<T> {
	pub fn new() -> InverseContext<T> {
		InverseContext {
			map: HashMap::new()
		}
	}

	pub fn contains(&self, term: &Term<T>) -> bool {
		self.map.contains_key(term)
	}

	pub fn insert(&mut self, term: Term<T>, value: InverseDefinition) {
		self.map.insert(term, value);
	}

	pub fn get(&self, term: &Term<T>) -> Option<&InverseDefinition> {
		self.map.get(term)
	}

	pub fn get_mut(&mut self, term: &Term<T>) -> Option<&mut InverseDefinition> {
		self.map.get_mut(term)
	}
}

impl<'a, T: Id, C: Context<T>> From<&'a C> for InverseContext<T> {
	fn from(context: &'a C) -> InverseContext<T> {
		let mut result = InverseContext::new();
		let mut default_language = context.default_language();

		let mut definitions: Vec<_> = context.definitions().collect();
		definitions.sort_by(|(a, _), (b, _)| {
			let ord = a.len().cmp(&b.len());
			if ord == Ordering::Equal {
				a.cmp(b)
			} else {
				ord
			}
		});

		for (term, term_definition) in definitions {
			if let Some(var) = term_definition.value.as_ref() {
				let container = &term_definition.container;

				if !result.contains(var) {
					result.insert(var.clone(), InverseDefinition::new());
				}

				let container_map = result.get_mut(var).unwrap();

				if container_map.container.is_none() {
					// ...
				}
			}
		}

		result
	}
}
