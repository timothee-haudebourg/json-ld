use crate::syntax::{
	context::{ContextTerm, ContextType, ContextTypeContainer, Index, Nest},
	Container, Context, KeywordType,
};
use crate::{Direction, LenientLangTagBuf, Nullable, Term, Type};
use iref::{Iri, IriBuf};
use std::collections::HashMap;
use std::hash::Hash;
use std::{borrow::Borrow, fmt};

/// Term binding.
pub enum Binding {
	/// Normal term definition.
	Normal(ContextTerm, NormalTermDefinition),

	/// `@type` term definition.
	Type(TypeTermDefinition),
}

/// Term binding reference.
pub enum BindingRef<'a> {
	/// Normal term definition.
	Normal(&'a ContextTerm, &'a NormalTermDefinition),

	/// `@type` term definition.
	Type(&'a TypeTermDefinition),
}

pub enum BindingTerm<'a> {
	Normal(&'a ContextTerm),
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

impl<'a> BindingRef<'a> {
	/// Returns a reference to the bound term.
	pub fn term(&self) -> BindingTerm<'a> {
		match self {
			Self::Normal(key, _) => BindingTerm::Normal(key),
			Self::Type(_) => BindingTerm::Type,
		}
	}

	/// Returns a reference to the bound term definition.
	pub fn definition(&self) -> TermDefinitionRef<'a> {
		match self {
			Self::Normal(_, d) => TermDefinitionRef::Normal(d),
			Self::Type(d) => TermDefinitionRef::Type(d),
		}
	}
}

/// Context term definitions.
#[derive(Clone)]
pub struct Definitions {
	normal: HashMap<ContextTerm, NormalTermDefinition>,
	type_: Option<TypeTermDefinition>,
}

impl Default for Definitions {
	fn default() -> Self {
		Self {
			normal: HashMap::new(),
			type_: None,
		}
	}
}

impl Definitions {
	#[allow(clippy::type_complexity)]
	pub fn into_parts(
		self,
	) -> (
		HashMap<ContextTerm, NormalTermDefinition>,
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
	pub fn get<Q>(&self, term: &Q) -> Option<TermDefinitionRef>
	where
		Q: ?Sized + Hash + Eq,
		ContextTerm: Borrow<Q>,
		KeywordType: Borrow<Q>,
	{
		if KeywordType.borrow() == term {
			self.type_.as_ref().map(TermDefinitionRef::Type)
		} else {
			self.normal.get(term).map(TermDefinitionRef::Normal)
		}
	}

	/// Returns a reference to the normal definition of the given `term`, if any.
	pub fn get_normal<Q>(&self, term: &Q) -> Option<&NormalTermDefinition>
	where
		Q: ?Sized + Hash + Eq,
		ContextTerm: Borrow<Q>,
	{
		self.normal.get(term)
	}

	/// Returns a reference to the `@type` definition, if any.
	pub fn get_type(&self) -> Option<&TypeTermDefinition> {
		self.type_.as_ref()
	}

	pub fn contains_term<Q>(&self, term: &Q) -> bool
	where
		Q: ?Sized + Hash + Eq,
		ContextTerm: Borrow<Q>,
		KeywordType: Borrow<Q>,
	{
		if KeywordType.borrow() == term {
			self.type_.is_some()
		} else {
			self.normal.contains_key(term)
		}
	}

	/// Inserts the given `binding`.
	pub fn insert(&mut self, binding: Binding) -> Option<TermDefinition> {
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
		term: ContextTerm,
		definition: NormalTermDefinition,
	) -> Option<NormalTermDefinition> {
		self.normal.insert(term, definition)
	}

	/// Inserts the given `@type` definition.
	pub fn insert_type(&mut self, definition: TypeTermDefinition) -> Option<TypeTermDefinition> {
		std::mem::replace(&mut self.type_, Some(definition))
	}

	/// Sets the given `term` normal definition.
	pub fn set_normal(
		&mut self,
		term: ContextTerm,
		definition: Option<NormalTermDefinition>,
	) -> Option<NormalTermDefinition> {
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
	pub fn iter(&self) -> Iter {
		Iter {
			type_: self.type_.as_ref(),
			normal: self.normal.iter(),
		}
	}

	// pub fn map_ids(
	// 	self,
	// 	mut map_iri: impl FnMut(IriBuf) -> IriBuf,
	// 	mut map_id: impl FnMut(Id) -> Id,
	// ) -> Definitions {
	// 	Definitions {
	// 		normal: self
	// 			.normal
	// 			.into_iter()
	// 			.map(|(key, d)| (key, d.map_ids(&mut map_iri, &mut map_id)))
	// 			.collect(),
	// 		type_: self.type_,
	// 	}
	// }
}

pub struct Iter<'a> {
	type_: Option<&'a TypeTermDefinition>,
	normal: std::collections::hash_map::Iter<'a, ContextTerm, NormalTermDefinition>,
}

impl<'a> Iterator for Iter<'a> {
	type Item = BindingRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.type_
			.take()
			.map(BindingRef::Type)
			.or_else(|| self.normal.next().map(|(k, d)| BindingRef::Normal(k, d)))
	}
}

impl<'a> IntoIterator for &'a Definitions {
	type Item = BindingRef<'a>;
	type IntoIter = Iter<'a>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct IntoIter {
	type_: Option<TypeTermDefinition>,
	normal: std::collections::hash_map::IntoIter<ContextTerm, NormalTermDefinition>,
}

impl Iterator for IntoIter {
	type Item = Binding;

	fn next(&mut self) -> Option<Self::Item> {
		self.type_
			.take()
			.map(Binding::Type)
			.or_else(|| self.normal.next().map(|(k, d)| Binding::Normal(k, d)))
	}
}

impl IntoIterator for Definitions {
	type Item = Binding;
	type IntoIter = IntoIter;

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
#[derive(PartialEq, Eq, Clone)]
pub struct TypeTermDefinition {
	/// Type container.
	pub container: ContextTypeContainer,

	/// Protection flag.
	pub protected: bool,
}

impl Default for TypeTermDefinition {
	fn default() -> Self {
		Self {
			container: ContextTypeContainer::Set,
			protected: false,
		}
	}
}

impl TypeTermDefinition {
	pub fn modulo_protected_field(&self) -> ModuloProtected<&Self> {
		ModuloProtected(self)
	}

	pub fn into_syntax_definition(self) -> ContextType {
		ContextType {
			container: self.container,
			protected: if self.protected { Some(true) } else { None },
		}
	}
}

/// Term definition.
#[derive(PartialEq, Eq, Clone)]
pub enum TermDefinition {
	/// `@type` term definition.
	Type(TypeTermDefinition),

	/// Normal term definition.
	Normal(NormalTermDefinition),
}

impl TermDefinition {
	pub fn as_ref(&self) -> TermDefinitionRef {
		match self {
			Self::Type(t) => TermDefinitionRef::Type(t),
			Self::Normal(n) => TermDefinitionRef::Normal(n),
		}
	}

	pub fn modulo_protected_field(&self) -> ModuloProtected<TermDefinitionRef> {
		ModuloProtected(self.as_ref())
	}

	pub fn value(&self) -> Option<&Term> {
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

	pub fn base_url(&self) -> Option<&IriBuf> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.base_url.as_ref(),
		}
	}

	pub fn context(&self) -> Option<&Context> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.context.as_deref(),
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

	pub fn index(&self) -> Option<&Index> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.index.as_ref(),
		}
	}

	pub fn language(&self) -> Option<Nullable<&LenientLangTagBuf>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.language.as_ref().map(Nullable::as_ref),
		}
	}

	pub fn nest(&self) -> Option<&Nest> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.nest.as_ref(),
		}
	}

	pub fn typ(&self) -> Option<&Type> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.typ.as_ref(),
		}
	}
}

/// Term definition reference.
#[derive(PartialEq, Eq)]
pub enum TermDefinitionRef<'a> {
	/// `@type` definition.
	Type(&'a TypeTermDefinition),

	/// Normal definition.
	Normal(&'a NormalTermDefinition),
}

impl<'a> TermDefinitionRef<'a> {
	pub fn modulo_protected_field(&self) -> ModuloProtected<Self> {
		ModuloProtected(*self)
	}

	pub fn value(&self) -> Option<&'a Term> {
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

	pub fn base_url(&self) -> Option<&'a Iri> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.base_url.as_deref(),
		}
	}

	pub fn context(&self) -> Option<&'a Context> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.context.as_deref(),
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

	pub fn index(&self) -> Option<&'a Index> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.index.as_ref(),
		}
	}

	pub fn language(&self) -> Option<Nullable<&'a LenientLangTagBuf>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.language.as_ref().map(Nullable::as_ref),
		}
	}

	pub fn nest(&self) -> Option<&'a Nest> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.nest.as_ref(),
		}
	}

	pub fn typ(&self) -> Option<&'a Type> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.typ.as_ref(),
		}
	}
}

impl<'a> Clone for TermDefinitionRef<'a> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<'a> Copy for TermDefinitionRef<'a> {}

// A term definition.
#[derive(PartialEq, Eq, Clone)]
pub struct NormalTermDefinition {
	// IRI mapping.
	pub value: Option<Term>,

	// Prefix flag.
	pub prefix: bool,

	// Protected flag.
	pub protected: bool,

	// Reverse property flag.
	pub reverse_property: bool,

	// Optional base URL.
	pub base_url: Option<IriBuf>,

	// Optional context.
	pub context: Option<Box<Context>>,

	// Container mapping.
	pub container: Container,

	// Optional direction mapping.
	pub direction: Option<Nullable<Direction>>,

	// Optional index mapping.
	pub index: Option<Index>,

	// Optional language mapping.
	pub language: Option<Nullable<LenientLangTagBuf>>,

	// Optional nest value.
	pub nest: Option<Nest>,

	// Optional type mapping.
	pub typ: Option<Type>,
}

impl NormalTermDefinition {
	pub fn modulo_protected_field(&self) -> ModuloProtected<&Self> {
		ModuloProtected(self)
	}

	pub fn base_url(&self) -> Option<&IriBuf> {
		self.base_url.as_ref()
	}

	// pub fn into_syntax_definition(
	// 	self,
	// 	vocabulary: &impl Vocabulary<Iri = IriBuf, BlankId = BlankIdBuf>,
	// ) -> Nullable<json_ld_syntax::context::TermDefinition> {
	// 	use json_ld_syntax::context::term_definition::{Id, Type as SyntaxType, TypeKeyword};

	// 	fn term_into_id(
	// 		vocabulary: &impl Vocabulary<Iri = IriBuf, BlankId = BlankIdBuf>,
	// 		term: Term<IriBuf, BlankIdBuf>,
	// 	) -> Nullable<Id> {
	// 		match term {
	// 			Term::Null => Nullable::Null,
	// 			Term::Keyword(k) => Nullable::Some(Id::Keyword(k)),
	// 			Term::Id(r) => Nullable::Some(Id::Term(r.with(vocabulary).to_string())),
	// 		}
	// 	}

	// 	fn term_into_key(
	// 		vocabulary: &impl Vocabulary<Iri = IriBuf, BlankId = BlankIdBuf>,
	// 		term: Term<IriBuf, BlankIdBuf>,
	// 	) -> Key {
	// 		match term {
	// 			Term::Null => panic!("invalid key"),
	// 			Term::Keyword(k) => k.to_string().into(),
	// 			Term::Id(r) => r.with(vocabulary).to_string().into(),
	// 		}
	// 	}

	// 	fn type_into_syntax(
	// 		vocabulary: &impl IriVocabulary<Iri = IriBuf>,
	// 		ty: Type<IriBuf>,
	// 	) -> SyntaxType {
	// 		match ty {
	// 			Type::Id => SyntaxType::Keyword(TypeKeyword::Id),
	// 			Type::Json => SyntaxType::Keyword(TypeKeyword::Json),
	// 			Type::None => SyntaxType::Keyword(TypeKeyword::None),
	// 			Type::Vocab => SyntaxType::Keyword(TypeKeyword::Vocab),
	// 			Type::Iri(t) => SyntaxType::Term(vocabulary.iri(&t).unwrap().to_string()),
	// 		}
	// 	}

	// 	let (id, reverse) = if self.reverse_property {
	// 		(None, self.value.map(|t| term_into_key(vocabulary, t)))
	// 	} else {
	// 		(self.value.map(|t| term_into_id(vocabulary, t)), None)
	// 	};

	// 	let container = self.container.into_syntax();

	// 	json_ld_syntax::context::term_definition::Expanded {
	// 		id,
	// 		type_: self
	// 			.typ
	// 			.map(|t| Nullable::Some(type_into_syntax(vocabulary, t))),
	// 		context: self.context.map(|e| Box::new(e.into_syntax(vocabulary))),
	// 		reverse,
	// 		index: self.index.clone(),
	// 		language: self.language,
	// 		direction: self.direction,
	// 		container: container.map(Nullable::Some),
	// 		nest: self.nest.clone(),
	// 		prefix: if self.prefix { Some(true) } else { None },
	// 		propagate: None,
	// 		protected: if self.protected { Some(true) } else { None },
	// 	}
	// 	.simplify()
	// }
}

impl Default for NormalTermDefinition {
	fn default() -> NormalTermDefinition {
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

impl<'a, 'b> PartialEq<ModuloProtected<&'b NormalTermDefinition>>
	for ModuloProtected<&'a NormalTermDefinition>
{
	fn eq(&self, other: &ModuloProtected<&'b NormalTermDefinition>) -> bool {
		// NOTE we ignore the `protected` flag.
		self.0.prefix == other.0.prefix
			&& self.0.reverse_property == other.0.reverse_property
			&& self.0.language == other.0.language
			&& self.0.direction == other.0.direction
			&& self.0.nest == other.0.nest
			&& self.0.index == other.0.index
			&& self.0.container == other.0.container
			&& self.0.base_url == other.0.base_url
			&& self.0.value == other.0.value
			&& self.0.typ == other.0.typ
			&& self.0.context == other.0.context
	}
}

impl<'a> Eq for ModuloProtected<&'a NormalTermDefinition> {}

impl<'a, 'b> PartialEq<ModuloProtected<&'b TypeTermDefinition>>
	for ModuloProtected<&'a TypeTermDefinition>
{
	fn eq(&self, other: &ModuloProtected<&'b TypeTermDefinition>) -> bool {
		// NOTE we ignore the `protected` flag.
		self.0.container == other.0.container
	}
}

impl<'a> Eq for ModuloProtected<&'a TypeTermDefinition> {}

impl<'a, 'b> PartialEq<ModuloProtected<TermDefinitionRef<'b>>>
	for ModuloProtected<TermDefinitionRef<'a>>
{
	fn eq(&self, other: &ModuloProtected<TermDefinitionRef<'b>>) -> bool {
		// NOTE we ignore the `protected` flag.
		self.0.prefix() == other.0.prefix()
			&& self.0.reverse_property() == other.0.reverse_property()
			&& self.0.language() == other.0.language()
			&& self.0.direction() == other.0.direction()
			&& self.0.nest() == other.0.nest()
			&& self.0.index() == other.0.index()
			&& self.0.container() == other.0.container()
			&& self.0.base_url() == other.0.base_url()
			&& self.0.value() == other.0.value()
			&& self.0.typ() == other.0.typ()
			&& self.0.context() == other.0.context()
	}
}

impl<'a> Eq for ModuloProtected<TermDefinitionRef<'a>> {}
