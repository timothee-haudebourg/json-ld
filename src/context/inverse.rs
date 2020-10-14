use std::{
	cmp::Ordering,
	collections::{HashMap, HashSet},
	fmt
};
use crate::{
	Id,
	Nullable,
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

#[derive(Clone, PartialEq, Eq)]
pub enum TypeSelection<T: Id> {
	Reverse,
	Any,
	Type(Type<T>)
}

impl<T: Id> fmt::Debug for TypeSelection<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			TypeSelection::Reverse => write!(f, "Reverse"),
			TypeSelection::Any => write!(f, "Any"),
			TypeSelection::Type(ty) => write!(f, "Type({})", ty.as_str())
		}
	}
}

struct InverseType<T: Id> {
	reverse: Option<String>,
	any: Option<String>,
	map: HashMap<Type<T>, String>
}

impl<T: Id> InverseType<T> {
	fn select(&self, selection: TypeSelection<T>) -> Option<&str> {
		match selection {
			TypeSelection::Reverse => self.reverse.as_ref(),
			TypeSelection::Any => self.any.as_ref(),
			TypeSelection::Type(ty) => {
				self.map.get(&ty)
			}
		}.map(|v| v.as_str())
	}

	fn set_any(&mut self, term: &str) {
		if self.any.is_none() {
			self.any = Some(term.to_string())
		}
	}

	fn set_none(&mut self, term: &str) {
		self.set(&Type::None, term)
	}

	fn set(&mut self, ty: &Type<T>, term: &str) {
		if !self.map.contains_key(ty) {
			self.map.insert(ty.clone(), term.to_string());
		}
	}
}

type LangDir = (Option<Nullable<String>>, Option<Nullable<Direction>>);

struct InverseLang {
	any: Option<String>,
	map: HashMap<LangDir, String>
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LangSelection<'a> {
	Any,
	Lang(Option<Nullable<&'a String>>, Option<Nullable<Direction>>)
}

impl InverseLang {
	fn select(&self, selection: LangSelection) -> Option<&str> {
		match selection {
			LangSelection::Any => self.any.as_ref(),
			LangSelection::Lang(lang, dir) => {
				let lang_dir = (lang.map(|l| l.cloned()), dir);
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
		self.set(None, None, term)
	}

	fn set(&mut self, lang: Option<Nullable<&String>>, dir: Option<Nullable<&Direction>>, term: &str) {
		let lang_dir = (lang.map(|l| l.cloned()), dir.map(|d| d.cloned()));
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
				map: HashMap::new()
			},
			typ: InverseType {
				reverse: None,
				any: None,
				map: HashMap::new()
			},
			any: Any {
				none: term.to_string()
			}
		}
	}
}

pub struct InverseDefinition<T: Id> {
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
	Any,
	Type(Vec<TypeSelection<T>>),
	Lang(Vec<LangSelection<'a>>)
}

impl<'a, T: Id> fmt::Debug for Selection<'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Selection::Any => write!(f, "Any"),
			Selection::Type(s) => write!(f, "Type({:?})", s),
			Selection::Lang(s) => write!(f, "Lang({:?})", s),
		}
	}
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
		println!("INSERT {}", term.as_str());
		self.map.insert(term, value);
	}

	pub fn get(&self, term: &Term<T>) -> Option<&InverseDefinition<T>> {
		self.map.get(term)
	}

	fn get_mut(&mut self, term: &Term<T>) -> Option<&mut InverseDefinition<T>> {
		self.map.get_mut(term)
	}

	fn reference_mut<F: FnOnce() -> InverseDefinition<T>>(&mut self, term: &Term<T>, insert: F) -> &mut InverseDefinition<T> {
		if !self.contains(term) {
			self.insert(term.clone(), insert());
		}
		self.map.get_mut(term).unwrap()
	}

	pub fn select(&self, var: &Term<T>, containers: &[Container], selection: &Selection<T>) -> Option<&str> {
		if let Some(container_map) = self.map.get(var) {
			for container in containers {
				if let Some(type_lang_map) = container_map.get(container) {
					match selection {
						Selection::Any => {
							return Some(type_lang_map.any.none.as_str())
						},
						Selection::Type(preferred_values) => {
							for item in preferred_values {
								if let Some(term) = type_lang_map.typ.select(item.clone()) {
									return Some(term)
								}
							}
						},
						Selection::Lang(preferred_values) => {
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

		println!("INVERT");
		for (term, term_definition) in definitions {
			println!("DEF {}", term.as_str());
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
									lang_map.set(Some(language.as_ref()), Some(direction.as_ref()), term)
								},
								(Some(language), None) => {
									// Otherwise, if term definition has a language mapping (might
									// be null):
									lang_map.set(Some(language.as_ref()), None, term)
								},
								(None, Some(direction)) => {
									// Otherwise, if term definition has a direction mapping (might
									// be null):
									lang_map.set(None, Some(direction.as_ref()), term)
								},
								(None, None) => {
									lang_map.set(context.default_language().map(|l| Nullable::Some(l)), context.default_base_direction().as_ref().map(|d| Nullable::Some(d)), term);
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
