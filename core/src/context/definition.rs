use super::{IntoSyntax, Nest};
use crate::{Container, Direction, LenientLanguageTagBuf, Nullable, Term, Type};
use contextual::WithContext;
use json_ld_syntax::{
	context::{
		definition::{Key, TypeContainer},
		term_definition::Index,
	},
	Entry, KeywordType,
};
use locspan::{BorrowStripped, Meta, StrippedEq, StrippedPartialEq};
use locspan_derive::{StrippedEq, StrippedPartialEq};
use rdf_types::{IriVocabulary, Vocabulary};
use std::collections::HashMap;
use std::hash::Hash;
use std::{borrow::Borrow, fmt};

/// Term binding.
pub enum Binding<T, B, M> {
	/// Normal term definition.
	Normal(Key, NormalTermDefinition<T, B, M>),

	/// `@type` term definition.
	Type(TypeTermDefinition),
}

/// Term binding reference.
pub enum BindingRef<'a, T, B, M> {
	/// Normal term definition.
	Normal(&'a Key, &'a NormalTermDefinition<T, B, M>),

	/// `@type` term definition.
	Type(&'a TypeTermDefinition),
}

pub enum BindingTerm<'a> {
	Normal(&'a Key),
	Type,
}

impl<'a> BindingTerm<'a> {
	pub fn as_str(&self) -> &'a str {
		match self {
			Self::Normal(key) => key.as_str(),
			Self::Type => "@type",
		}
	}
}

impl<'a> fmt::Display for BindingTerm<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a, T, B, M> BindingRef<'a, T, B, M> {
	/// Returns a reference to the bound term.
	pub fn term(&self) -> BindingTerm<'a> {
		match self {
			Self::Normal(key, _) => BindingTerm::Normal(key),
			Self::Type(_) => BindingTerm::Type,
		}
	}

	/// Returns a reference to the bound term definition.
	pub fn definition(&self) -> TermDefinitionRef<'a, T, B, M> {
		match self {
			Self::Normal(_, d) => TermDefinitionRef::Normal(d),
			Self::Type(d) => TermDefinitionRef::Type(d),
		}
	}
}

/// Context term definitions.
#[derive(Clone)]
pub struct Definitions<T, B, M> {
	normal: HashMap<Key, NormalTermDefinition<T, B, M>>,
	type_: Option<TypeTermDefinition>,
}

impl<T, B, M> Default for Definitions<T, B, M> {
	fn default() -> Self {
		Self {
			normal: HashMap::new(),
			type_: None,
		}
	}
}

impl<T, B, M> Definitions<T, B, M> {
	#[allow(clippy::type_complexity)]
	pub fn into_parts(
		self,
	) -> (
		HashMap<Key, NormalTermDefinition<T, B, M>>,
		Option<TypeTermDefinition>,
	) {
		(self.normal, self.type_)
	}

	/// Returns the number of defined terms.
	pub fn len(&self) -> usize {
		if self.type_.is_some() {
			self.normal.len() + 1
		} else {
			self.normal.len()
		}
	}

	/// Checks if no terms are defined.
	pub fn is_empty(&self) -> bool {
		self.type_.is_none() && self.normal.is_empty()
	}

	/// Returns a reference to the definition of the given `term`, if any.
	pub fn get<Q: ?Sized>(&self, term: &Q) -> Option<TermDefinitionRef<T, B, M>>
	where
		Q: Hash + Eq,
		Key: Borrow<Q>,
		KeywordType: Borrow<Q>,
	{
		if KeywordType.borrow() == term {
			self.type_.as_ref().map(TermDefinitionRef::Type)
		} else {
			self.normal.get(term).map(TermDefinitionRef::Normal)
		}
	}

	/// Returns a reference to the normal definition of the given `term`, if any.
	pub fn get_normal<Q: ?Sized>(&self, term: &Q) -> Option<&NormalTermDefinition<T, B, M>>
	where
		Q: Hash + Eq,
		Key: Borrow<Q>,
	{
		self.normal.get(term)
	}

	/// Returns a reference to the `@type` definition, if any.
	pub fn get_type(&self) -> Option<&TypeTermDefinition> {
		self.type_.as_ref()
	}

	pub fn contains_term<Q: ?Sized>(&self, term: &Q) -> bool
	where
		Q: Hash + Eq,
		Key: Borrow<Q>,
		KeywordType: Borrow<Q>,
	{
		if KeywordType.borrow() == term {
			self.type_.is_some()
		} else {
			self.normal.contains_key(term)
		}
	}

	/// Inserts the given `binding`.
	pub fn insert(&mut self, binding: Binding<T, B, M>) -> Option<TermDefinition<T, B, M>> {
		match binding {
			Binding::Normal(key, definition) => self
				.insert_normal(key, definition)
				.map(TermDefinition::Normal),
			Binding::Type(definition) => self.insert_type(definition).map(TermDefinition::Type),
		}
	}

	/// Defines the given normal term.
	pub fn insert_normal(
		&mut self,
		term: Key,
		definition: NormalTermDefinition<T, B, M>,
	) -> Option<NormalTermDefinition<T, B, M>> {
		self.normal.insert(term, definition)
	}

	/// Inserts the given `@type` definition.
	pub fn insert_type(&mut self, definition: TypeTermDefinition) -> Option<TypeTermDefinition> {
		std::mem::replace(&mut self.type_, Some(definition))
	}

	/// Sets the given `term` normal definition.
	pub fn set_normal(
		&mut self,
		term: Key,
		definition: Option<NormalTermDefinition<T, B, M>>,
	) -> Option<NormalTermDefinition<T, B, M>> {
		match definition {
			Some(d) => self.normal.insert(term, d),
			None => self.normal.remove(&term),
		}
	}

	/// Sets the given `@type` definition.
	pub fn set_type(
		&mut self,
		definition: Option<TypeTermDefinition>,
	) -> Option<TypeTermDefinition> {
		std::mem::replace(&mut self.type_, definition)
	}

	/// Returns an iterator over the term definitions.
	pub fn iter(&self) -> Iter<T, B, M> {
		Iter {
			type_: self.type_.as_ref(),
			normal: self.normal.iter(),
		}
	}
}

pub struct Iter<'a, T, B, M> {
	type_: Option<&'a TypeTermDefinition>,
	normal: std::collections::hash_map::Iter<'a, Key, NormalTermDefinition<T, B, M>>,
}

impl<'a, T, B, M> Iterator for Iter<'a, T, B, M> {
	type Item = BindingRef<'a, T, B, M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.type_
			.take()
			.map(BindingRef::Type)
			.or_else(|| self.normal.next().map(|(k, d)| BindingRef::Normal(k, d)))
	}
}

impl<'a, T, B, M> IntoIterator for &'a Definitions<T, B, M> {
	type Item = BindingRef<'a, T, B, M>;
	type IntoIter = Iter<'a, T, B, M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct IntoIter<T, B, M> {
	type_: Option<TypeTermDefinition>,
	normal: std::collections::hash_map::IntoIter<Key, NormalTermDefinition<T, B, M>>,
}

impl<T, B, M> Iterator for IntoIter<T, B, M> {
	type Item = Binding<T, B, M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.type_
			.take()
			.map(Binding::Type)
			.or_else(|| self.normal.next().map(|(k, d)| Binding::Normal(k, d)))
	}
}

impl<T, B, M> IntoIterator for Definitions<T, B, M> {
	type Item = Binding<T, B, M>;
	type IntoIter = IntoIter<T, B, M>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			type_: self.type_,
			normal: self.normal.into_iter(),
		}
	}
}

/// `@type` term definition.
///
/// Such definition compared to a [`NormalTermDefinition`] can only contain
/// a `@container` and `@protected` value.
#[derive(PartialEq, Eq, StrippedPartialEq, StrippedEq, Clone)]
pub struct TypeTermDefinition {
	/// Type container.
	pub container: TypeContainer,

	/// Protection flag.
	pub protected: bool,
}

impl Default for TypeTermDefinition {
	fn default() -> Self {
		Self {
			container: TypeContainer::Set,
			protected: false,
		}
	}
}

impl TypeTermDefinition {
	pub fn modulo_protected_field(&self) -> ModuloProtected<&Self> {
		ModuloProtected(self)
	}

	pub fn into_syntax_definition<M: Clone>(
		&self,
		meta: M,
	) -> Meta<json_ld_syntax::context::definition::Type<M>, M> {
		let def = json_ld_syntax::context::definition::Type {
			container: Entry::new_with(meta.clone(), Meta(self.container, meta.clone())),
			protected: if self.protected {
				Some(Entry::new_with(meta.clone(), Meta(true, meta.clone())))
			} else {
				None
			},
		};

		Meta(def, meta)
	}
}

/// Term definition.
#[derive(PartialEq, Eq, StrippedPartialEq, StrippedEq, Clone)]
#[locspan(stripped(T, B), fixed(T, B))]
pub enum TermDefinition<T, B, M> {
	/// `@type` term definition.
	Type(TypeTermDefinition),

	/// Normal term definition.
	Normal(NormalTermDefinition<T, B, M>),
}

impl<T, B, M> TermDefinition<T, B, M> {
	pub fn as_ref(&self) -> TermDefinitionRef<T, B, M> {
		match self {
			Self::Type(t) => TermDefinitionRef::Type(t),
			Self::Normal(n) => TermDefinitionRef::Normal(n),
		}
	}

	pub fn modulo_protected_field(&self) -> ModuloProtected<TermDefinitionRef<T, B, M>> {
		ModuloProtected(self.as_ref())
	}

	pub fn value(&self) -> Option<&Term<T, B>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.value.as_ref(),
		}
	}

	pub fn prefix(&self) -> bool {
		match self {
			Self::Type(_) => false,
			Self::Normal(d) => d.prefix,
		}
	}

	pub fn protected(&self) -> bool {
		match self {
			Self::Type(d) => d.protected,
			Self::Normal(d) => d.protected,
		}
	}

	pub fn reverse_property(&self) -> bool {
		match self {
			Self::Type(_) => false,
			Self::Normal(d) => d.reverse_property,
		}
	}

	pub fn base_url(&self) -> Option<&T> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.base_url.as_ref(),
		}
	}

	pub fn context(&self) -> Option<&Entry<Box<json_ld_syntax::context::Context<M>>, M>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.context.as_ref(),
		}
	}

	pub fn container(&self) -> Container {
		match self {
			Self::Type(d) => d.container.into(),
			Self::Normal(d) => d.container,
		}
	}

	pub fn direction(&self) -> Option<Nullable<Direction>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.direction,
		}
	}

	pub fn index(&self) -> Option<&Entry<Index, M>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.index.as_ref(),
		}
	}

	pub fn language(&self) -> Option<&Nullable<LenientLanguageTagBuf>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.language.as_ref(),
		}
	}

	pub fn nest(&self) -> Option<&Entry<Nest, M>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.nest.as_ref(),
		}
	}

	pub fn typ(&self) -> Option<&Type<T>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.typ.as_ref(),
		}
	}
}

/// Term definition reference.
#[derive(PartialEq, Eq, StrippedPartialEq, StrippedEq)]
#[locspan(stripped(T, B), fixed(T, B))]
pub enum TermDefinitionRef<'a, T, B, M> {
	/// `@type` definition.
	Type(&'a TypeTermDefinition),

	/// Normal definition.
	Normal(&'a NormalTermDefinition<T, B, M>),
}

impl<'a, T, B, M> TermDefinitionRef<'a, T, B, M> {
	pub fn modulo_protected_field(&self) -> ModuloProtected<Self> {
		ModuloProtected(*self)
	}

	pub fn value(&self) -> Option<&'a Term<T, B>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.value.as_ref(),
		}
	}

	pub fn prefix(&self) -> bool {
		match self {
			Self::Type(_) => false,
			Self::Normal(d) => d.prefix,
		}
	}

	pub fn protected(&self) -> bool {
		match self {
			Self::Type(d) => d.protected,
			Self::Normal(d) => d.protected,
		}
	}

	pub fn reverse_property(&self) -> bool {
		match self {
			Self::Type(_) => false,
			Self::Normal(d) => d.reverse_property,
		}
	}

	pub fn base_url(&self) -> Option<&'a T> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.base_url.as_ref(),
		}
	}

	pub fn context(&self) -> Option<&'a Entry<Box<json_ld_syntax::context::Context<M>>, M>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.context.as_ref(),
		}
	}

	pub fn container(&self) -> Container {
		match self {
			Self::Type(d) => d.container.into(),
			Self::Normal(d) => d.container,
		}
	}

	pub fn direction(&self) -> Option<Nullable<Direction>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.direction,
		}
	}

	pub fn index(&self) -> Option<&'a Entry<Index, M>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.index.as_ref(),
		}
	}

	pub fn language(&self) -> Option<&'a Nullable<LenientLanguageTagBuf>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.language.as_ref(),
		}
	}

	pub fn nest(&self) -> Option<&'a Entry<Nest, M>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.nest.as_ref(),
		}
	}

	pub fn typ(&self) -> Option<&'a Type<T>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.typ.as_ref(),
		}
	}
}

impl<'a, T, B, M> Clone for TermDefinitionRef<'a, T, B, M> {
	fn clone(&self) -> Self {
		match self {
			Self::Type(d) => Self::Type(d),
			Self::Normal(d) => Self::Normal(*d),
		}
	}
}

impl<'a, T, B, M> Copy for TermDefinitionRef<'a, T, B, M> {}

// A term definition.
#[derive(PartialEq, Eq, StrippedPartialEq, StrippedEq, Clone)]
#[locspan(stripped(T, B), fixed(T, B))]
pub struct NormalTermDefinition<T, B, M> {
	// IRI mapping.
	#[locspan(stripped)]
	pub value: Option<Term<T, B>>,

	// Prefix flag.
	#[locspan(stripped)]
	pub prefix: bool,

	// Protected flag.
	#[locspan(stripped)]
	pub protected: bool,

	// Reverse property flag.
	#[locspan(stripped)]
	pub reverse_property: bool,

	// Optional base URL.
	#[locspan(stripped)]
	pub base_url: Option<T>,

	// Optional context.
	pub context: Option<Entry<Box<json_ld_syntax::context::Context<M>>, M>>,

	// Container mapping.
	#[locspan(stripped)]
	pub container: Container,

	// Optional direction mapping.
	#[locspan(stripped)]
	pub direction: Option<Nullable<Direction>>,

	// Optional index mapping.
	#[locspan(unwrap_deref2_stripped)]
	pub index: Option<Entry<Index, M>>,

	// Optional language mapping.
	#[locspan(stripped)]
	pub language: Option<Nullable<LenientLanguageTagBuf>>,

	// Optional nest value.
	#[locspan(unwrap_deref2_stripped)]
	pub nest: Option<Entry<Nest, M>>,

	// Optional type mapping.
	#[locspan(stripped)]
	pub typ: Option<Type<T>>,
}

impl<T, B, M> NormalTermDefinition<T, B, M> {
	pub fn modulo_protected_field(&self) -> ModuloProtected<&Self> {
		ModuloProtected(self)
	}

	pub fn base_url(&self) -> Option<&T> {
		self.base_url.as_ref()
	}

	pub fn into_syntax_definition(
		self,
		vocabulary: &impl Vocabulary<Iri = T, BlankId = B>,
		meta: M,
	) -> Meta<Nullable<json_ld_syntax::context::TermDefinition<M>>, M>
	where
		M: Clone,
	{
		use json_ld_syntax::context::term_definition::{Id, Type as SyntaxType, TypeKeyword};

		fn term_into_id<T, B>(
			vocabulary: &impl Vocabulary<Iri = T, BlankId = B>,
			term: Term<T, B>,
		) -> Nullable<Id> {
			match term {
				Term::Null => Nullable::Null,
				Term::Keyword(k) => Nullable::Some(Id::Keyword(k)),
				Term::Id(r) => Nullable::Some(Id::Term(r.with(vocabulary).to_string())),
			}
		}

		fn term_into_key<T, B>(
			vocabulary: &impl Vocabulary<Iri = T, BlankId = B>,
			term: Term<T, B>,
		) -> Key {
			match term {
				Term::Null => panic!("invalid key"),
				Term::Keyword(k) => k.to_string().into(),
				Term::Id(r) => r.with(vocabulary).to_string().into(),
			}
		}

		fn type_into_syntax<T>(
			vocabulary: &impl IriVocabulary<Iri = T>,
			ty: Type<T>,
		) -> SyntaxType {
			match ty {
				Type::Id => SyntaxType::Keyword(TypeKeyword::Id),
				Type::Json => SyntaxType::Keyword(TypeKeyword::Json),
				Type::None => SyntaxType::Keyword(TypeKeyword::None),
				Type::Vocab => SyntaxType::Keyword(TypeKeyword::Vocab),
				Type::Iri(t) => SyntaxType::Term(vocabulary.iri(&t).unwrap().to_string()),
			}
		}

		let (id, reverse) = if self.reverse_property {
			(
				None,
				self.value.map(|t| {
					Entry::new_with(
						meta.clone(),
						Meta(term_into_key(vocabulary, t), meta.clone()),
					)
				}),
			)
		} else {
			(
				self.value.map(|t| {
					Entry::new_with(
						meta.clone(),
						Meta(term_into_id(vocabulary, t), meta.clone()),
					)
				}),
				None,
			)
		};

		let container = self.container.into_syntax(meta.clone());

		let expanded = json_ld_syntax::context::term_definition::Expanded {
			id,
			type_: self.typ.map(|t| {
				Entry::new_with(
					meta.clone(),
					Meta(
						Nullable::Some(type_into_syntax(vocabulary, t)),
						meta.clone(),
					),
				)
			}),
			context: self.context.map(|e| {
				Entry::new_with(
					e.key_metadata.clone(),
					Meta(
						Box::new(e.value.0.into_syntax(vocabulary, meta.clone())),
						e.value.1,
					),
				)
			}),
			reverse,
			index: self.index.clone(),
			language: self
				.language
				.map(|l| Entry::new_with(meta.clone(), Meta(l, meta.clone()))),
			direction: self
				.direction
				.map(|d| Entry::new_with(meta.clone(), Meta(d, meta.clone()))),
			container: container
				.map(|Meta(c, m)| Entry::new_with(meta.clone(), Meta(Nullable::Some(c), m))),
			nest: self.nest.clone(),
			prefix: if self.prefix {
				Some(Entry::new_with(meta.clone(), Meta(true, meta.clone())))
			} else {
				None
			},
			propagate: None,
			protected: if self.protected {
				Some(Entry::new_with(meta.clone(), Meta(true, meta.clone())))
			} else {
				None
			},
		};

		Meta(expanded.simplify(), meta)
	}
}

impl<T, B, M> Default for NormalTermDefinition<T, B, M> {
	fn default() -> NormalTermDefinition<T, B, M> {
		NormalTermDefinition {
			value: None,
			prefix: false,
			protected: false,
			reverse_property: false,
			base_url: None,
			typ: None,
			language: None,
			direction: None,
			context: None,
			nest: None,
			index: None,
			container: Container::new(),
		}
	}
}

/// Wrapper to consider a term definition without the `@protected` flag.
pub struct ModuloProtected<T>(T);

impl<'a, 'b, T: PartialEq, B: PartialEq, M, N>
	StrippedPartialEq<ModuloProtected<&'b NormalTermDefinition<T, B, N>>>
	for ModuloProtected<&'a NormalTermDefinition<T, B, M>>
{
	fn stripped_eq(&self, other: &ModuloProtected<&'b NormalTermDefinition<T, B, N>>) -> bool {
		// NOTE we ignore the `protected` flag.
		self.0.prefix == other.0.prefix
			&& self.0.reverse_property == other.0.reverse_property
			&& self.0.language == other.0.language
			&& self.0.direction == other.0.direction
			&& self.0.nest.stripped() == other.0.nest.stripped()
			&& self.0.index.stripped() == other.0.index.stripped()
			&& self.0.container == other.0.container
			&& self.0.base_url == other.0.base_url
			&& self.0.value == other.0.value
			&& self.0.typ == other.0.typ
			&& self.0.context.stripped() == other.0.context.stripped()
	}
}

impl<'a, T: Eq, B: Eq, M> StrippedEq for ModuloProtected<&'a NormalTermDefinition<T, B, M>> {}

impl<'a, 'b> StrippedPartialEq<ModuloProtected<&'b TypeTermDefinition>>
	for ModuloProtected<&'a TypeTermDefinition>
{
	fn stripped_eq(&self, other: &ModuloProtected<&'b TypeTermDefinition>) -> bool {
		// NOTE we ignore the `protected` flag.
		self.0.container == other.0.container
	}
}

impl<'a> StrippedEq for ModuloProtected<&'a TypeTermDefinition> {}

impl<'a, 'b, T: PartialEq, B: PartialEq, M>
	StrippedPartialEq<ModuloProtected<TermDefinitionRef<'b, T, B, M>>>
	for ModuloProtected<TermDefinitionRef<'a, T, B, M>>
{
	fn stripped_eq(&self, other: &ModuloProtected<TermDefinitionRef<'b, T, B, M>>) -> bool {
		// NOTE we ignore the `protected` flag.
		self.0.prefix() == other.0.prefix()
			&& self.0.reverse_property() == other.0.reverse_property()
			&& self.0.language() == other.0.language()
			&& self.0.direction() == other.0.direction()
			&& self.0.nest().stripped() == other.0.nest().stripped()
			&& self.0.index().stripped() == other.0.index().stripped()
			&& self.0.container() == other.0.container()
			&& self.0.base_url() == other.0.base_url()
			&& self.0.value() == other.0.value()
			&& self.0.typ() == other.0.typ()
			&& self.0.context().stripped() == other.0.context().stripped()
	}
}

impl<'a, T: Eq, B: Eq, M> StrippedEq for ModuloProtected<TermDefinitionRef<'a, T, B, M>> {}
