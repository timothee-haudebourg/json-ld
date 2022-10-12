//! Context processing algorithm and related types.
mod definition;
pub mod inverse;

use crate::{Direction, LenientLanguageTag, LenientLanguageTagBuf, Term};
use contextual::WithContext;
use locspan::{BorrowStripped, Meta, StrippedPartialEq};
use once_cell::sync::OnceCell;
use rdf_types::Vocabulary;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

pub use json_ld_syntax::context::{definition::Key, term_definition::Nest};

pub use definition::*;
pub use inverse::InverseContext;

/// JSON-LD context.
pub struct Context<T, B, L, M> {
	original_base_url: Option<T>,
	base_iri: Option<T>,
	vocabulary: Option<Term<T, B>>,
	default_language: Option<LenientLanguageTagBuf>,
	default_base_direction: Option<Direction>,
	previous_context: Option<Box<Self>>,
	definitions: HashMap<Key, TermDefinition<T, B, L, M>>,
	inverse: OnceCell<InverseContext<T, B>>,
}

impl<T, B, L, M> Default for Context<T, B, L, M> {
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

pub type DefinitionEntryRef<'a, T, B, L, M> = (&'a Key, &'a TermDefinition<T, B, L, M>);

impl<T, B, L, M> Context<T, B, L, M> {
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

	pub fn get<Q: ?Sized>(&self, term: &Q) -> Option<&TermDefinition<T, B, L, M>>
	where
		Key: Borrow<Q>,
		Q: Hash + Eq,
	{
		self.definitions.get(term)
	}

	pub fn contains_key<Q: ?Sized>(&self, term: &Q) -> bool
	where
		Key: Borrow<Q>,
		Q: Hash + Eq,
	{
		self.definitions.contains_key(term)
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
	) -> Box<dyn 'a + Iterator<Item = DefinitionEntryRef<'a, T, B, L, M>>> {
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
		definition: Option<TermDefinition<T, B, L, M>>,
	) -> Option<TermDefinition<T, B, L, M>> {
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

	pub fn into_syntax_definition(
		self,
		vocabulary: &impl Vocabulary<T, B>,
		meta: M,
	) -> Meta<json_ld_syntax::context::Definition<M>, M>
	where
		L: IntoSyntax<T, B, M>,
		M: Clone,
	{
		use json_ld_syntax::{Entry, Nullable};

		let definition = json_ld_syntax::context::Definition {
			base: self.base_iri.map(|i| {
				Entry::new(
					meta.clone(),
					Meta(
						Nullable::Some(vocabulary.iri(&i).unwrap().into()),
						meta.clone(),
					),
				)
			}),
			import: None,
			language: self
				.default_language
				.map(|l| Entry::new(meta.clone(), Meta(Nullable::Some(l), meta.clone()))),
			direction: self
				.default_base_direction
				.map(|d| Entry::new(meta.clone(), Meta(Nullable::Some(d), meta.clone()))),
			propagate: None,
			protected: None,
			type_: None,
			version: None,
			vocab: self.vocabulary.map(|v| {
				let vocab = match v {
					Term::Null => Nullable::Null,
					Term::Ref(r) => Nullable::Some(r.with(vocabulary).to_string().into()),
					Term::Keyword(_) => panic!("invalid vocab"),
				};

				Entry::new(meta.clone(), Meta(vocab, meta.clone()))
			}),
			bindings: self
				.definitions
				.into_iter()
				.map(|(key, definition)| {
					(
						Meta(key, meta.clone()),
						definition.into_syntax_definition(vocabulary, meta.clone()),
					)
				})
				.collect(),
		};

		Meta(definition, meta)
	}
}

pub trait IntoSyntax<T, B, M> {
	fn into_syntax(
		self,
		vocabulary: &impl Vocabulary<T, B>,
		meta: M,
	) -> json_ld_syntax::context::Value<M>;
}

impl<T, B, M> IntoSyntax<T, B, M> for json_ld_syntax::context::Value<M> {
	fn into_syntax(
		self,
		_namespace: &impl Vocabulary<T, B>,
		_meta: M,
	) -> json_ld_syntax::context::Value<M> {
		self
	}
}

impl<T, B, M: Clone, L: IntoSyntax<T, B, M>> IntoSyntax<T, B, M> for Context<T, B, L, M> {
	fn into_syntax(
		self,
		vocabulary: &impl Vocabulary<T, B>,
		meta: M,
	) -> json_ld_syntax::context::Value<M> {
		let Meta(definition, meta) = self.into_syntax_definition(vocabulary, meta);
		json_ld_syntax::context::Value::One(Meta(
			json_ld_syntax::Context::Definition(definition),
			meta,
		))
	}
}

impl<T: Clone, B: Clone, L: Clone, M: Clone> Clone for Context<T, B, L, M> {
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

impl<T: PartialEq, B: PartialEq, L: PartialEq, M> StrippedPartialEq for Context<T, B, L, M> {
	fn stripped_eq(&self, other: &Self) -> bool {
		self.original_base_url == other.original_base_url
			&& self.base_iri == other.base_iri
			&& self.vocabulary == other.vocabulary
			&& self.default_language == other.default_language
			&& self.default_base_direction == other.default_base_direction
			&& self.previous_context.stripped() == other.previous_context.stripped()
	}
}
