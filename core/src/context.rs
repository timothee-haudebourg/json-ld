//! Context processing algorithm and related types.
mod definition;
pub mod inverse;

use crate::{Direction, LenientLanguageTag, LenientLanguageTagBuf, Term};
use iref::{Iri, IriBuf};
use once_cell::sync::OnceCell;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

pub use json_ld_syntax::context::{Key, Nest};

pub use definition::*;
pub use inverse::InverseContext;

/// JSON-LD context.
pub struct Context<T, L> {
	original_base_url: Option<IriBuf>,
	base_iri: Option<IriBuf>,
	vocabulary: Option<Term<T>>,
	default_language: Option<LenientLanguageTagBuf>,
	default_base_direction: Option<Direction>,
	previous_context: Option<Box<Self>>,
	definitions: HashMap<Key, TermDefinition<T, L>>,
	inverse: OnceCell<InverseContext<T>>,
}

impl<T, L> Default for Context<T, L> {
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

impl<T, L> Context<T, L> {
	pub fn new(base_iri: Option<Iri>) -> Self {
		Self {
			original_base_url: base_iri.map(|iri| iri.into()),
			base_iri: base_iri.map(|iri| iri.into()),
			vocabulary: None,
			default_language: None,
			default_base_direction: None,
			previous_context: None,
			definitions: HashMap::new(),
			inverse: OnceCell::default(),
		}
	}

	pub fn get<Q: ?Sized>(&self, term: &Q) -> Option<&TermDefinition<T, L>>
	where
		Key: Borrow<Q>,
		Q: Hash + Eq,
	{
		self.definitions.get(term)
	}

	pub fn original_base_url(&self) -> Option<Iri> {
		self.original_base_url.as_ref().map(|iri| iri.as_iri())
	}

	pub fn base_iri(&self) -> Option<Iri> {
		self.base_iri.as_ref().map(|iri| iri.as_iri())
	}

	pub fn vocabulary(&self) -> Option<&Term<T>> {
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

	pub fn definitions<'a>(
		&'a self,
	) -> Box<dyn 'a + Iterator<Item = (&'a Key, &'a TermDefinition<T, L>)>> {
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

	pub fn inverse(&self) -> &InverseContext<T>
	where
		T: Clone + Hash + Eq,
	{
		self.inverse.get_or_init(|| self.into())
	}

	pub fn set(
		&mut self,
		key: Key,
		definition: Option<TermDefinition<T, L>>,
	) -> Option<TermDefinition<T, L>> {
		self.inverse.take();
		match definition {
			Some(def) => self.definitions.insert(key, def),
			None => self.definitions.remove(&key),
		}
	}

	pub fn set_base_iri(&mut self, iri: Option<Iri>) {
		self.inverse.take();
		self.base_iri = match iri {
			Some(iri) => {
				let iri_buf: IriBuf = iri.into();
				Some(iri_buf)
			}
			None => None,
		}
	}

	pub fn set_vocabulary(&mut self, vocab: Option<Term<T>>) {
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

impl<T: Clone, L: Clone> Clone for Context<T, L> {
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

impl<T: PartialEq, L: PartialEq> PartialEq for Context<T, L> {
	fn eq(&self, other: &Self) -> bool {
		self.original_base_url == other.original_base_url
			&& self.base_iri == other.base_iri
			&& self.vocabulary == other.vocabulary
			&& self.default_language == other.default_language
			&& self.default_base_direction == other.default_base_direction
			&& self.previous_context == other.previous_context
	}
}
