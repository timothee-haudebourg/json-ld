//! Context processing algorithm and types.

mod definition;
mod loader;
mod processing;
pub mod inverse;

use std::{
	collections::HashMap,
	marker::PhantomData
};
use futures::future::BoxFuture;
use iref::{Iri, IriBuf};
use langtag::{
	LanguageTag,
	LanguageTagBuf
};
use json::JsonValue;
use crate::{
	ProcessingMode,
	Error,
	Direction,
	Id,
	syntax::Term,
	util
};

pub use definition::*;
pub use loader::*;
pub use processing::*;
pub use inverse::{
	InverseContext,
	Inversible
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ProcessingOptions {
	/// The processing mode
	pub processing_mode: ProcessingMode,

	/// Override protected definitions.
	pub override_protected: bool,

	/// Propagate the processed context.
	pub propagate: bool
}

impl ProcessingOptions {
	/// Return the same set of options, but with `override_protected` set to `true`.
	pub fn with_override(&self) -> ProcessingOptions {
		let mut opt = *self;
		opt.override_protected = true;
		opt
	}

	/// Return the same set of options, but with `override_protected` set to `false`.
	pub fn with_no_override(&self) -> ProcessingOptions {
		let mut opt = *self;
		opt.override_protected = false;
		opt
	}

	/// Return the same set of options, but with `propagate` set to `false`.
	pub fn without_propagation(&self) -> ProcessingOptions {
		let mut opt = *self;
		opt.propagate = false;
		opt
	}
}

impl Default for ProcessingOptions {
	fn default() -> ProcessingOptions {
		ProcessingOptions {
			processing_mode: ProcessingMode::default(),
			override_protected: false,
			propagate: true
		}
	}
}

/// JSON-LD active context.
///
/// An active context holds all the term definitions used to expand a JSON-LD value.
pub trait Context<T: Id = IriBuf> : Clone {
	// Later
	// type Definitions<'a>: Iterator<Item = (&'a str, TermDefinition<T, Self>)>;

	/// The type of local contexts associated to this type of contexts.
	type LocalContext: Local<T>;

	/// Create a newly-initialized active context with the given *base IRI*.
	fn new(base_iri: Option<Iri>) -> Self;

	/// Get the definition of a term.
	fn get(&self, term: &str) -> Option<&TermDefinition<T, Self>>;

	fn get_opt(&self, term: Option<&str>) -> Option<&TermDefinition<T, Self>> {
		if let Some(term) = term {
			self.get(term)
		} else {
			None
		}
	}

	#[inline]
	fn contains(&self, term: &str) -> bool {
		self.get(term).is_some()
	}

	/// Original base URL of the context.
	fn original_base_url(&self) -> Option<Iri>;

	/// Current *base IRI* of the context.
	fn base_iri(&self) -> Option<Iri>;

	/// Optional vocabulary mapping.
	fn vocabulary(&self) -> Option<&Term<T>>;

	/// Optional default language.
	fn default_language(&self) -> Option<LanguageTag>;

	/// Optional default base direction.
	fn default_base_direction(&self) -> Option<Direction>;

	/// Get the previous context.
	fn previous_context(&self) -> Option<&Self>;

	fn definitions<'a>(&'a self) -> Box<dyn 'a + Iterator<Item = (&'a String, &'a TermDefinition<T, Self>)>>;
}

pub trait ContextMut<T: Id = IriBuf>: Context<T> {
	fn set(&mut self, term: &str, definition: Option<TermDefinition<T, Self>>) -> Option<TermDefinition<T, Self>>;

	fn set_base_iri(&mut self, iri: Option<Iri>);

	fn set_vocabulary(&mut self, vocab: Option<Term<T>>);

	fn set_default_language(&mut self, lang: Option<LanguageTagBuf>);

	fn set_default_base_direction(&mut self, dir: Option<Direction>);

	fn set_previous_context(&mut self, previous: Self);
}

/// Local context used for context expansion.
///
/// Local contexts can be seen as "abstract contexts" that can be processed to enrich an
/// existing active context.
pub trait Local<T: Id = IriBuf>: Sized + PartialEq {
	/// Process the local context with specific options.
	fn process_full<'a, 's: 'a, C: Send + Sync + ContextMut<T>, L: Send + Sync + Loader>(&'s self, active_context: &'a C, stack: ProcessingStack, loader: &'a mut L, base_url: Option<Iri<'a>>, options: ProcessingOptions) -> BoxFuture<'a, Result<Processed<&'s Self, C>, Error>> where C::LocalContext: Send + Sync + From<L::Output> + From<Self>, L::Output: Into<Self>, T: Send + Sync;

	/// Process the local context with specific options.
	fn process_with<'a, 's: 'a, C: Send + Sync + ContextMut<T>, L: Send + Sync + Loader>(&'s self, active_context: &'a C, loader: &'a mut L, base_url: Option<Iri<'a>>, options: ProcessingOptions) -> BoxFuture<'a, Result<Processed<&'s Self, C>, Error>> where C::LocalContext: Send + Sync + From<L::Output> + From<Self>, L::Output: Into<Self>, T: Send + Sync {
		self.process_full(active_context, ProcessingStack::new(), loader, base_url, options)
	}

	/// Process the local context with the given active context with the default options:
	/// `is_remote` is `false`, `override_protected` is `false` and `propagate` is `true`.
	fn process<'a, 's: 'a, C: Send + Sync + ContextMut<T>, L: Send + Sync + Loader>(&'s self, active_context: &'a C, loader: &'a mut L, base_url: Option<Iri<'a>>) -> BoxFuture<'a, Result<Processed<&'s Self, C>, Error>> where C::LocalContext: Send + Sync + From<L::Output> + From<Self>, L::Output: Into<Self>, T: Send + Sync {
		self.process_full(active_context, ProcessingStack::new(), loader, base_url, ProcessingOptions::default())
	}
}

#[derive(Clone)]
pub struct Processed<L, C> {
	local: L,
	processed: C
}

impl<L, C> Processed<L, C> {
	pub fn new(local: L, processed: C) -> Processed<L, C> {
		Processed {
			local,
			processed
		}
	}

	pub fn into_inner(self) -> C {
		self.processed
	}
}

impl<'a, L: Clone, C> Processed<&'a L, C> {
	pub fn owned(self) -> Processed<L, C> {
		Processed {
			local: L::clone(self.local),
			processed: self.processed
		}
	}
}

impl<L: util::AsJson, C> util::AsJson for Processed<L, C> {
	fn as_json(&self) -> JsonValue {
		self.local.as_json()
	}
}

impl<'a, C> util::AsJson for Processed<&'a JsonValue, C> {
	fn as_json(&self) -> JsonValue {
		self.local.clone()
	}
}

impl<L, C> std::ops::Deref for Processed<L, C> {
	type Target = C;

	fn deref(&self) -> &C {
		&self.processed
	}
}

impl<L, C> std::convert::AsRef<C> for Processed<L, C> {
	fn as_ref(&self) -> &C {
		&self.processed
	}
}

#[derive(Clone, PartialEq, Eq)]
pub struct JsonContext<T: Id = IriBuf> {
	original_base_url: Option<IriBuf>,
	base_iri: Option<IriBuf>,
	vocabulary: Option<Term<T>>,
	default_language: Option<LanguageTagBuf>,
	default_base_direction: Option<Direction>,
	previous_context: Option<Box<Self>>,
	definitions: HashMap<String, TermDefinition<T, Self>>
}

impl<T: Id> JsonContext<T> {
	pub fn new(base_iri: Option<Iri>) -> JsonContext<T> {
		JsonContext {
			original_base_url: base_iri.map(|iri| iri.into()),
			base_iri: base_iri.map(|iri| iri.into()),
			vocabulary: None,
			default_language: None,
			default_base_direction: None,
			previous_context: None,
			definitions: HashMap::new()
		}
	}
}

impl<T: Id> Default for JsonContext<T> {
	fn default() -> JsonContext<T> {
		JsonContext {
			original_base_url: None,
			base_iri: None,
			vocabulary: None,
			default_language: None,
			default_base_direction: None,
			previous_context: None,
			definitions: HashMap::new()
		}
	}
}

impl<T: Id> Context<T> for JsonContext<T> {
	type LocalContext = JsonValue;

	fn new(base_iri: Option<Iri>) -> JsonContext<T> {
		Self::new(base_iri)
	}

	fn get(&self, term: &str) -> Option<&TermDefinition<T, Self>> {
		self.definitions.get(term)
	}

	fn original_base_url(&self) -> Option<Iri> {
		match &self.original_base_url {
			Some(iri) => Some(iri.as_iri()),
			None => None
		}
	}

	fn base_iri(&self) -> Option<Iri> {
		match &self.base_iri {
			Some(iri) => Some(iri.as_iri()),
			None => None
		}
	}

	fn vocabulary(&self) -> Option<&Term<T>> {
		match &self.vocabulary {
			Some(v) => Some(v),
			None => None
		}
	}

	fn default_language(&self) -> Option<LanguageTag> {
		match &self.default_language {
			Some(tag) => Some(tag.as_ref()),
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

impl<T: Id> ContextMut<T> for JsonContext<T> {
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
			Some(iri) => {
				let mut iri_buf: IriBuf = iri.into();
				iri_buf.path_mut().normalize();
				Some(iri_buf)
			},
			None => None
		}
	}

	fn set_vocabulary(&mut self, vocab: Option<Term<T>>) {
		self.vocabulary = vocab;
	}

	fn set_default_language(&mut self, lang: Option<LanguageTagBuf>) {
		self.default_language = lang;
	}

	fn set_default_base_direction(&mut self, dir: Option<Direction>) {
		self.default_base_direction = dir;
	}

	fn set_previous_context(&mut self, previous: Self) {
		self.previous_context = Some(Box::new(previous))
	}
}
