mod processing;

use std::pin::Pin;
use std::future::Future;
use async_trait::async_trait;
use iref::{Iri, IriRef, IriBuf};
use crate::{Error, Keyword, Direction, Container};

pub use processing::*;

pub trait Id: Clone + PartialEq + Eq {
	fn from_iri(iri: Iri) -> Self;

	fn from_blank_id(id: &str) -> Self;

	fn iri(&self) -> Option<IriBuf>;
}

#[derive(Clone, PartialEq, Eq)]
pub enum Key<T: Id> {
	Id(T),
	Keyword(Keyword)
}

impl<T: Id> Key<T> {
	pub fn is_keyword(&self) -> bool {
		match self {
			Key::Keyword(_) => true,
			_ => false
		}
	}

	pub fn iri(&self) -> Option<IriBuf> {
		match self {
			Key::Id(k) => k.iri(),
			_ => None
		}
	}
}

// A term definition.
pub struct TermDefinition<T: Id, C: ActiveContext<T>> {
	// IRI mapping.
	pub value: Key<T>,

	// Prefix flag.
	pub prefix: bool,

	// Protected flag.
	pub protected: bool,

	// Reverse property flag.
	pub reverse_property: bool,

	// Optional type mapping.
	pub typ: Option<Key<T>>,

	// Optional language mapping.
	pub language: Option<String>,

	// Optional direction mapping.
	pub direction: Option<Direction>,

	// Optional context.
	pub context: Option<Box<dyn LocalContext<T, C>>>,

	// Optional nest value.
	pub nest: Option<String>,

	// Optional index mapping.
	pub index: Option<String>,

	// Container mapping.
	pub container: Container
}

/// JSON-LD active context.
///
/// An active context holds all the term definitions used to expand a JSON-LD value.
pub trait ActiveContext<T: Id> : Sized {
	type Definitions<'a>: Iterator<Item = (&'a str, TermDefinition<T, Self>)>;

	/// Create a newly-initialized active context with the given *base IRI*.
	fn new(original_base_url: Iri, base_iri: Iri) -> Self;

	/// Original base URL of the context.
	fn original_base_url(&self) -> Iri;

	/// Current *base IRI* of the context.
	fn base_iri(&self) -> Option<Iri>;

	/// Get the definition of a term.
	fn get(&self, term: &str) -> Option<TermDefinition<T, Self>>;

	/// Optional vocabulary mapping.
	fn vocabulary(&self) -> Option<&Key<T>>;

	/// Optional default language.
	fn default_language(&self) -> Option<String>;

	/// Optional default base direction.
	fn default_base_direction(&self) -> Option<Direction>;

	/// Get the previous context.
	fn previous_context(&self) -> Option<&Self>;

	/// Make a copy of this active context.
	fn copy(&self) -> Self;

	fn definitions<'a>(&'a self) -> Self::Definitions<'a>;
}

pub trait MutableActiveContext<T: Id>: ActiveContext<T> {
	fn set(&mut self, term: &str, definition: Option<TermDefinition<T, Self>>) -> Option<TermDefinition<T, Self>>;

	fn set_base_iri(&mut self, iri: Option<Iri>);

	fn set_vocabulary(&mut self, vocab: Option<Key<T>>);

	fn set_default_language(&mut self, lang: Option<String>);

	fn set_default_base_direction(&mut self, dir: Option<Direction>);

	fn set_previous_context(&mut self, previous: Self);
}

/// Local context used for context expansion.
///
/// Local contexts can be seen as "abstract contexts" that can be processed to enrich an
/// existing active context.
pub trait LocalContext<T: Id, C: ActiveContext<T>> {
	fn process<'a>(&'a self, active_context: &'a C, base_url: Iri) -> Pin<Box<dyn 'a + Future<Output = Result<C, ContextProcessingError>>>>;
}
