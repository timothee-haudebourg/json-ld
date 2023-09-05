//! Context processing algorithm and related types.
mod definition;
pub mod inverse;

use crate::{Direction, LenientLanguageTag, LenientLanguageTagBuf, Term};
use contextual::WithContext;
use json_ld_syntax::KeywordType;
use locspan::{BorrowStripped, Meta, StrippedPartialEq};
use once_cell::sync::OnceCell;
use rdf_types::Vocabulary;
use std::borrow::Borrow;
use std::hash::Hash;

pub use json_ld_syntax::context::{
	definition::{Key, KeyOrType, Type},
	term_definition::Nest,
};

pub use definition::*;
pub use inverse::InverseContext;

/// JSON-LD context.
pub struct Context<T, B, M> {
	original_base_url: Option<T>,
	base_iri: Option<T>,
	vocabulary: Option<Term<T, B>>,
	default_language: Option<LenientLanguageTagBuf>,
	default_base_direction: Option<Direction>,
	previous_context: Option<Box<Self>>,
	definitions: Definitions<T, B, M>,
	inverse: OnceCell<InverseContext<T, B>>,
}

impl<T, B, M> Default for Context<T, B, M> {
	fn default() -> Self {
		Self {
			original_base_url: None,
			base_iri: None,
			vocabulary: None,
			default_language: None,
			default_base_direction: None,
			previous_context: None,
			definitions: Definitions::default(),
			inverse: OnceCell::default(),
		}
	}
}

pub type DefinitionEntryRef<'a, T, B, M> = (&'a Key, &'a TermDefinition<T, B, M>);

impl<T, B, M> Context<T, B, M> {
	/// Create a new context with the given base IRI.
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
			definitions: Definitions::default(),
			inverse: OnceCell::default(),
		}
	}

	/// Returns a reference to the given `term` definition, if any.
	pub fn get<Q: ?Sized>(&self, term: &Q) -> Option<TermDefinitionRef<T, B, M>>
	where
		Key: Borrow<Q>,
		KeywordType: Borrow<Q>,
		Q: Hash + Eq,
	{
		self.definitions.get(term)
	}

	/// Returns a reference to the given `term` normal definition, if any.
	pub fn get_normal<Q: ?Sized>(&self, term: &Q) -> Option<&NormalTermDefinition<T, B, M>>
	where
		Key: Borrow<Q>,
		Q: Hash + Eq,
	{
		self.definitions.get_normal(term)
	}

	/// Returns a reference to the `@type` definition, if any.
	pub fn get_type(&self) -> Option<&TypeTermDefinition> {
		self.definitions.get_type()
	}

	/// Checks if the given `term` is defined.
	pub fn contains_term<Q: ?Sized>(&self, term: &Q) -> bool
	where
		Key: Borrow<Q>,
		KeywordType: Borrow<Q>,
		Q: Hash + Eq,
	{
		self.definitions.contains_term(term)
	}

	/// Returns the original base URL of the context.
	pub fn original_base_url(&self) -> Option<&T> {
		self.original_base_url.as_ref()
	}

	/// Returns the base IRI of the context.
	pub fn base_iri(&self) -> Option<&T> {
		self.base_iri.as_ref()
	}

	/// Returns the `@vocab` value, if any.
	pub fn vocabulary(&self) -> Option<&Term<T, B>> {
		match &self.vocabulary {
			Some(v) => Some(v),
			None => None,
		}
	}

	/// Returns the default `@language` value.
	pub fn default_language(&self) -> Option<LenientLanguageTag> {
		self.default_language.as_ref().map(|tag| tag.as_ref())
	}

	/// Returns the default `@direction` value.
	pub fn default_base_direction(&self) -> Option<Direction> {
		self.default_base_direction
	}

	/// Returns a reference to the previous context.
	pub fn previous_context(&self) -> Option<&Self> {
		match &self.previous_context {
			Some(c) => Some(c),
			None => None,
		}
	}

	/// Returns the number of terms defined.
	pub fn len(&self) -> usize {
		self.definitions.len()
	}

	/// Checks if no terms are defined.
	pub fn is_empty(&self) -> bool {
		self.definitions.is_empty()
	}

	/// Returns a handle to the term definitions.
	pub fn definitions(&self) -> &Definitions<T, B, M> {
		&self.definitions
	}

	/// Checks if the context has a protected definition.
	pub fn has_protected_items(&self) -> bool {
		for binding in self.definitions() {
			if binding.definition().protected() {
				return true;
			}
		}

		false
	}

	/// Returns the inverse of this context.
	pub fn inverse(&self) -> &InverseContext<T, B>
	where
		T: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
	{
		self.inverse.get_or_init(|| self.into())
	}

	/// Sets the normal definition for the given term `key`.
	pub fn set_normal(
		&mut self,
		key: Key,
		definition: Option<NormalTermDefinition<T, B, M>>,
	) -> Option<NormalTermDefinition<T, B, M>> {
		self.inverse.take();
		self.definitions.set_normal(key, definition)
	}

	/// Sets the `@type` definition.
	pub fn set_type(&mut self, type_: Option<TypeTermDefinition>) -> Option<TypeTermDefinition> {
		self.definitions.set_type(type_)
	}

	/// Sets the base IRI.
	pub fn set_base_iri(&mut self, iri: Option<T>) {
		self.inverse.take();
		self.base_iri = iri
	}

	/// Sets the `@vocab` value.
	pub fn set_vocabulary(&mut self, vocab: Option<Term<T, B>>) {
		self.inverse.take();
		self.vocabulary = vocab;
	}

	/// Sets the default `@language` value.
	pub fn set_default_language(&mut self, lang: Option<LenientLanguageTagBuf>) {
		self.inverse.take();
		self.default_language = lang;
	}

	/// Sets the default `@direction` value.
	pub fn set_default_base_direction(&mut self, dir: Option<Direction>) {
		self.inverse.take();
		self.default_base_direction = dir;
	}

	/// Sets the previous context.
	pub fn set_previous_context(&mut self, previous: Self) {
		self.inverse.take();
		self.previous_context = Some(Box::new(previous))
	}

	/// Converts this context into its syntactic definition.
	pub fn into_syntax_definition(
		self,
		vocabulary: &impl Vocabulary<Iri = T, BlankId = B>,
		meta: M,
	) -> Meta<json_ld_syntax::context::Definition<M>, M>
	where
		M: Clone,
	{
		use json_ld_syntax::{Entry, Nullable};

		let (bindings, type_) = self.definitions.into_parts();

		let definition = json_ld_syntax::context::Definition {
			base: self.base_iri.map(|i| {
				Entry::new_with(
					meta.clone(),
					Meta(
						Nullable::Some(vocabulary.iri(&i).unwrap().to_owned().into()),
						meta.clone(),
					),
				)
			}),
			import: None,
			language: self
				.default_language
				.map(|l| Entry::new_with(meta.clone(), Meta(Nullable::Some(l), meta.clone()))),
			direction: self
				.default_base_direction
				.map(|d| Entry::new_with(meta.clone(), Meta(Nullable::Some(d), meta.clone()))),
			propagate: None,
			protected: None,
			type_: type_
				.map(|t| Entry::new_with(meta.clone(), t.into_syntax_definition(meta.clone()))),
			version: None,
			vocab: self.vocabulary.map(|v| {
				let vocab = match v {
					Term::Null => Nullable::Null,
					Term::Id(r) => Nullable::Some(r.with(vocabulary).to_string().into()),
					Term::Keyword(_) => panic!("invalid vocab"),
				};

				Entry::new_with(meta.clone(), Meta(vocab, meta.clone()))
			}),
			bindings: bindings
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

/// Context fragment to syntax method.
pub trait IntoSyntax<T, B, M> {
	fn into_syntax(
		self,
		vocabulary: &impl Vocabulary<Iri = T, BlankId = B>,
		meta: M,
	) -> json_ld_syntax::context::Context<M>;
}

impl<T, B, M> IntoSyntax<T, B, M> for json_ld_syntax::context::Context<M> {
	fn into_syntax(
		self,
		_namespace: &impl Vocabulary<Iri = T, BlankId = B>,
		_meta: M,
	) -> json_ld_syntax::context::Context<M> {
		self
	}
}

impl<T, B, M: Clone> IntoSyntax<T, B, M> for Context<T, B, M> {
	fn into_syntax(
		self,
		vocabulary: &impl Vocabulary<Iri = T, BlankId = B>,
		meta: M,
	) -> json_ld_syntax::context::Context<M> {
		let Meta(definition, meta) = self.into_syntax_definition(vocabulary, meta);
		json_ld_syntax::context::Context::One(Meta(
			json_ld_syntax::ContextEntry::Definition(definition),
			meta,
		))
	}
}

impl<T: Clone, B: Clone, M: Clone> Clone for Context<T, B, M> {
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

impl<T: PartialEq, B: PartialEq, M> StrippedPartialEq for Context<T, B, M> {
	fn stripped_eq(&self, other: &Self) -> bool {
		self.original_base_url == other.original_base_url
			&& self.base_iri == other.base_iri
			&& self.vocabulary == other.vocabulary
			&& self.default_language == other.default_language
			&& self.default_base_direction == other.default_base_direction
			&& self.previous_context.stripped() == other.previous_context.stripped()
	}
}
