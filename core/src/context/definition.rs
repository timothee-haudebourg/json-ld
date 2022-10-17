use super::{IntoSyntax, Nest};
use crate::{Container, Direction, LenientLanguageTagBuf, Nullable, Term, Type};
use contextual::WithContext;
use json_ld_syntax::{
	context::{
		definition::{Key, KeyOrTypeRef, KeyRef, TypeContainer},
		term_definition::Index,
	},
	Entry, KeywordType,
};
use locspan::{BorrowStripped, Meta, StrippedEq, StrippedPartialEq};
use locspan_derive::{StrippedEq, StrippedPartialEq};
use rdf_types::{IriVocabulary, Vocabulary};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

pub enum Binding<T, B, L, M> {
	Normal(Key, NormalTermDefinition<T, B, L, M>),
	Type(TypeTermDefinition),
}

pub enum BindingRef<'a, T, B, L, M> {
	Normal(&'a Key, &'a NormalTermDefinition<T, B, L, M>),
	Type(&'a TypeTermDefinition),
}

impl<'a, T, B, L, M> BindingRef<'a, T, B, L, M> {
	pub fn key(&self) -> KeyOrTypeRef<'a> {
		match self {
			Self::Normal(key, _) => KeyOrTypeRef::Key(KeyRef::from(*key)),
			Self::Type(_) => KeyOrTypeRef::Type,
		}
	}

	pub fn definition(&self) -> TermDefinitionRef<'a, T, B, L, M> {
		match self {
			Self::Normal(_, d) => TermDefinitionRef::Normal(d),
			Self::Type(d) => TermDefinitionRef::Type(d),
		}
	}
}

#[derive(Clone)]
pub struct Definitions<T, B, L, M> {
	normal: HashMap<Key, NormalTermDefinition<T, B, L, M>>,
	type_: Option<TypeTermDefinition>,
}

impl<T, B, L, M> Default for Definitions<T, B, L, M> {
	fn default() -> Self {
		Self {
			normal: HashMap::new(),
			type_: None,
		}
	}
}

impl<T, B, L, M> Definitions<T, B, L, M> {
	#[allow(clippy::type_complexity)]
	pub fn into_parts(
		self,
	) -> (
		HashMap<Key, NormalTermDefinition<T, B, L, M>>,
		Option<TypeTermDefinition>,
	) {
		(self.normal, self.type_)
	}

	pub fn len(&self) -> usize {
		if self.type_.is_some() {
			self.normal.len() + 1
		} else {
			self.normal.len()
		}
	}

	pub fn is_empty(&self) -> bool {
		self.type_.is_none() && self.normal.is_empty()
	}

	pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<TermDefinitionRef<T, B, L, M>>
	where
		Q: Hash + Eq,
		Key: Borrow<Q>,
		KeywordType: Borrow<Q>,
	{
		if KeywordType.borrow() == key {
			self.type_.as_ref().map(TermDefinitionRef::Type)
		} else {
			self.normal.get(key).map(TermDefinitionRef::Normal)
		}
	}

	pub fn get_normal<Q: ?Sized>(&self, key: &Q) -> Option<&NormalTermDefinition<T, B, L, M>>
	where
		Q: Hash + Eq,
		Key: Borrow<Q>,
	{
		self.normal.get(key)
	}

	pub fn get_type(&self) -> Option<&TypeTermDefinition> {
		self.type_.as_ref()
	}

	pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
	where
		Q: Hash + Eq,
		Key: Borrow<Q>,
		KeywordType: Borrow<Q>,
	{
		if KeywordType.borrow() == key {
			self.type_.is_some()
		} else {
			self.normal.contains_key(key)
		}
	}

	pub fn insert(&mut self, binding: Binding<T, B, L, M>) -> Option<TermDefinition<T, B, L, M>> {
		match binding {
			Binding::Normal(key, definition) => self
				.insert_normal(key, definition)
				.map(TermDefinition::Normal),
			Binding::Type(definition) => self.insert_type(definition).map(TermDefinition::Type),
		}
	}

	pub fn insert_normal(
		&mut self,
		key: Key,
		definition: NormalTermDefinition<T, B, L, M>,
	) -> Option<NormalTermDefinition<T, B, L, M>> {
		self.normal.insert(key, definition)
	}

	pub fn insert_type(&mut self, definition: TypeTermDefinition) -> Option<TypeTermDefinition> {
		std::mem::replace(&mut self.type_, Some(definition))
	}

	pub fn set_normal(
		&mut self,
		key: Key,
		definition: Option<NormalTermDefinition<T, B, L, M>>,
	) -> Option<NormalTermDefinition<T, B, L, M>> {
		match definition {
			Some(d) => self.normal.insert(key, d),
			None => self.normal.remove(&key),
		}
	}

	pub fn set_type(
		&mut self,
		definition: Option<TypeTermDefinition>,
	) -> Option<TypeTermDefinition> {
		std::mem::replace(&mut self.type_, definition)
	}

	pub fn iter(&self) -> Iter<T, B, L, M> {
		Iter {
			type_: self.type_.as_ref(),
			normal: self.normal.iter(),
		}
	}
}

pub struct Iter<'a, T, B, L, M> {
	type_: Option<&'a TypeTermDefinition>,
	normal: std::collections::hash_map::Iter<'a, Key, NormalTermDefinition<T, B, L, M>>,
}

impl<'a, T, B, L, M> Iterator for Iter<'a, T, B, L, M> {
	type Item = BindingRef<'a, T, B, L, M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.type_
			.take()
			.map(BindingRef::Type)
			.or_else(|| self.normal.next().map(|(k, d)| BindingRef::Normal(k, d)))
	}
}

impl<'a, T, B, L, M> IntoIterator for &'a Definitions<T, B, L, M> {
	type Item = BindingRef<'a, T, B, L, M>;
	type IntoIter = Iter<'a, T, B, L, M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct IntoIter<T, B, L, M> {
	type_: Option<TypeTermDefinition>,
	normal: std::collections::hash_map::IntoIter<Key, NormalTermDefinition<T, B, L, M>>,
}

impl<T, B, L, M> Iterator for IntoIter<T, B, L, M> {
	type Item = Binding<T, B, L, M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.type_
			.take()
			.map(Binding::Type)
			.or_else(|| self.normal.next().map(|(k, d)| Binding::Normal(k, d)))
	}
}

impl<T, B, L, M> IntoIterator for Definitions<T, B, L, M> {
	type Item = Binding<T, B, L, M>;
	type IntoIter = IntoIter<T, B, L, M>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			type_: self.type_,
			normal: self.normal.into_iter(),
		}
	}
}

#[derive(PartialEq, Eq, StrippedPartialEq, StrippedEq, Clone)]
pub struct TypeTermDefinition {
	pub container: TypeContainer,
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
			container: Entry::new(meta.clone(), Meta(self.container, meta.clone())),
			protected: if self.protected {
				Some(Entry::new(meta.clone(), Meta(true, meta.clone())))
			} else {
				None
			},
		};

		Meta(def, meta)
	}
}

#[derive(PartialEq, Eq, StrippedPartialEq, StrippedEq, Clone)]
#[stripped(T, B)]
pub enum TermDefinition<T, B, C, M> {
	Type(TypeTermDefinition),
	Normal(NormalTermDefinition<T, B, C, M>),
}

impl<T, B, C, M> TermDefinition<T, B, C, M> {
	pub fn as_ref(&self) -> TermDefinitionRef<T, B, C, M> {
		match self {
			Self::Type(t) => TermDefinitionRef::Type(t),
			Self::Normal(n) => TermDefinitionRef::Normal(n),
		}
	}

	pub fn modulo_protected_field(&self) -> ModuloProtected<TermDefinitionRef<T, B, C, M>> {
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

	pub fn context(&self) -> Option<&Entry<C, M>> {
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

#[derive(PartialEq, Eq, StrippedPartialEq, StrippedEq)]
#[stripped(T, B)]
pub enum TermDefinitionRef<'a, T, B, C, M> {
	Type(&'a TypeTermDefinition),
	Normal(&'a NormalTermDefinition<T, B, C, M>),
}

impl<'a, T, B, C, M> TermDefinitionRef<'a, T, B, C, M> {
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

	pub fn context(&self) -> Option<&'a Entry<C, M>> {
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

impl<'a, T, B, C, M> Clone for TermDefinitionRef<'a, T, B, C, M> {
	fn clone(&self) -> Self {
		match self {
			Self::Type(d) => Self::Type(d),
			Self::Normal(d) => Self::Normal(*d),
		}
	}
}

impl<'a, T, B, C, M> Copy for TermDefinitionRef<'a, T, B, C, M> {}

// A term definition.
#[derive(PartialEq, Eq, StrippedPartialEq, StrippedEq, Clone)]
#[stripped(T, B)]
pub struct NormalTermDefinition<T, B, C, M> {
	// IRI mapping.
	#[stripped]
	pub value: Option<Term<T, B>>,

	// Prefix flag.
	#[stripped]
	pub prefix: bool,

	// Protected flag.
	#[stripped]
	pub protected: bool,

	// Reverse property flag.
	#[stripped]
	pub reverse_property: bool,

	// Optional base URL.
	#[stripped]
	pub base_url: Option<T>,

	// Optional context.
	pub context: Option<Entry<C, M>>,

	// Container mapping.
	#[stripped]
	pub container: Container,

	// Optional direction mapping.
	#[stripped]
	pub direction: Option<Nullable<Direction>>,

	// Optional index mapping.
	#[stripped_option_deref2]
	pub index: Option<Entry<Index, M>>,

	// Optional language mapping.
	#[stripped]
	pub language: Option<Nullable<LenientLanguageTagBuf>>,

	// Optional nest value.
	#[stripped_option_deref2]
	pub nest: Option<Entry<Nest, M>>,

	// Optional type mapping.
	#[stripped]
	pub typ: Option<Type<T>>,
}

impl<T, B, C, M> NormalTermDefinition<T, B, C, M> {
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
		C: IntoSyntax<T, B, M>,
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
				Term::Ref(r) => Nullable::Some(Id::Term(r.with(vocabulary).to_string())),
			}
		}

		fn term_into_key<T, B>(
			vocabulary: &impl Vocabulary<Iri = T, BlankId = B>,
			term: Term<T, B>,
		) -> Key {
			match term {
				Term::Null => panic!("invalid key"),
				Term::Keyword(k) => k.to_string().into(),
				Term::Ref(r) => r.with(vocabulary).to_string().into(),
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
				Type::Ref(t) => SyntaxType::Term(vocabulary.iri(&t).unwrap().to_string()),
			}
		}

		let (id, reverse) = if self.reverse_property {
			(
				None,
				self.value.map(|t| {
					Entry::new(
						meta.clone(),
						Meta(term_into_key(vocabulary, t), meta.clone()),
					)
				}),
			)
		} else {
			(
				self.value.map(|t| {
					Entry::new(
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
				Entry::new(
					meta.clone(),
					Meta(
						Nullable::Some(type_into_syntax(vocabulary, t)),
						meta.clone(),
					),
				)
			}),
			context: self.context.map(|e| {
				Entry::new(
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
				.map(|l| Entry::new(meta.clone(), Meta(l, meta.clone()))),
			direction: self
				.direction
				.map(|d| Entry::new(meta.clone(), Meta(d, meta.clone()))),
			container: container
				.map(|Meta(c, m)| Entry::new(meta.clone(), Meta(Nullable::Some(c), m))),
			nest: self.nest.clone(),
			prefix: if self.prefix {
				Some(Entry::new(meta.clone(), Meta(true, meta.clone())))
			} else {
				None
			},
			propagate: None,
			protected: if self.protected {
				Some(Entry::new(meta.clone(), Meta(true, meta.clone())))
			} else {
				None
			},
		};

		Meta(expanded.simplify(), meta)
	}
}

impl<T, B, C, M> Default for NormalTermDefinition<T, B, C, M> {
	fn default() -> NormalTermDefinition<T, B, C, M> {
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

pub struct ModuloProtected<T>(T);

impl<'a, 'b, T: PartialEq, B: PartialEq, C: StrippedPartialEq, M>
	StrippedPartialEq<ModuloProtected<&'b NormalTermDefinition<T, B, C, M>>>
	for ModuloProtected<&'a NormalTermDefinition<T, B, C, M>>
{
	fn stripped_eq(&self, other: &ModuloProtected<&'b NormalTermDefinition<T, B, C, M>>) -> bool {
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

impl<'a, T: Eq, B: Eq, C: StrippedEq, M> StrippedEq
	for ModuloProtected<&'a NormalTermDefinition<T, B, C, M>>
{
}

impl<'a, 'b> StrippedPartialEq<ModuloProtected<&'b TypeTermDefinition>>
	for ModuloProtected<&'a TypeTermDefinition>
{
	fn stripped_eq(&self, other: &ModuloProtected<&'b TypeTermDefinition>) -> bool {
		// NOTE we ignore the `protected` flag.
		self.0.container == other.0.container
	}
}

impl<'a> StrippedEq for ModuloProtected<&'a TypeTermDefinition> {}

impl<'a, 'b, T: PartialEq, B: PartialEq, C: StrippedPartialEq, M>
	StrippedPartialEq<ModuloProtected<TermDefinitionRef<'b, T, B, C, M>>>
	for ModuloProtected<TermDefinitionRef<'a, T, B, C, M>>
{
	fn stripped_eq(&self, other: &ModuloProtected<TermDefinitionRef<'b, T, B, C, M>>) -> bool {
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

impl<'a, T: Eq, B: Eq, C: StrippedEq, M> StrippedEq
	for ModuloProtected<TermDefinitionRef<'a, T, B, C, M>>
{
}
