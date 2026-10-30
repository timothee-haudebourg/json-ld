use langtag::LangTag;
use langtag::LangTagBuf;

use super::JsonLdBindingRef;
use super::JsonLdContainer;
use super::JsonLdContextTerm;
use super::RawJsonLdProcessedContext;
use crate::JsonLdLenient;
use crate::{JsonLdDirection, JsonLdNullable, JsonLdTerm, JsonLdType};
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonLdTypeSelection {
	Reverse,
	Any,
	Type(JsonLdType),
}

struct InverseType {
	reverse: Option<JsonLdContextTerm>,
	any: Option<JsonLdContextTerm>,
	map: HashMap<JsonLdType, JsonLdContextTerm>,
}

impl InverseType {
	fn select(&self, selection: JsonLdTypeSelection) -> Option<&JsonLdContextTerm> {
		match selection {
			JsonLdTypeSelection::Reverse => self.reverse.as_ref(),
			JsonLdTypeSelection::Any => self.any.as_ref(),
			JsonLdTypeSelection::Type(ty) => self.map.get(&ty),
		}
	}

	fn set_any(&mut self, term: &JsonLdContextTerm) {
		if self.any.is_none() {
			self.any = Some(term.clone())
		}
	}

	fn set_none(&mut self, term: &JsonLdContextTerm) {
		self.set(&JsonLdType::None, term)
	}

	fn set(&mut self, ty: &JsonLdType, term: &JsonLdContextTerm) {
		if !self.map.contains_key(ty) {
			self.map.insert(ty.clone(), term.clone());
		}
	}
}

type LangDir = JsonLdNullable<(Option<JsonLdLenient<LangTagBuf>>, Option<JsonLdDirection>)>;

struct InverseLang {
	any: Option<JsonLdContextTerm>,
	map: HashMap<LangDir, JsonLdContextTerm>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JsonLdLangSelection<'a> {
	Any,
	Lang(JsonLdNullable<(Option<JsonLdLenient<&'a LangTag>>, Option<JsonLdDirection>)>),
}

impl InverseLang {
	fn select(&self, selection: JsonLdLangSelection) -> Option<&JsonLdContextTerm> {
		match selection {
			JsonLdLangSelection::Any => self.any.as_ref(),
			JsonLdLangSelection::Lang(lang_dir) => {
				let lang_dir = lang_dir.map(|(l, d)| (l.map(|l| l.into_owned()), d));
				self.map.get(&lang_dir)
			}
		}
	}

	fn set_any(&mut self, term: &JsonLdContextTerm) {
		if self.any.is_none() {
			self.any = Some(term.clone())
		}
	}

	fn set_none(&mut self, term: &JsonLdContextTerm) {
		self.set(JsonLdNullable::Some((None, None)), term)
	}

	fn set(
		&mut self,
		lang_dir: JsonLdNullable<(Option<JsonLdLenient<&LangTag>>, Option<JsonLdDirection>)>,
		term: &JsonLdContextTerm,
	) {
		let lang_dir = lang_dir.map(|(l, d)| (l.map(|l| l.into_owned()), d));
		self.map.entry(lang_dir).or_insert_with(|| term.clone());
	}
}

struct InverseContainer {
	language: InverseLang,
	typ: InverseType,
	any: Any,
}

struct Any {
	none: JsonLdContextTerm,
}

impl InverseContainer {
	pub fn new(term: &JsonLdContextTerm) -> InverseContainer {
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
	map: HashMap<JsonLdContainer, InverseContainer>,
}

impl InverseDefinition {
	fn new() -> InverseDefinition {
		InverseDefinition {
			map: HashMap::new(),
		}
	}

	fn get(&self, container: &JsonLdContainer) -> Option<&InverseContainer> {
		self.map.get(container)
	}

	fn contains(&self, container: &JsonLdContainer) -> bool {
		self.map.contains_key(container)
	}

	fn reference_mut<F: FnOnce() -> InverseContainer>(
		&mut self,
		container: &JsonLdContainer,
		insert: F,
	) -> &mut InverseContainer {
		if !self.contains(container) {
			self.map.insert(*container, insert());
		}
		self.map.get_mut(container).unwrap()
	}

	pub fn select(
		&self,
		containers: &[JsonLdContainer],
		selection: &JsonLdSelection,
	) -> Option<&JsonLdContextTerm> {
		for container in containers {
			if let Some(type_lang_map) = self.get(container) {
				match selection {
					JsonLdSelection::Any => return Some(&type_lang_map.any.none),
					JsonLdSelection::Type(preferred_values) => {
						for item in preferred_values {
							if let Some(term) = type_lang_map.typ.select(item.clone()) {
								return Some(term);
							}
						}
					}
					JsonLdSelection::Lang(preferred_values) => {
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
pub struct InverseJsonLdContext {
	map: HashMap<JsonLdTerm, InverseDefinition>,
}

#[derive(Debug)]
pub enum JsonLdSelection<'a> {
	Any,
	Type(Vec<JsonLdTypeSelection>),
	Lang(Vec<JsonLdLangSelection<'a>>),
}

impl InverseJsonLdContext {
	pub fn new() -> Self {
		InverseJsonLdContext {
			map: HashMap::new(),
		}
	}

	pub fn contains(&self, term: &JsonLdTerm) -> bool {
		self.map.contains_key(term)
	}

	pub fn insert(&mut self, term: JsonLdTerm, value: InverseDefinition) {
		self.map.insert(term, value);
	}

	pub fn get(&self, term: &JsonLdTerm) -> Option<&InverseDefinition> {
		self.map.get(term)
	}

	pub fn get_mut(&mut self, term: &JsonLdTerm) -> Option<&mut InverseDefinition> {
		self.map.get_mut(term)
	}

	fn reference_mut<F: FnOnce() -> InverseDefinition>(
		&mut self,
		term: &JsonLdTerm,
		insert: F,
	) -> &mut InverseDefinition {
		if !self.contains(term) {
			self.insert(term.clone(), insert());
		}
		self.map.get_mut(term).unwrap()
	}

	pub fn select(
		&self,
		var: &JsonLdTerm,
		containers: &[JsonLdContainer],
		selection: &JsonLdSelection,
	) -> Option<&JsonLdContextTerm> {
		match self.get(var) {
			Some(container_map) => container_map.select(containers, selection),
			None => None,
		}
	}
}

impl Default for InverseJsonLdContext {
	fn default() -> Self {
		Self::new()
	}
}

impl<'a> From<&'a RawJsonLdProcessedContext> for InverseJsonLdContext {
	fn from(context: &'a RawJsonLdProcessedContext) -> Self {
		let mut result = InverseJsonLdContext::new();

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
			if let JsonLdBindingRef::Normal(term, term_definition) = binding
				&& let Some(var) = term_definition.value.as_ref()
			{
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
						Some(JsonLdType::None) => {
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
											JsonLdNullable::Some(language),
											JsonLdNullable::Some(direction),
										) => lang_map.set(
											JsonLdNullable::Some((
												Some(language.as_deref()),
												Some(*direction),
											)),
											term,
										),
										(JsonLdNullable::Some(language), JsonLdNullable::Null) => {
											lang_map.set(
												JsonLdNullable::Some((
													Some(language.as_deref()),
													None,
												)),
												term,
											)
										}
										(JsonLdNullable::Null, JsonLdNullable::Some(direction)) => {
											lang_map.set(
												JsonLdNullable::Some((None, Some(*direction))),
												term,
											)
										}
										(JsonLdNullable::Null, JsonLdNullable::Null) => {
											lang_map.set(JsonLdNullable::Null, term)
										}
									}
								}
								(Some(language), None) => {
									// Otherwise, if term definition has a language mapping (might
									// be null):
									match language {
										JsonLdNullable::Some(language) => lang_map.set(
											JsonLdNullable::Some((Some(language.as_deref()), None)),
											term,
										),
										JsonLdNullable::Null => {
											lang_map.set(JsonLdNullable::Null, term)
										}
									}
								}
								(None, Some(direction)) => {
									// Otherwise, if term definition has a direction mapping (might
									// be null):
									match direction {
										JsonLdNullable::Some(direction) => lang_map.set(
											JsonLdNullable::Some((None, Some(*direction))),
											term,
										),
										JsonLdNullable::Null => {
											lang_map.set(JsonLdNullable::Some((None, None)), term)
										}
									}
								}
								(None, None) => {
									lang_map.set(
										JsonLdNullable::Some((
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

		result
	}
}
