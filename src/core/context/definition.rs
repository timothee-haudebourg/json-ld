use super::JsonLdContainer;
use crate::{
	JsonLdBacktrace, JsonLdBacktraceBuf, JsonLdDirection, JsonLdNullable, JsonLdTerm, JsonLdType,
};
use crate::{
	JsonLdLenient,
	syntax::{
		JsonLdContext, JsonLdKeywordType,
		context::{
			ContextTypeContainer, JsonLdContextTerm, JsonLdContextType, JsonLdIndex, JsonLdNest,
		},
	},
};
use langtag::LangTagBuf;
use rdf_syntax::{Iri, IriBuf};
use std::collections::HashMap;
use std::hash::Hash;
use std::{borrow::Borrow, fmt};

/// Term binding.
pub enum JsonLdBinding {
	/// Normal term definition.
	Normal(JsonLdContextTerm, Box<JsonLdNormalTermDefinition>),

	/// `@type` term definition.
	Type(JsonLdTypeTermDefinition),
}

/// Term binding reference.
pub enum JsonLdBindingRef<'a> {
	/// Normal term definition.
	Normal(&'a JsonLdContextTerm, &'a JsonLdNormalTermDefinition),

	/// `@type` term definition.
	Type(&'a JsonLdTypeTermDefinition),
}

pub enum JsonLdBindingTerm<'a> {
	Normal(&'a JsonLdContextTerm),
	Type,
}

impl<'a> JsonLdBindingTerm<'a> {
	pub fn as_str(&self) -> &'a str {
		match self {
			Self::Normal(key) => key.as_str(),
			Self::Type => "@type",
		}
	}
}

impl<'a> fmt::Display for JsonLdBindingTerm<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<'a> JsonLdBindingRef<'a> {
	/// Returns a reference to the bound term.
	pub fn term(&self) -> JsonLdBindingTerm<'a> {
		match self {
			Self::Normal(key, _) => JsonLdBindingTerm::Normal(key),
			Self::Type(_) => JsonLdBindingTerm::Type,
		}
	}

	/// Returns a reference to the bound term definition.
	pub fn definition(&self) -> JsonLdTermDefinitionRef<'a> {
		match self {
			Self::Normal(_, d) => JsonLdTermDefinitionRef::Normal(d),
			Self::Type(d) => JsonLdTermDefinitionRef::Type(d),
		}
	}
}

/// Context term definitions.
#[derive(Clone, Default)]
pub struct JsonLdDefinitions {
	normal: HashMap<JsonLdContextTerm, JsonLdNormalTermDefinition>,
	type_: Option<JsonLdTypeTermDefinition>,
}

impl JsonLdDefinitions {
	#[allow(clippy::type_complexity)]
	pub fn into_parts(
		self,
	) -> (
		HashMap<JsonLdContextTerm, JsonLdNormalTermDefinition>,
		Option<JsonLdTypeTermDefinition>,
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
	pub fn get<Q>(&self, term: &Q) -> Option<JsonLdTermDefinitionRef<'_>>
	where
		Q: ?Sized + Hash + Eq,
		JsonLdContextTerm: Borrow<Q>,
		JsonLdKeywordType: Borrow<Q>,
	{
		if JsonLdKeywordType.borrow() == term {
			self.type_.as_ref().map(JsonLdTermDefinitionRef::Type)
		} else {
			self.normal.get(term).map(JsonLdTermDefinitionRef::Normal)
		}
	}

	/// Returns a reference to the normal definition of the given `term`, if any.
	pub fn get_normal<Q>(&self, term: &Q) -> Option<&JsonLdNormalTermDefinition>
	where
		Q: ?Sized + Hash + Eq,
		JsonLdContextTerm: Borrow<Q>,
	{
		self.normal.get(term)
	}

	/// Returns a reference to the `@type` definition, if any.
	pub fn get_type(&self) -> Option<&JsonLdTypeTermDefinition> {
		self.type_.as_ref()
	}

	pub fn contains_term<Q>(&self, term: &Q) -> bool
	where
		Q: ?Sized + Hash + Eq,
		JsonLdContextTerm: Borrow<Q>,
		JsonLdKeywordType: Borrow<Q>,
	{
		if JsonLdKeywordType.borrow() == term {
			self.type_.is_some()
		} else {
			self.normal.contains_key(term)
		}
	}

	/// Inserts the given `binding`.
	pub fn insert(&mut self, binding: JsonLdBinding) -> Option<JsonLdTermDefinition> {
		match binding {
			JsonLdBinding::Normal(key, definition) => self
				.insert_normal(key, *definition)
				.map(JsonLdTermDefinition::Normal),
			JsonLdBinding::Type(definition) => {
				self.insert_type(definition).map(JsonLdTermDefinition::Type)
			}
		}
	}

	/// Defines the given normal term.
	pub fn insert_normal(
		&mut self,
		term: JsonLdContextTerm,
		definition: JsonLdNormalTermDefinition,
	) -> Option<JsonLdNormalTermDefinition> {
		self.normal.insert(term, definition)
	}

	/// Inserts the given `@type` definition.
	pub fn insert_type(
		&mut self,
		definition: JsonLdTypeTermDefinition,
	) -> Option<JsonLdTypeTermDefinition> {
		self.type_.replace(definition)
	}

	/// Sets the given `term` normal definition.
	pub fn set_normal(
		&mut self,
		term: JsonLdContextTerm,
		definition: Option<JsonLdNormalTermDefinition>,
	) -> Option<JsonLdNormalTermDefinition> {
		match definition {
			Some(d) => self.normal.insert(term, d),
			None => self.normal.remove(&term),
		}
	}

	/// Sets the given `@type` definition.
	pub fn set_type(
		&mut self,
		definition: Option<JsonLdTypeTermDefinition>,
	) -> Option<JsonLdTypeTermDefinition> {
		std::mem::replace(&mut self.type_, definition)
	}

	/// Returns an iterator over the term definitions.
	pub fn iter(&self) -> JsonLdDefinitionsIter<'_> {
		JsonLdDefinitionsIter {
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

pub struct JsonLdDefinitionsIter<'a> {
	type_: Option<&'a JsonLdTypeTermDefinition>,
	normal: std::collections::hash_map::Iter<'a, JsonLdContextTerm, JsonLdNormalTermDefinition>,
}

impl<'a> Iterator for JsonLdDefinitionsIter<'a> {
	type Item = JsonLdBindingRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.type_.take().map(JsonLdBindingRef::Type).or_else(|| {
			self.normal
				.next()
				.map(|(k, d)| JsonLdBindingRef::Normal(k, d))
		})
	}
}

impl<'a> IntoIterator for &'a JsonLdDefinitions {
	type Item = JsonLdBindingRef<'a>;
	type IntoIter = JsonLdDefinitionsIter<'a>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct JsonLdDefinitionsIntoIter {
	type_: Option<JsonLdTypeTermDefinition>,
	normal: std::collections::hash_map::IntoIter<JsonLdContextTerm, JsonLdNormalTermDefinition>,
}

impl Iterator for JsonLdDefinitionsIntoIter {
	type Item = JsonLdBinding;

	fn next(&mut self) -> Option<Self::Item> {
		self.type_.take().map(JsonLdBinding::Type).or_else(|| {
			self.normal
				.next()
				.map(|(k, d)| JsonLdBinding::Normal(k, Box::new(d)))
		})
	}
}

impl IntoIterator for JsonLdDefinitions {
	type Item = JsonLdBinding;
	type IntoIter = JsonLdDefinitionsIntoIter;

	fn into_iter(self) -> Self::IntoIter {
		JsonLdDefinitionsIntoIter {
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
pub struct JsonLdTypeTermDefinition {
	/// Type container.
	pub container: ContextTypeContainer,

	/// Protection flag.
	pub protected: bool,
}

impl Default for JsonLdTypeTermDefinition {
	fn default() -> Self {
		Self {
			container: ContextTypeContainer::Set,
			protected: false,
		}
	}
}

impl JsonLdTypeTermDefinition {
	pub fn modulo_protected_field(&self) -> JsonLdModuloProtected<&Self> {
		JsonLdModuloProtected(self)
	}

	pub fn into_syntax_definition(self) -> JsonLdContextType {
		JsonLdContextType {
			container: self.container,
			protected: if self.protected { Some(true) } else { None },
		}
	}
}

/// Term definition.
#[derive(PartialEq, Eq, Clone)]
pub enum JsonLdTermDefinition {
	/// `@type` term definition.
	Type(JsonLdTypeTermDefinition),

	/// Normal term definition.
	Normal(JsonLdNormalTermDefinition),
}

impl JsonLdTermDefinition {
	pub fn as_ref(&self) -> JsonLdTermDefinitionRef<'_> {
		match self {
			Self::Type(t) => JsonLdTermDefinitionRef::Type(t),
			Self::Normal(n) => JsonLdTermDefinitionRef::Normal(n),
		}
	}

	pub fn modulo_protected_field(&self) -> JsonLdModuloProtected<JsonLdTermDefinitionRef<'_>> {
		JsonLdModuloProtected(self.as_ref())
	}

	pub fn value(&self) -> Option<&JsonLdTerm> {
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

	pub fn context(&self) -> Option<&JsonLdContext> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.context.as_deref(),
		}
	}

	pub fn container(&self) -> JsonLdContainer {
		match self {
			Self::Type(d) => d.container.into(),
			Self::Normal(d) => d.container,
		}
	}

	pub fn direction(&self) -> Option<JsonLdNullable<JsonLdDirection>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.direction,
		}
	}

	pub fn index(&self) -> Option<&JsonLdIndex> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.index.as_ref(),
		}
	}

	pub fn language(&self) -> Option<JsonLdNullable<&JsonLdLenient<LangTagBuf>>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.language.as_ref().map(JsonLdNullable::as_ref),
		}
	}

	pub fn nest(&self) -> Option<&JsonLdNest> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.nest.as_ref(),
		}
	}

	pub fn typ(&self) -> Option<&JsonLdType> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.typ.as_ref(),
		}
	}
}

/// Term definition reference.
#[derive(PartialEq, Eq)]
pub enum JsonLdTermDefinitionRef<'a> {
	/// `@type` definition.
	Type(&'a JsonLdTypeTermDefinition),

	/// Normal definition.
	Normal(&'a JsonLdNormalTermDefinition),
}

impl<'a> JsonLdTermDefinitionRef<'a> {
	pub fn modulo_protected_field(&self) -> JsonLdModuloProtected<Self> {
		JsonLdModuloProtected(*self)
	}

	pub fn value(&self) -> Option<&'a JsonLdTerm> {
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

	pub fn context(&self) -> Option<&'a JsonLdContext> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.context.as_deref(),
		}
	}

	/// Location of the term definition itself, as recorded when it was
	/// created by the Create Term Definition algorithm.
	pub fn location(&self) -> Option<&'a JsonLdBacktrace> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => Some(&d.location),
		}
	}

	pub fn container(&self) -> JsonLdContainer {
		match self {
			Self::Type(d) => d.container.into(),
			Self::Normal(d) => d.container,
		}
	}

	pub fn direction(&self) -> Option<JsonLdNullable<JsonLdDirection>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.direction,
		}
	}

	pub fn index(&self) -> Option<&'a JsonLdIndex> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.index.as_ref(),
		}
	}

	pub fn language(&self) -> Option<JsonLdNullable<&'a JsonLdLenient<LangTagBuf>>> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.language.as_ref().map(JsonLdNullable::as_ref),
		}
	}

	pub fn nest(&self) -> Option<&'a JsonLdNest> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.nest.as_ref(),
		}
	}

	pub fn typ(&self) -> Option<&'a JsonLdType> {
		match self {
			Self::Type(_) => None,
			Self::Normal(d) => d.typ.as_ref(),
		}
	}
}

impl<'a> Clone for JsonLdTermDefinitionRef<'a> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<'a> Copy for JsonLdTermDefinitionRef<'a> {}

// A term definition.
#[derive(PartialEq, Eq, Clone, Default)]
pub struct JsonLdNormalTermDefinition {
	/// Location of the term definition.
	pub location: JsonLdBacktraceBuf,

	// IRI mapping.
	pub value: Option<JsonLdTerm>,

	// Prefix flag.
	pub prefix: bool,

	// Protected flag.
	pub protected: bool,

	// Reverse property flag.
	pub reverse_property: bool,

	// Optional base URL.
	pub base_url: Option<IriBuf>,

	// Optional context.
	pub context: Option<Box<JsonLdContext>>,

	// Container mapping.
	pub container: JsonLdContainer,

	// Optional direction mapping.
	pub direction: Option<JsonLdNullable<JsonLdDirection>>,

	// Optional index mapping.
	pub index: Option<JsonLdIndex>,

	// Optional language mapping.
	pub language: Option<JsonLdNullable<JsonLdLenient<LangTagBuf>>>,

	// Optional nest value.
	pub nest: Option<JsonLdNest>,

	// Optional type mapping.
	pub typ: Option<JsonLdType>,
}

impl JsonLdNormalTermDefinition {
	pub fn modulo_protected_field(&self) -> JsonLdModuloProtected<&Self> {
		JsonLdModuloProtected(self)
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

/// Wrapper to consider a term definition without the `@protected` flag.
pub struct JsonLdModuloProtected<T>(T);

impl<'b> PartialEq<JsonLdModuloProtected<&'b JsonLdNormalTermDefinition>>
	for JsonLdModuloProtected<&JsonLdNormalTermDefinition>
{
	fn eq(&self, other: &JsonLdModuloProtected<&'b JsonLdNormalTermDefinition>) -> bool {
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

impl Eq for JsonLdModuloProtected<&JsonLdNormalTermDefinition> {}

impl<'b> PartialEq<JsonLdModuloProtected<&'b JsonLdTypeTermDefinition>>
	for JsonLdModuloProtected<&JsonLdTypeTermDefinition>
{
	fn eq(&self, other: &JsonLdModuloProtected<&'b JsonLdTypeTermDefinition>) -> bool {
		// NOTE we ignore the `protected` flag.
		self.0.container == other.0.container
	}
}

impl Eq for JsonLdModuloProtected<&JsonLdTypeTermDefinition> {}

impl<'a, 'b> PartialEq<JsonLdModuloProtected<JsonLdTermDefinitionRef<'b>>>
	for JsonLdModuloProtected<JsonLdTermDefinitionRef<'a>>
{
	fn eq(&self, other: &JsonLdModuloProtected<JsonLdTermDefinitionRef<'b>>) -> bool {
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

impl<'a> Eq for JsonLdModuloProtected<JsonLdTermDefinitionRef<'a>> {}
