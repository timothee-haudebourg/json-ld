use super::BindingRef;
use super::ContextTerm;
use super::ProcessedContext;
use crate::syntax::Container;
use crate::{Direction, LenientLangTag, LenientLangTagBuf, Nullable, Term, Type};
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeSelection {
	Reverse,
	Any,
	Type(Type),
}

struct InverseType {
	reverse: Option<ContextTerm>,
	any: Option<ContextTerm>,
	map: HashMap<Type, ContextTerm>,
}

impl InverseType {
	fn select(&self, selection: TypeSelection) -> Option<&ContextTerm> {
		match selection {
			TypeSelection::Reverse => self.reverse.as_ref(),
			TypeSelection::Any => self.any.as_ref(),
			TypeSelection::Type(ty) => self.map.get(&ty),
		}
	}

	fn set_any(&mut self, term: &ContextTerm) {
		if self.any.is_none() {
			self.any = Some(term.clone())
		}
	}

	fn set_none(&mut self, term: &ContextTerm) {
		self.set(&Type::None, term)
	}

	fn set(&mut self, ty: &Type, term: &ContextTerm) {
		if !self.map.contains_key(ty) {
			self.map.insert(ty.clone(), term.clone());
		}
	}
}

type LangDir = Nullable<(Option<LenientLangTagBuf>, Option<Direction>)>;

struct InverseLang {
	any: Option<ContextTerm>,
	map: HashMap<LangDir, ContextTerm>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LangSelection<'a> {
	Any,
	Lang(Nullable<(Option<&'a LenientLangTag>, Option<Direction>)>),
}

impl InverseLang {
	fn select(&self, selection: LangSelection) -> Option<&ContextTerm> {
		match selection {
			LangSelection::Any => self.any.as_ref(),
			LangSelection::Lang(lang_dir) => {
				let lang_dir = lang_dir.map(|(l, d)| (l.map(|l| l.to_owned()), d));
				self.map.get(&lang_dir)
			}
		}
	}

	fn set_any(&mut self, term: &ContextTerm) {
		if self.any.is_none() {
			self.any = Some(term.clone())
		}
	}

	fn set_none(&mut self, term: &ContextTerm) {
		self.set(Nullable::Some((None, None)), term)
	}

	fn set(
		&mut self,
		lang_dir: Nullable<(Option<&LenientLangTag>, Option<Direction>)>,
		term: &ContextTerm,
	) {
		let lang_dir = lang_dir.map(|(l, d)| (l.map(|l| l.to_owned()), d));
		self.map.entry(lang_dir).or_insert_with(|| term.clone());
	}
}

struct InverseContainer {
	language: InverseLang,
	typ: InverseType,
	any: Any,
}

struct Any {
	none: ContextTerm,
}

impl InverseContainer {
	pub fn new(term: &ContextTerm) -> InverseContainer {
		InverseContainer {
			language: InverseLang {
				any: None,
				map: HashMap::new(),
			},
			typ: InverseType {
				reverse: None,
				any: None,
				map: HashMap::new(),
			},
			any: Any { none: term.clone() },
		}
	}
}

pub struct InverseDefinition {
	map: HashMap<Container, InverseContainer>,
}

impl InverseDefinition {
	fn new() -> InverseDefinition {
		InverseDefinition {
			map: HashMap::new(),
		}
	}

	fn get(&self, container: &Container) -> Option<&InverseContainer> {
		self.map.get(container)
	}

	fn contains(&self, container: &Container) -> bool {
		self.map.contains_key(container)
	}

	fn reference_mut<F: FnOnce() -> InverseContainer>(
		&mut self,
		container: &Container,
		insert: F,
	) -> &mut InverseContainer {
		if !self.contains(container) {
			self.map.insert(*container, insert());
		}
		self.map.get_mut(container).unwrap()
	}

	pub fn select(&self, containers: &[Container], selection: &Selection) -> Option<&ContextTerm> {
		for container in containers {
			if let Some(type_lang_map) = self.get(container) {
				match selection {
					Selection::Any => return Some(&type_lang_map.any.none),
					Selection::Type(preferred_values) => {
						for item in preferred_values {
							if let Some(term) = type_lang_map.typ.select(item.clone()) {
								return Some(term);
							}
						}
					}
					Selection::Lang(preferred_values) => {
						for item in preferred_values {
							if let Some(term) = type_lang_map.language.select(*item) {
								return Some(term);
							}
						}
					}
				}
			}
		}

		None
	}
}

/// Inverse context.
pub struct InverseContext {
	map: HashMap<Term, InverseDefinition>,
}

#[derive(Debug)]
pub enum Selection<'a> {
	Any,
	Type(Vec<TypeSelection>),
	Lang(Vec<LangSelection<'a>>),
}

impl InverseContext {
	pub fn new() -> Self {
		InverseContext {
			map: HashMap::new(),
		}
	}

	pub fn contains(&self, term: &Term) -> bool {
		self.map.contains_key(term)
	}

	pub fn insert(&mut self, term: Term, value: InverseDefinition) {
		self.map.insert(term, value);
	}

	pub fn get(&self, term: &Term) -> Option<&InverseDefinition> {
		self.map.get(term)
	}

	pub fn get_mut(&mut self, term: &Term) -> Option<&mut InverseDefinition> {
		self.map.get_mut(term)
	}

	fn reference_mut<F: FnOnce() -> InverseDefinition>(
		&mut self,
		term: &Term,
		insert: F,
	) -> &mut InverseDefinition {
		if !self.contains(term) {
			self.insert(term.clone(), insert());
		}
		self.map.get_mut(term).unwrap()
	}

	pub fn select(
		&self,
		var: &Term,
		containers: &[Container],
		selection: &Selection,
	) -> Option<&ContextTerm> {
		match self.get(var) {
			Some(container_map) => container_map.select(containers, selection),
			None => None,
		}
	}
}

impl Default for InverseContext {
	fn default() -> Self {
		Self::new()
	}
}

impl<'a> From<&'a ProcessedContext> for InverseContext {
	fn from(context: &'a ProcessedContext) -> Self {
		let mut result = InverseContext::new();

		let mut definitions: Vec<_> = context.definitions().iter().collect();
		definitions.sort_by(|a, b| {
			let a = a.term().as_str();
			let b = b.term().as_str();
			let ord = a.len().cmp(&b.len());
			if ord == Ordering::Equal {
				a.cmp(b)
			} else {
				ord
			}
		});

		for binding in definitions {
			if let BindingRef::Normal(term, term_definition) = binding {
				if let Some(var) = term_definition.value.as_ref() {
					let container = &term_definition.container;
					let container_map = result.reference_mut(var, InverseDefinition::new);
					let type_lang_map =
						container_map.reference_mut(container, || InverseContainer::new(term));

					let type_map = &mut type_lang_map.typ;
					let lang_map = &mut type_lang_map.language;

					if term_definition.reverse_property {
						// If the term definition indicates that the term represents a reverse property:
						if type_map.reverse.is_none() {
							type_map.reverse = Some(term.clone())
						}
					} else {
						match &term_definition.typ {
							Some(Type::None) => {
								// Otherwise, if term definition has a type mapping which is @none:
								type_map.set_any(term);
								lang_map.set_any(term);
							}
							Some(typ) => {
								// Otherwise, if term definition has a type mapping:
								type_map.set(typ, term)
							}
							None => {
								match (&term_definition.language, &term_definition.direction) {
									(Some(language), Some(direction)) => {
										// Otherwise, if term definition has both a language mapping
										// and a direction mapping:
										match (language, direction) {
											(
												Nullable::Some(language),
												Nullable::Some(direction),
											) => lang_map.set(
												Nullable::Some((
													Some(language.as_lenient_lang_tag_ref()),
													Some(*direction),
												)),
												term,
											),
											(Nullable::Some(language), Nullable::Null) => lang_map
												.set(
													Nullable::Some((
														Some(language.as_lenient_lang_tag_ref()),
														None,
													)),
													term,
												),
											(Nullable::Null, Nullable::Some(direction)) => lang_map
												.set(
													Nullable::Some((None, Some(*direction))),
													term,
												),
											(Nullable::Null, Nullable::Null) => {
												lang_map.set(Nullable::Null, term)
											}
										}
									}
									(Some(language), None) => {
										// Otherwise, if term definition has a language mapping (might
										// be null):
										match language {
											Nullable::Some(language) => lang_map.set(
												Nullable::Some((
													Some(language.as_lenient_lang_tag_ref()),
													None,
												)),
												term,
											),
											Nullable::Null => lang_map.set(Nullable::Null, term),
										}
									}
									(None, Some(direction)) => {
										// Otherwise, if term definition has a direction mapping (might
										// be null):
										match direction {
											Nullable::Some(direction) => lang_map.set(
												Nullable::Some((None, Some(*direction))),
												term,
											),
											Nullable::Null => {
												lang_map.set(Nullable::Some((None, None)), term)
											}
										}
									}
									(None, None) => {
										lang_map.set(
											Nullable::Some((
												context.default_language(),
												context.default_base_direction(),
											)),
											term,
										);
										lang_map.set_none(term);
										type_map.set_none(term);
									}
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
