use iref::IriBuf;
use crate::{
	Id,
	Term,
	Type,
	Direction,
	Container
};
use super::ActiveContext;

// A term definition.
#[derive(Clone)]
pub struct TermDefinition<T: Id, C: ActiveContext<T>> {
	// IRI mapping.
	pub value: Option<Term<T>>,

	// Prefix flag.
	pub prefix: bool,

	// Protected flag.
	pub protected: bool,

	// Reverse property flag.
	pub reverse_property: bool,

	// Optional base URL.
	pub base_url: Option<IriBuf>,

	// Optional context.
	pub context: Option<C::LocalContext>,

	// Container mapping.
	pub container: Container,

	// Optional direction mapping.
	pub direction: Option<Option<Direction>>,

	// Optional index mapping.
	pub index: Option<String>,

	// Optional language mapping.
	pub language: Option<Option<String>>,

	// Optional nest value.
	pub nest: Option<String>,

	// Optional type mapping.
	pub typ: Option<Type<T>>
}

impl<T: Id, C: ActiveContext<T>> Default for TermDefinition<T, C> {
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
			container: Container::new()
		}
	}
}

impl<T: Id, C: ActiveContext<T>> PartialEq for TermDefinition<T, C> {
	fn eq(&self, other: &TermDefinition<T, C>) -> bool {
		// NOTE we ignore the `protected` flag.
		self.prefix == other.prefix &&
		self.reverse_property == other.reverse_property &&
		self.language == other.language &&
		self.direction == other.direction &&
		self.nest == other.nest &&
		self.index == other.index &&
		self.container == other.container &&
		self.base_url == other.base_url &&
		self.value == other.value &&
		self.typ == other.typ &&
		self.context == other.context
	}
}

impl<T: Id, C: ActiveContext<T>> Eq for TermDefinition<T, C> {}
