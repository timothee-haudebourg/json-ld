use iref::IriBuf;

use super::BindingRef;
use super::Context;
use super::Key;
use crate::{Container, Direction, LenientLangTag, LenientLangTagBuf, Nullable, Term, Type};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

#[derive(Clone, PartialEq, Eq)]
pub enum TypeSelection<T = IriBuf> {
	Reverse,
	Any,
	Type(Type<T>),
}

impl<T: fmt::Debug> fmt::Debug for TypeSelection<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			TypeSelection::Reverse => write!(f, "Reverse"),
			TypeSelection::Any => write!(f, "Any"),
			TypeSelection::Type(ty) => write!(f, "Type({ty:?})"),
		}
	}
}

struct InverseType<T> {
	reverse: Option<Key>,
	any: Option<Key>,
	map: HashMap<Type<T>, Key>,
}

impl<T> InverseType<T> {
	fn select(&self, selection: TypeSelection<T>) -> Option<&Key>
	where
		T: Hash + Eq,
	{
		match selection {
			TypeSelection::Reverse => self.reverse.as_ref(),
			TypeSelection::Any => self.any.as_ref(),
			TypeSelection::Type(ty) => self.map.get(&ty),
		}
	}

	fn set_any(&mut self, term: &Key) {
		if self.any.is_none() {
			self.any = Some(term.clone())
		}
	}

	fn set_none(&mut self, term: &Key)
	where
		T: Clone + Hash + Eq,
	{
		self.set(&Type::None, term)
	}

	fn set(&mut self, ty: &Type<T>, term: &Key)
	where
		T: Clone + Hash + Eq,
	{
		if !self.map.contains_key(ty) {
			self.map.insert(ty.clone(), term.clone());
		}
	}
}

type LangDir = Nullable<(Option<LenientLangTagBuf>, Option<Direction>)>;

struct InverseLang {
	any: Option<Key>,
	map: HashMap<LangDir, Key>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LangSelection<'a> {
	Any,
	Lang(Nullable<(Option<&'a LenientLangTag>, Option<Direction>)>),
}

impl InverseLang {
	fn select(&self, selection: LangSelection) -> Option<&Key> {
		match selection {
			LangSelection::Any => self.any.as_ref(),
			LangSelection::Lang(lang_dir) => {
				let lang_dir = lang_dir.map(|(l, d)| (l.map(|l| l.to_owned()), d));
				self.map.get(&lang_dir)
			}
		}
	}

	fn set_any(&mut self, term: &Key) {
		if self.any.is_none() {
			self.any = Some(term.clone())
		}
	}

	fn set_none(&mut self, term: &Key) {
		self.set(Nullable::Some((None, None)), term)
	}

	fn set(
		&mut self,
		lang_dir: Nullable<(Option<&LenientLangTag>, Option<Direction>)>,
		term: &Key,
	) {
		let lang_dir = lang_dir.map(|(l, d)| (l.map(|l| l.to_owned()), d));
		self.map.entry(lang_dir).or_insert_with(|| term.clone());
	}
}

struct InverseContainer<T> {
	language: InverseLang,
	typ: InverseType<T>,
	any: Any,
}

struct Any {
	none: Key,
}

impl<T> InverseContainer<T> {
	pub fn new(term: &Key) -> InverseContainer<T> {
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

pub struct InverseDefinition<T> {
	map: HashMap<Container, InverseContainer<T>>,
}

impl<T> InverseDefinition<T> {
	fn new() -> InverseDefinition<T> {
		InverseDefinition {
			map: HashMap::new(),
		}
	}

	fn get(&self, container: &Container) -> Option<&InverseContainer<T>> {
		self.map.get(container)
	}

	fn contains(&self, container: &Container) -> bool {
		self.map.contains_key(container)
	}

	fn reference_mut<F: FnOnce() -> InverseContainer<T>>(
		&mut self,
		container: &Container,
		insert: F,
	) -> &mut InverseContainer<T> {
		if !self.contains(container) {
			self.map.insert(*container, insert());
		}
		self.map.get_mut(container).unwrap()
	}

	pub fn select(&self, containers: &[Container], selection: &Selection<T>) -> Option<&Key>
	where
		T: Clone + Hash + Eq,
	{
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
pub struct InverseContext<T, B> {
	map: HashMap<Term<T, B>, InverseDefinition<T>>,
}

pub enum Selection<'a, T> {
	Any,
	Type(Vec<TypeSelection<T>>),
	Lang(Vec<LangSelection<'a>>),
}

impl<'a, T: fmt::Debug> fmt::Debug for Selection<'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Selection::Any => write!(f, "Any"),
			Selection::Type(s) => write!(f, "Type({s:?})"),
			Selection::Lang(s) => write!(f, "Lang({s:?})"),
		}
	}
}

impl<T, B> InverseContext<T, B> {
	pub fn new() -> Self {
		InverseContext {
			map: HashMap::new(),
		}
	}
}

impl<T: Hash + Eq, B: Hash + Eq> InverseContext<T, B> {
	pub fn contains(&self, term: &Term<T, B>) -> bool {
		self.map.contains_key(term)
	}

	pub fn insert(&mut self, term: Term<T, B>, value: InverseDefinition<T>) {
		self.map.insert(term, value);
	}

	pub fn get(&self, term: &Term<T, B>) -> Option<&InverseDefinition<T>> {
		self.map.get(term)
	}

	pub fn get_mut(&mut self, term: &Term<T, B>) -> Option<&mut InverseDefinition<T>> {
		self.map.get_mut(term)
	}

	fn reference_mut<F: FnOnce() -> InverseDefinition<T>>(
		&mut self,
		term: &Term<T, B>,
		insert: F,
	) -> &mut InverseDefinition<T>
	where
		T: Clone,
		B: Clone,
	{
		if !self.contains(term) {
			self.insert(term.clone(), insert());
		}
		self.map.get_mut(term).unwrap()
	}

	pub fn select(
		&self,
		var: &Term<T, B>,
		containers: &[Container],
		selection: &Selection<T>,
	) -> Option<&Key>
	where
		T: Clone,
	{
		match self.get(var) {
			Some(container_map) => container_map.select(containers, selection),
			None => None,
		}
	}
}

impl<T, B> Default for InverseContext<T, B> {
	fn default() -> Self {
		Self::new()
	}
}

impl<'a, T: Clone + Hash + Eq, B: Clone + Hash + Eq> From<&'a Context<T, B>>
	for InverseContext<T, B>
{
	fn from(context: &'a Context<T, B>) -> Self {
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
