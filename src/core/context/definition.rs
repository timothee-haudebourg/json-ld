use super::JsonLdContainer;
use crate::{
	JsonLdBacktrace, JsonLdBacktraceBuf, JsonLdDirection, JsonLdNullable, JsonLdTerm, JsonLdType,
};
use crate::{
	JsonLdLenient,
	syntax::{
		JsonLdContext, JsonLdKeywordType,
		context::{
			JsonLdContextTerm, JsonLdContextType, JsonLdContextTypeContainer, JsonLdIndex,
			JsonLdNest,
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
	Normal(JsonLdContextTerm, Box<JsonLdProcessedNormalTermDefinition>),

	/// `@type` term definition.
	Type(JsonLdProcessedTypeTermDefinition),
}

/// Term binding reference.
pub enum JsonLdBindingRef<'a> {
	/// Normal term definition.
	Normal(
		&'a JsonLdContextTerm,
		&'a JsonLdProcessedNormalTermDefinition,
	),

	/// `@type` term definition.
	Type(&'a JsonLdProcessedTypeTermDefinition),
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
	pub fn definition(&self) -> JsonLdProcessedTermDefinitionRef<'a> {
		match self {
			Self::Normal(_, d) => JsonLdProcessedTermDefinitionRef::Normal(d),
			Self::Type(d) => JsonLdProcessedTermDefinitionRef::Type(d),
		}
	}
}

/// Context term definitions.
#[derive(Clone, Default)]
pub struct JsonLdDefinitions {
	normal: HashMap<JsonLdContextTerm, JsonLdProcessedNormalTermDefinition>,
	type_: Option<JsonLdProcessedTypeTermDefinition>,
}

impl JsonLdDefinitions {
	#[allow(clippy::type_complexity)]
	pub fn into_parts(
		self,
	) -> (
		HashMap<JsonLdContextTerm, JsonLdProcessedNormalTermDefinition>,
		Option<JsonLdProcessedTypeTermDefinition>,
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
	pub fn get<Q>(&self, term: &Q) -> Option<JsonLdProcessedTermDefinitionRef<'_>>
	where
		Q: ?Sized + Hash + Eq,
		JsonLdContextTerm: Borrow<Q>,
		JsonLdKeywordType: Borrow<Q>,
	{
		if JsonLdKeywordType.borrow() == term {
			self.type_
				.as_ref()
				.map(JsonLdProcessedTermDefinitionRef::Type)
		} else {
			self.normal
				.get(term)
				.map(JsonLdProcessedTermDefinitionRef::Normal)
		}
	}

	/// Returns a reference to the normal definition of the given `term`, if any.
	pub fn get_normal<Q>(&self, term: &Q) -> Option<&JsonLdProcessedNormalTermDefinition>
	where
		Q: ?Sized + Hash + Eq,
		JsonLdContextTerm: Borrow<Q>,
	{
		self.normal.get(term)
	}

	/// Returns a reference to the `@type` definition, if any.
	pub fn get_type(&self) -> Option<&JsonLdProcessedTypeTermDefinition> {
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
	pub fn insert(&mut self, binding: JsonLdBinding) -> Option<JsonLdProcessedTermDefinition> {
		match binding {
			JsonLdBinding::Normal(key, definition) => self
				.insert_normal(key, *definition)
				.map(JsonLdProcessedTermDefinition::Normal),
			JsonLdBinding::Type(definition) => self
				.insert_type(definition)
				.map(JsonLdProcessedTermDefinition::Type),
		}
	}

	/// Defines the given normal term.
	pub fn insert_normal(
		&mut self,
		term: JsonLdContextTerm,
		definition: JsonLdProcessedNormalTermDefinition,
	) -> Option<JsonLdProcessedNormalTermDefinition> {
		self.normal.insert(term, definition)
	}

	/// Inserts the given `@type` definition.
	pub fn insert_type(
		&mut self,
		definition: JsonLdProcessedTypeTermDefinition,
	) -> Option<JsonLdProcessedTypeTermDefinition> {
		self.type_.replace(definition)
	}

	/// Sets the given `term` normal definition.
	pub fn set_normal(
		&mut self,
		term: JsonLdContextTerm,
		definition: Option<JsonLdProcessedNormalTermDefinition>,
	) -> Option<JsonLdProcessedNormalTermDefinition> {
		match definition {
			Some(d) => self.normal.insert(term, d),
			None => self.normal.remove(&term),
		}
	}

	/// Sets the given `@type` definition.
	pub fn set_type(
		&mut self,
		definition: Option<JsonLdProcessedTypeTermDefinition>,
	) -> Option<JsonLdProcessedTypeTermDefinition> {
		std::mem::replace(&mut self.type_, definition)
	}

	/// Returns an iterator over the term definitions.
	pub fn iter(&self) -> JsonLdDefinitionsIter<'_> {
		JsonLdDefinitionsIter {
			type_: self.type_.as_ref(),
			normal: self.normal.iter(),
		}
	}
}

pub struct JsonLdDefinitionsIter<'a> {
	type_: Option<&'a JsonLdProcessedTypeTermDefinition>,
	normal: std::collections::hash_map::Iter<
		'a,
		JsonLdContextTerm,
		JsonLdProcessedNormalTermDefinition,
	>,
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
	type_: Option<JsonLdProcessedTypeTermDefinition>,
	normal: std::collections::hash_map::IntoIter<
		JsonLdContextTerm,
		JsonLdProcessedNormalTermDefinition,
	>,
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
/// Such definition compared to a [`JsonLdProcessedNormalTermDefinition`] can only contain
/// a `@container` and `@protected` value.
#[derive(PartialEq, Eq, Clone)]
pub struct JsonLdProcessedTypeTermDefinition {
	/// Type container.
	pub container: JsonLdContextTypeContainer,

	/// Protection flag.
	pub protected: bool,
}

impl Default for JsonLdProcessedTypeTermDefinition {
	fn default() -> Self {
		Self {
			container: JsonLdContextTypeContainer::Set,
			protected: false,
		}
	}
}

impl JsonLdProcessedTypeTermDefinition {
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
pub enum JsonLdProcessedTermDefinition {
	/// `@type` term definition.
	Type(JsonLdProcessedTypeTermDefinition),

	/// Normal term definition.
	Normal(JsonLdProcessedNormalTermDefinition),
}

impl JsonLdProcessedTermDefinition {
	pub fn as_ref(&self) -> JsonLdProcessedTermDefinitionRef<'_> {
		match self {
			Self::Type(t) => JsonLdProcessedTermDefinitionRef::Type(t),
			Self::Normal(n) => JsonLdProcessedTermDefinitionRef::Normal(n),
		}
	}

	pub fn modulo_protected_field(
		&self,
	) -> JsonLdModuloProtected<JsonLdProcessedTermDefinitionRef<'_>> {
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
pub enum JsonLdProcessedTermDefinitionRef<'a> {
	/// `@type` definition.
	Type(&'a JsonLdProcessedTypeTermDefinition),

	/// Normal definition.
	Normal(&'a JsonLdProcessedNormalTermDefinition),
}

impl<'a> JsonLdProcessedTermDefinitionRef<'a> {
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

impl<'a> Clone for JsonLdProcessedTermDefinitionRef<'a> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<'a> Copy for JsonLdProcessedTermDefinitionRef<'a> {}

// A term definition.
#[derive(PartialEq, Eq, Clone, Default)]
pub struct JsonLdProcessedNormalTermDefinition {
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

impl JsonLdProcessedNormalTermDefinition {
	pub fn modulo_protected_field(&self) -> JsonLdModuloProtected<&Self> {
		JsonLdModuloProtected(self)
	}

	pub fn base_url(&self) -> Option<&IriBuf> {
		self.base_url.as_ref()
	}
}

/// Wrapper to consider a term definition without the `@protected` flag.
pub struct JsonLdModuloProtected<T>(T);

impl<'b> PartialEq<JsonLdModuloProtected<&'b JsonLdProcessedNormalTermDefinition>>
	for JsonLdModuloProtected<&JsonLdProcessedNormalTermDefinition>
{
	fn eq(&self, other: &JsonLdModuloProtected<&'b JsonLdProcessedNormalTermDefinition>) -> bool {
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

impl Eq for JsonLdModuloProtected<&JsonLdProcessedNormalTermDefinition> {}

impl<'b> PartialEq<JsonLdModuloProtected<&'b JsonLdProcessedTypeTermDefinition>>
	for JsonLdModuloProtected<&JsonLdProcessedTypeTermDefinition>
{
	fn eq(&self, other: &JsonLdModuloProtected<&'b JsonLdProcessedTypeTermDefinition>) -> bool {
		// NOTE we ignore the `protected` flag.
		self.0.container == other.0.container
	}
}

impl Eq for JsonLdModuloProtected<&JsonLdProcessedTypeTermDefinition> {}

impl<'a, 'b> PartialEq<JsonLdModuloProtected<JsonLdProcessedTermDefinitionRef<'b>>>
	for JsonLdModuloProtected<JsonLdProcessedTermDefinitionRef<'a>>
{
	fn eq(&self, other: &JsonLdModuloProtected<JsonLdProcessedTermDefinitionRef<'b>>) -> bool {
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

impl<'a> Eq for JsonLdModuloProtected<JsonLdProcessedTermDefinitionRef<'a>> {}
