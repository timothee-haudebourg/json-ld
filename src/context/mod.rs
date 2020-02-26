use crate::Keyword;

pub enum Key<T> {
	Id(T),
	Keyword(Keyword)
}

// A term definition.
pub struct TermDefinition<T> {
	// IRI mapping.
	iri: Key<T>,

	// Reverse proxy flag.
	reverse_proxy: bool,

	// Optional type mapping.
	typ: Option<Key>,

	// Optional language mapping.
	language: Option<String>,

	// Optional direction mapping.
	direction: Option<Direction>,

	// Optional context.
	context: Option<LocalContext>,

	// Optional nest value.
	nest: Option<String>,

	// Optional prefix flag.
	prefix: Option<bool>,

	// Optional index mapping.
	index: Option<String>,

	// Protected flag.
	protected: bool,

	// Container mapping.
	container: Vec<Container>
}

pub enum Container {
	Graph,
	Id,
	Index,
	Language,
	List,
	Set,
	Type
}

trait ActiveContext<T> : Sized + Default {
	type CopiedContext : MutableActiveContext<T>;

	fn get(&self, term: &str) -> TermDefinition<T>;

	// Current base IRI.
	fn base_iri(&self) -> String;

	// Optional vocabulary mapping.
	fn vocabulary(&self) -> Option<String>;

	// Optional default language.
	fn default_language(&self) -> Option<String>;

	// Optional default base direction.
	fn default_base_direction(&self) -> Option<Direction>;

	// Get the previous context.
	fn previous_context(&self) -> Option<&Self::CopiedContext>;

	// Make a copy of this active context.
	fn copy(&self) -> Self::CopiedContext;
}

trait MutableActiveContext<T> : ActiveContext<T> {
	fn set(&mut self, term: &str, definition: Option<TermDefinition>) -> Option<TermDefinition>;

	fn set_base_iri(&mut self, iri: String);

	fn set_vocabulary(&mut self, vocab: String);

	fn set_default_language(&mut self, lang: String);

	fn set_default_base_direction(&mut self, dir: Direction);

	fn set_previous_context(&mut self, previous: Self::CopiedContext);
}

trait LocalContext<T, C: ActiveContext<T>> {
	type Error;

	pub fn process(&self, active_context: &C) -> Result<C::CopiedContext, Error>;
}
