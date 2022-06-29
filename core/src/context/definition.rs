use super::Nest;
use crate::{Direction, LenientLanguageTagBuf, Nullable, Term, Type};
use iref::{Iri, IriBuf};
use json_ld_syntax::{context::Index, Container};
use locspan_derive::StrippedPartialEq;

// A term definition.
#[derive(StrippedPartialEq, Clone)]
#[stripped(T)]
pub struct TermDefinition<T, C> {
	// IRI mapping.
	#[stripped]
	pub value: Option<Term<T>>,

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
	pub base_url: Option<IriBuf>,

	// Optional context.
	pub context: Option<C>,

	// Container mapping.
	#[stripped]
	pub container: Container,

	// Optional direction mapping.
	#[stripped]
	pub direction: Option<Nullable<Direction>>,

	// Optional index mapping.
	#[stripped]
	pub index: Option<Index>,

	// Optional language mapping.
	#[stripped]
	pub language: Option<Nullable<LenientLanguageTagBuf>>,

	// Optional nest value.
	#[stripped]
	pub nest: Option<Nest>,

	// Optional type mapping.
	#[stripped]
	pub typ: Option<Type<T>>,
}

impl<T, C> TermDefinition<T, C> {
	pub fn base_url(&self) -> Option<Iri> {
		self.base_url.as_ref().map(|iri| iri.as_iri())
	}
}

impl<T, C> Default for TermDefinition<T, C> {
	fn default() -> TermDefinition<T, C> {
		TermDefinition {
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

impl<T: PartialEq, C: PartialEq> PartialEq for TermDefinition<T, C> {
	fn eq(&self, other: &TermDefinition<T, C>) -> bool {
		// NOTE we ignore the `protected` flag.
		self.prefix == other.prefix
			&& self.reverse_property == other.reverse_property
			&& self.language == other.language
			&& self.direction == other.direction
			&& self.nest == other.nest
			&& self.index == other.index
			&& self.container == other.container
			&& self.base_url == other.base_url
			&& self.value == other.value
			&& self.typ == other.typ
			&& self.context == other.context
	}
}

impl<T: Eq, C: Eq> Eq for TermDefinition<T, C> {}
