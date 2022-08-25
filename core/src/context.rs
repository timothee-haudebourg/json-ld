//! Context processing algorithm and related types.
mod definition;
pub mod inverse;

use crate::{Direction, LenientLanguageTag, LenientLanguageTagBuf, Term};
use once_cell::sync::OnceCell;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

pub use json_ld_syntax::context::{definition::Key, term_definition::Nest};

pub use definition::*;
pub use inverse::InverseContext;

/// JSON-LD context.
pub struct Context<T, B, L> {
	original_base_url: Option<T>,
	base_iri: Option<T>,
	vocabulary: Option<Term<T, B>>,
	default_language: Option<LenientLanguageTagBuf>,
	default_base_direction: Option<Direction>,
	previous_context: Option<Box<Self>>,
	definitions: HashMap<Key, TermDefinition<T, B, L>>,
	inverse: OnceCell<InverseContext<T, B>>,
}

impl<T, B, L> Default for Context<T, B, L> {
	fn default() -> Self {
		Self {
			original_base_url: None,
			base_iri: None,
			vocabulary: None,
			default_language: None,
			default_base_direction: None,
			previous_context: None,
			definitions: HashMap::new(),
			inverse: OnceCell::default(),
		}
	}
}

pub type DefinitionEntryRef<'a, T, B, L> = (&'a Key, &'a TermDefinition<T, B, L>);

impl<T, B, L> Context<T, B, L> {
	pub fn new(base_iri: Option<T>) -> Self
	where
		T: Clone,
	{
		Self {
			original_base_url: base_iri.clone(),
			base_iri,
			vocabulary: None,
			default_language: None,
			default_base_direction: None,
			previous_context: None,
			definitions: HashMap::new(),
			inverse: OnceCell::default(),
		}
	}

	pub fn get<Q: ?Sized>(&self, term: &Q) -> Option<&TermDefinition<T, B, L>>
	where
		Key: Borrow<Q>,
		Q: Hash + Eq,
	{
		self.definitions.get(term)
	}

	pub fn original_base_url(&self) -> Option<&T> {
		self.original_base_url.as_ref()
	}

	pub fn base_iri(&self) -> Option<&T> {
		self.base_iri.as_ref()
	}

	pub fn vocabulary(&self) -> Option<&Term<T, B>> {
		match &self.vocabulary {
			Some(v) => Some(v),
			None => None,
		}
	}

	pub fn default_language(&self) -> Option<LenientLanguageTag> {
		self.default_language.as_ref().map(|tag| tag.as_ref())
	}

	pub fn default_base_direction(&self) -> Option<Direction> {
		self.default_base_direction
	}

	pub fn previous_context(&self) -> Option<&Self> {
		match &self.previous_context {
			Some(c) => Some(c),
			None => None,
		}
	}

	pub fn len(&self) -> usize {
		self.definitions.len()
	}

	pub fn is_empty(&self) -> bool {
		self.definitions.is_empty()
	}

	pub fn definitions<'a>(
		&'a self,
	) -> Box<dyn 'a + Iterator<Item = DefinitionEntryRef<'a, T, B, L>>> {
		Box::new(self.definitions.iter())
	}

	/// Checks if the context has a protected definition.
	pub fn has_protected_items(&self) -> bool {
		for (_, definition) in self.definitions() {
			if definition.protected {
				return true;
			}
		}

		false
	}

	pub fn inverse(&self) -> &InverseContext<T, B>
	where
		T: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
	{
		self.inverse.get_or_init(|| self.into())
	}

	pub fn set(
		&mut self,
		key: Key,
		definition: Option<TermDefinition<T, B, L>>,
	) -> Option<TermDefinition<T, B, L>> {
		self.inverse.take();
		match definition {
			Some(def) => self.definitions.insert(key, def),
			None => self.definitions.remove(&key),
		}
	}

	pub fn set_base_iri(&mut self, iri: Option<T>) {
		self.inverse.take();
		self.base_iri = iri
	}

	pub fn set_vocabulary(&mut self, vocab: Option<Term<T, B>>) {
		self.inverse.take();
		self.vocabulary = vocab;
	}

	pub fn set_default_language(&mut self, lang: Option<LenientLanguageTagBuf>) {
		self.inverse.take();
		self.default_language = lang;
	}

	pub fn set_default_base_direction(&mut self, dir: Option<Direction>) {
		self.inverse.take();
		self.default_base_direction = dir;
	}

	pub fn set_previous_context(&mut self, previous: Self) {
		self.inverse.take();
		self.previous_context = Some(Box::new(previous))
	}
}

impl<T: Clone, B: Clone, L: Clone> Clone for Context<T, B, L> {
	fn clone(&self) -> Self {
		Self {
			original_base_url: self.original_base_url.clone(),
			base_iri: self.base_iri.clone(),
			vocabulary: self.vocabulary.clone(),
			default_language: self.default_language.clone(),
			default_base_direction: self.default_base_direction,
			previous_context: self.previous_context.clone(),
			definitions: self.definitions.clone(),
			inverse: OnceCell::default(),
		}
	}
}

impl<T: PartialEq, B: PartialEq, L: PartialEq> PartialEq for Context<T, B, L> {
	fn eq(&self, other: &Self) -> bool {
		self.original_base_url == other.original_base_url
			&& self.base_iri == other.base_iri
			&& self.vocabulary == other.vocabulary
			&& self.default_language == other.default_language
			&& self.default_base_direction == other.default_base_direction
			&& self.previous_context == other.previous_context
	}
}
