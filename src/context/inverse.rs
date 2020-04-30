use std::cmp::Ordering;
use std::collections::HashMap;
use crate::{
	Id,
	Direction,
	syntax::{
		Term,
		Container,
		Type
	}
};
use super::{
	Context
};

#[derive(Clone)]
pub enum TypeSelection<'a, T: Id> {
	Reverse,
	Any,
	None,
	Type(&'a Type<T>)
}

struct InverseType<T: Id> {
	reverse: Option<String>,
	any: Option<String>,
	none: Option<String>,
	map: HashMap<Type<T>, String>
}

impl<T: Id> InverseType<T> {
	fn select(&self, selection: TypeSelection<T>) -> Option<&str> {
		match selection {
			TypeSelection::Reverse => self.reverse.as_ref(),
			TypeSelection::Any => self.any.as_ref(),
			TypeSelection::None => self.none.as_ref(),
			TypeSelection::Type(t) => self.map.get(t)
		}.map(|v| v.as_str())
	}

	fn set_any(&mut self, term: &str) {
		if self.any.is_none() {
			self.any = Some(term.to_string())
		}
	}

	fn set_none(&mut self, term: &str) {
		if self.none.is_none() {
			self.none = Some(term.to_string())
		}
	}

	fn set(&mut self, ty: &Type<T>, term: &str) {
		if !self.map.contains_key(ty) {
			self.map.insert(ty.clone(), term.to_string());
		}
	}
}

type LangDir = (Option<String>, Option<Direction>);

struct InverseLang {
	any: Option<String>,
	none: Option<String>,
	map: HashMap<LangDir, String>
}

#[derive(Clone, Copy)]
pub enum LanguageSelection<'a> {
	Any,
	None,
	Language(Option<&'a String>, Option<Direction>)
}

impl InverseLang {
	fn select(&self, selection: LanguageSelection) -> Option<&str> {
		match selection {
			LanguageSelection::Any => self.any.as_ref(),
			LanguageSelection::None => self.none.as_ref(),
			LanguageSelection::Language(lang, dir) => {
				let lang_dir = (lang.map(|l| l.clone()), dir);
				self.map.get(&lang_dir)
			}
		}.map(|v| v.as_str())
	}

	fn set_any(&mut self, term: &str) {
		if self.any.is_none() {
			self.any = Some(term.to_string())
		}
	}

	fn set_none(&mut self, term: &str) {
		if self.none.is_none() {
			self.none = Some(term.to_string())
		}
	}

	fn set(&mut self, lang: Option<&String>, dir: Option<&Direction>, term: &str) {
		let lang_dir = (lang.map(|l| l.clone()), dir.map(|d| d.clone()));
		if !self.map.contains_key(&lang_dir) {
			self.map.insert(lang_dir, term.to_string());
		}
	}
}

struct InverseContainer<T: Id> {
	language: InverseLang,
	typ: InverseType<T>,
	any: Any
}

struct Any {
	none: String
}

impl<T: Id> InverseContainer<T> {
	pub fn new(term: &str) -> InverseContainer<T> {
		InverseContainer {
			language: InverseLang {
				any: None,
				none: None,
				map: HashMap::new()
			},
			typ: InverseType {
				reverse: None,
				any: None,
				none: None,
				map: HashMap::new()
			},
			any: Any {
				none: term.to_string()
			}
		}
	}
}

struct InverseDefinition<T: Id> {
	map: HashMap<Container, InverseContainer<T>>
}

impl<T: Id> InverseDefinition<T> {
	fn new() -> InverseDefinition<T> {
		InverseDefinition {
			map: HashMap::new()
		}
	}

	fn get(&self, container: &Container) -> Option<&InverseContainer<T>> {
		self.map.get(container)
	}

	fn contains(&self, container: &Container) -> bool {
		self.map.contains_key(container)
	}

	fn reference_mut<F: FnOnce() -> InverseContainer<T>>(&mut self, container: &Container, insert: F) -> &mut InverseContainer<T> {
		if !self.contains(container) {
			self.map.insert(container.clone(), insert());
		}
		self.map.get_mut(container).unwrap()
	}
}

pub struct InverseContext<T: Id> {
	map: HashMap<Term<T>, InverseDefinition<T>>
}

pub enum Selection<'a, T: Id> {
	Type(&'a [TypeSelection<'a, T>]),
	Language(&'a [LanguageSelection<'a>])
}

impl<T: Id> InverseContext<T> {
	pub fn new() -> InverseContext<T> {
		InverseContext {
			map: HashMap::new()
		}
	}

	fn contains(&self, term: &Term<T>) -> bool {
		self.map.contains_key(term)
	}

	fn insert(&mut self, term: Term<T>, value: InverseDefinition<T>) {
		self.map.insert(term, value);
	}

	fn get(&self, term: &Term<T>) -> Option<&InverseDefinition<T>> {
		self.map.get(term)
	}

	fn get_mut(&mut self, term: &Term<T>) -> Option<&mut InverseDefinition<T>> {
		self.map.get_mut(term)
	}

	fn reference_mut<F: FnOnce() -> InverseDefinition<T>>(&mut self, term: &Term<T>, insert: F) -> &mut InverseDefinition<T> {
		if !self.contains(term) {
			self.map.insert(term.clone(), insert());
		}
		self.map.get_mut(term).unwrap()
	}

	fn select(&self, var: &Term<T>, containers: &[Container], selection: Selection<T>) -> Option<&str> {
		if let Some(container_map) = self.map.get(var) {
			for container in containers {
				if let Some(type_lang_map) = container_map.get(container) {
					match selection {
						Selection::Type(preferred_values) => {
							for item in preferred_values {
								if let Some(term) = type_lang_map.typ.select(item.clone()) {
									return Some(term)
								}
							}
						},
						Selection::Language(preferred_values) => {
							for item in preferred_values {
								if let Some(term) = type_lang_map.language.select(*item) {
									return Some(term)
								}
							}
						}
					}
				}
			}
		}

		None
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
				let container_map = result.reference_mut(var, || InverseDefinition::new());
				let type_lang_map = container_map.reference_mut(container, || InverseContainer::new(term));

				let type_map = &mut type_lang_map.typ;
				let lang_map = &mut type_lang_map.language;

				if term_definition.reverse_property {
					// If the term definition indicates that the term represents a reverse property:
					if type_map.reverse.is_none() {
						type_map.reverse = Some(term.to_string())
					}
				} else {
					match &term_definition.typ {
						Some(Type::None) => {
							// Otherwise, if term definition has a type mapping which is @none:
							type_map.set_any(term);
							lang_map.set_any(term);
						},
						Some(typ) => {
							// Otherwise, if term definition has a type mapping:
							type_map.set(typ, term)
						},
						None => {
							match (&term_definition.language, &term_definition.direction) {
								(Some(language), Some(direction)) => {
									// Otherwise, if term definition has both a language mapping
									// and a direction mapping:
									lang_map.set(language.as_ref(), direction.as_ref(), term)
								},
								(Some(language), None) => {
									// Otherwise, if term definition has a language mapping (might
									// be null):
									lang_map.set(language.as_ref(), None, term)
								},
								(None, Some(direction)) => {
									// Otherwise, if term definition has a direction mapping (might
									// be null):
									lang_map.set(None, direction.as_ref(), term)
								},
								(None, None) => {
									lang_map.set(context.default_language(), context.default_base_direction().as_ref(), term);
									lang_map.set_none(term);
									type_map.set_none(term);
								}
							}
						}
					}
				}
			}
		}

		result
	}
}
