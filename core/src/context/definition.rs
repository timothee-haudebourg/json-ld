use super::Nest;
use crate::{Container, Direction, LenientLanguageTagBuf, Nullable, Term, Type};
use json_ld_syntax::context::term_definition::Index;
use locspan::{BorrowStripped, StrippedEq, StrippedPartialEq};
use locspan_derive::{StrippedEq, StrippedPartialEq};

// A term definition.
#[derive(PartialEq, Eq, StrippedPartialEq, StrippedEq, Clone)]
#[stripped(T, B)]
pub struct TermDefinition<T, B, C> {
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

impl<T, B, C> TermDefinition<T, B, C> {
	pub fn base_url(&self) -> Option<&T> {
		self.base_url.as_ref()
	}

	pub fn modulo_protected_field(&self) -> ModuloProtectedField<T, B, C> {
		ModuloProtectedField(self)
	}
}

impl<T, B, C> Default for TermDefinition<T, B, C> {
	fn default() -> TermDefinition<T, B, C> {
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

pub struct ModuloProtectedField<'a, T, B, C>(&'a TermDefinition<T, B, C>);

impl<'a, 'b, T: PartialEq, B: PartialEq, C: PartialEq> PartialEq<ModuloProtectedField<'b, T, B, C>>
	for ModuloProtectedField<'a, T, B, C>
{
	fn eq(&self, other: &ModuloProtectedField<'b, T, B, C>) -> bool {
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

impl<'a, T: Eq, B: Eq, C: Eq> Eq for ModuloProtectedField<'a, T, B, C> {}

impl<'a, 'b, T: PartialEq, B: PartialEq, C: StrippedPartialEq>
	StrippedPartialEq<ModuloProtectedField<'b, T, B, C>> for ModuloProtectedField<'a, T, B, C>
{
	fn stripped_eq(&self, other: &ModuloProtectedField<'b, T, B, C>) -> bool {
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
			&& self.0.context.stripped() == other.0.context.stripped()
	}
}

impl<'a, T: Eq, B: Eq, C: StrippedEq> StrippedEq for ModuloProtectedField<'a, T, B, C> {}
