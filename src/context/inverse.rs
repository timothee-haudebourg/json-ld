use super::Context;
use crate::{
	lang::{LenientLanguageTag, LenientLanguageTagBuf},
	syntax::{Container, Term, Type},
	Direction, Id, Nullable,
};
use mown::Mown;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use std::{cmp::Ordering, collections::HashMap, fmt};

/// Context that can be inverted.
///
/// This type keeps an inversion of the underlying context which is computed
/// when [`inverse`](Inversible::inverse) is called and reset when the context is mutably accessed.
pub struct Inversible<T: Id, C> {
	/// Underlying context.
	context: C,

	/// Inverse context.
	inverse: Arc<OnceCell<InverseContext<T>>>,
}

impl<T: Id, C: Clone> Clone for Inversible<T, C> {
	#[inline]
	fn clone(&self) -> Self {
		Inversible {
			context: self.context.clone(),
			inverse: self.inverse.clone(),
		}
	}
}

impl<T: Id, C> std::ops::Deref for Inversible<T, C> {
	type Target = C;

	#[inline]
	fn deref(&self) -> &C {
		&self.context
	}
}

impl<T: Id, C> std::ops::DerefMut for Inversible<T, C> {
	#[inline]
	fn deref_mut(&mut self) -> &mut C {
		self.inverse = Arc::new(OnceCell::new());
		&mut self.context
	}
}

impl<T: Id, C> Inversible<T, C> {
	pub fn new(context: C) -> Inversible<T, C> {
		Inversible {
			context,
			inverse: Arc::new(OnceCell::new()),
		}
	}

	pub fn inverse(&self) -> &InverseContext<T>
	where
		C: std::ops::Deref,
		C::Target: Context<T>,
	{
		self.inverse
			.get_or_init(|| InverseContext::from(&*self.context))
	}

	pub fn into_owned<'a>(self) -> Inversible<T, Mown<'a, C>> {
		Inversible {
			context: Mown::Owned(self.context),
			inverse: self.inverse,
		}
	}
}

impl<'a, T: Id, C> Inversible<T, &'a C> {
	pub fn into_borrowed(self) -> Inversible<T, Mown<'a, C>> {
		Inversible {
			context: Mown::Borrowed(self.context),
			inverse: self.inverse,
		}
	}
}

impl<'a, T: Id, C> Inversible<T, Mown<'a, C>> {
	pub fn as_ref(&self) -> Inversible<T, &C> {
		Inversible {
			context: self.context.as_ref(),
			inverse: self.inverse.clone(),
		}
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum TypeSelection<T: Id> {
	Reverse,
	Any,
	Type(Type<T>),
}

impl<T: Id> fmt::Debug for TypeSelection<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			TypeSelection::Reverse => write!(f, "Reverse"),
			TypeSelection::Any => write!(f, "Any"),
			TypeSelection::Type(ty) => write!(f, "Type({})", ty.as_str()),
		}
	}
}

struct InverseType<T: Id> {
	reverse: Option<String>,
	any: Option<String>,
	map: HashMap<Type<T>, String>,
}

impl<T: Id> InverseType<T> {
	fn select(&self, selection: TypeSelection<T>) -> Option<&str> {
		match selection {
			TypeSelection::Reverse => self.reverse.as_ref(),
			TypeSelection::Any => self.any.as_ref(),
			TypeSelection::Type(ty) => self.map.get(&ty),
		}
		.map(|v| v.as_str())
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

type LangDir = Nullable<(Option<LenientLanguageTagBuf>, Option<Direction>)>;

struct InverseLang {
	any: Option<String>,
	map: HashMap<LangDir, String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LangSelection<'a> {
	Any,
	Lang(Nullable<(Option<LenientLanguageTag<'a>>, Option<Direction>)>),
}

impl InverseLang {
	fn select(&self, selection: LangSelection) -> Option<&str> {
		match selection {
			LangSelection::Any => self.any.as_ref(),
			LangSelection::Lang(lang_dir) => {
				let lang_dir = lang_dir.map(|(l, d)| (l.map(|l| l.cloned()), d));
				self.map.get(&lang_dir)
			}
		}
		.map(|v| v.as_str())
	}

	fn set_any(&mut self, term: &str) {
		if self.any.is_none() {
			self.any = Some(term.to_string())
		}
	}

	fn set_none(&mut self, term: &str) {
		self.set(Nullable::Some((None, None)), term)
	}

	fn set(
		&mut self,
		lang_dir: Nullable<(Option<LenientLanguageTag>, Option<Direction>)>,
		term: &str,
	) {
		let lang_dir = lang_dir.map(|(l, d)| (l.map(|l| l.cloned()), d));
		self.map.entry(lang_dir).or_insert_with(|| term.to_string());
	}
}

struct InverseContainer<T: Id> {
	language: InverseLang,
	typ: InverseType<T>,
	any: Any,
}

struct Any {
	none: String,
}

impl<T: Id> InverseContainer<T> {
	pub fn new(term: &str) -> InverseContainer<T> {
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
			any: Any {
				none: term.to_string(),
			},
		}
	}
}

pub struct InverseDefinition<T: Id> {
	map: HashMap<Container, InverseContainer<T>>,
}

impl<T: Id> InverseDefinition<T> {
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

	pub fn select(&self, containers: &[Container], selection: &Selection<T>) -> Option<&str> {
		for container in containers {
			if let Some(type_lang_map) = self.get(container) {
				match selection {
					Selection::Any => return Some(type_lang_map.any.none.as_str()),
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

pub struct InverseContext<T: Id> {
	map: HashMap<Term<T>, InverseDefinition<T>>,
}

pub enum Selection<'a, T: Id> {
	Any,
	Type(Vec<TypeSelection<T>>),
	Lang(Vec<LangSelection<'a>>),
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
			map: HashMap::new(),
		}
	}

	pub fn contains(&self, term: &Term<T>) -> bool {
		self.map.contains_key(term)
	}

	pub fn insert(&mut self, term: Term<T>, value: InverseDefinition<T>) {
		self.map.insert(term, value);
	}

	pub fn get(&self, term: &Term<T>) -> Option<&InverseDefinition<T>> {
		self.map.get(term)
	}

	pub fn get_mut(&mut self, term: &Term<T>) -> Option<&mut InverseDefinition<T>> {
		self.map.get_mut(term)
	}

	fn reference_mut<F: FnOnce() -> InverseDefinition<T>>(
		&mut self,
		term: &Term<T>,
		insert: F,
	) -> &mut InverseDefinition<T> {
		if !self.contains(term) {
			self.insert(term.clone(), insert());
		}
		self.map.get_mut(term).unwrap()
	}

	pub fn select(
		&self,
		var: &Term<T>,
		containers: &[Container],
		selection: &Selection<T>,
	) -> Option<&str> {
		match self.get(var) {
			Some(container_map) => container_map.select(containers, selection),
			None => None,
		}
	}
}

impl<T: Id> Default for InverseContext<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<'a, T: Id, C: Context<T>> From<&'a C> for InverseContext<T> {
	fn from(context: &'a C) -> InverseContext<T> {
		let mut result = InverseContext::new();

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
				let container_map = result.reference_mut(var, InverseDefinition::new);
				let type_lang_map =
					container_map.reference_mut(container, || InverseContainer::new(term));

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
										(Nullable::Some(language), Nullable::Some(direction)) => {
											lang_map.set(
												Nullable::Some((
													Some(language.as_ref()),
													Some(*direction),
												)),
												term,
											)
										}
										(Nullable::Some(language), Nullable::Null) => lang_map.set(
											Nullable::Some((Some(language.as_ref()), None)),
											term,
										),
										(Nullable::Null, Nullable::Some(direction)) => lang_map
											.set(Nullable::Some((None, Some(*direction))), term),
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
											Nullable::Some((Some(language.as_ref()), None)),
											term,
										),
										Nullable::Null => lang_map.set(Nullable::Null, term),
									}
								}
								(None, Some(direction)) => {
									// Otherwise, if term definition has a direction mapping (might
									// be null):
									match direction {
										Nullable::Some(direction) => lang_map
											.set(Nullable::Some((None, Some(*direction))), term),
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

		result
	}
}
