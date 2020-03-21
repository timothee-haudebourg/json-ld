mod loader;
mod processing;

use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;
use iref::{Iri, IriBuf};
use json::JsonValue;
use crate::{Keyword, Direction, Container};

pub use loader::*;
pub use processing::*;

pub trait Id: Clone + PartialEq + Eq {
	fn from_iri(iri: Iri) -> Self;

	fn from_blank_id(id: &str) -> Self;

	fn iri(&self) -> Option<Iri>;
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

	pub fn iri(&self) -> Option<Iri> {
		match self {
			Key::Id(k) => k.iri(),
			_ => None
		}
	}
}

// A term definition.
#[derive(Clone, PartialEq, Eq)]
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
	pub context: Option<C::LocalContext>,

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
pub trait ActiveContext<T: Id> : Clone {
	// Later
	// type Definitions<'a>: Iterator<Item = (&'a str, TermDefinition<T, Self>)>;

	/// The type of local contexts associated to this type of contexts.
	type LocalContext: LocalContext<T, Self>;

	/// Create a newly-initialized active context with the given *base IRI*.
	fn new(original_base_url: Iri, base_iri: Iri) -> Self;

	/// Get the definition of a term.
	fn get(&self, term: &str) -> Option<&TermDefinition<T, Self>>;

	/// Original base URL of the context.
	fn original_base_url(&self) -> Iri;

	/// Current *base IRI* of the context.
	fn base_iri(&self) -> Option<Iri>;

	/// Optional vocabulary mapping.
	fn vocabulary(&self) -> Option<&Key<T>>;

	/// Optional default language.
	fn default_language(&self) -> Option<&str>;

	/// Optional default base direction.
	fn default_base_direction(&self) -> Option<Direction>;

	/// Get the previous context.
	fn previous_context(&self) -> Option<&Self>;

	fn definitions<'a>(&'a self) -> Box<dyn 'a + Iterator<Item = (&'a String, &'a TermDefinition<T, Self>)>>;
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
pub trait LocalContext<T: Id, C: ActiveContext<T>>: PartialEq {
	fn process<'a, L: ContextLoader<C::LocalContext>>(&'a self, active_context: &'a C, loader: &'a mut L, base_url: Iri, is_remote: bool, override_protected: bool, propagate: bool) -> Pin<Box<dyn 'a + Future<Output = Result<C, ContextProcessingError>>>>;

	fn as_json_ld(&self) -> &json::JsonValue;
}

#[derive(Clone, PartialEq, Eq)]
pub struct JsonLdContext<T: Id> {
	original_base_url: IriBuf,
	base_iri: Option<IriBuf>,
	vocabulary: Option<Key<T>>,
	default_language: Option<String>,
	default_base_direction: Option<Direction>,
	previous_context: Option<Box<Self>>,
	definitions: HashMap<String, TermDefinition<T, Self>>
}

impl<T: Id> ActiveContext<T> for JsonLdContext<T> {
	type LocalContext = JsonValue;

	fn new(original_base_url: Iri, base_iri: Iri) -> JsonLdContext<T> {
		JsonLdContext {
			original_base_url: original_base_url.into(),
			base_iri: Some(base_iri.into()),
			vocabulary: None,
			default_language: None,
			default_base_direction: None,
			previous_context: None,
			definitions: HashMap::new()
		}
	}

	fn get(&self, term: &str) -> Option<&TermDefinition<T, Self>> {
		self.definitions.get(term)
	}

	fn original_base_url(&self) -> Iri {
		self.original_base_url.as_iri()
	}

	fn base_iri(&self) -> Option<Iri> {
		match &self.base_iri {
			Some(iri) => Some(iri.as_iri()),
			None => None
		}
	}

	fn vocabulary(&self) -> Option<&Key<T>> {
		match &self.vocabulary {
			Some(v) => Some(v),
			None => None
		}
	}

	fn default_language(&self) -> Option<&str> {
		match &self.default_language {
			Some(l) => Some(l.as_str()),
			None => None
		}
	}

	fn default_base_direction(&self) -> Option<Direction> {
		self.default_base_direction
	}

	fn previous_context(&self) -> Option<&Self> {
		match &self.previous_context {
			Some(c) => Some(c),
			None => None
		}
	}

	fn definitions<'a>(&'a self) -> Box<dyn 'a + Iterator<Item = (&'a String, &'a TermDefinition<T, Self>)>> {
		Box::new(self.definitions.iter())
	}
}

impl<T: Id> MutableActiveContext<T> for JsonLdContext<T> {
	fn set(&mut self, term: &str, definition: Option<TermDefinition<T, Self>>) -> Option<TermDefinition<T, Self>> {
		match definition {
			Some(def) => {
				self.definitions.insert(term.to_string(), def)
			},
			None => {
				self.definitions.remove(term)
			}
		}
	}

	fn set_base_iri(&mut self, iri: Option<Iri>) {
		self.base_iri = match iri {
			Some(iri) => Some(iri.into()),
			None => None
		}
	}

	fn set_vocabulary(&mut self, vocab: Option<Key<T>>) {
		self.vocabulary = vocab;
	}

	fn set_default_language(&mut self, lang: Option<String>) {
		self.default_language = lang;
	}

	fn set_default_base_direction(&mut self, dir: Option<Direction>) {
		self.default_base_direction = dir;
	}

	fn set_previous_context(&mut self, previous: Self) {
		self.previous_context = Some(Box::new(previous))
	}
}
