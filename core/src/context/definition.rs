use super::{IntoSyntax, Nest};
use crate::{Container, Direction, LenientLanguageTagBuf, Nullable, Term, Type};
use contextual::WithContext;
use json_ld_syntax::{context::term_definition::Index, Entry};
use locspan::{BorrowStripped, Meta, StrippedEq, StrippedPartialEq};
use locspan_derive::{StrippedEq, StrippedPartialEq};
use rdf_types::{IriVocabulary, Vocabulary};

// A term definition.
#[derive(PartialEq, Eq, StrippedPartialEq, StrippedEq, Clone)]
#[stripped(T, B)]
pub struct TermDefinition<T, B, C, M> {
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
	pub context: Option<Entry<C, M>>,

	// Container mapping.
	#[stripped]
	pub container: Container,

	// Optional direction mapping.
	#[stripped]
	pub direction: Option<Nullable<Direction>>,

	// Optional index mapping.
	#[stripped_option_deref2]
	pub index: Option<Entry<Index, M>>,

	// Optional language mapping.
	#[stripped]
	pub language: Option<Nullable<LenientLanguageTagBuf>>,

	// Optional nest value.
	#[stripped_option_deref2]
	pub nest: Option<Entry<Nest, M>>,

	// Optional type mapping.
	#[stripped]
	pub typ: Option<Type<T>>,
}

impl<T, B, C, M> TermDefinition<T, B, C, M> {
	pub fn base_url(&self) -> Option<&T> {
		self.base_url.as_ref()
	}

	pub fn modulo_protected_field(&self) -> ModuloProtectedField<T, B, C, M> {
		ModuloProtectedField(self)
	}

	pub fn into_syntax_definition(
		self,
		vocabulary: &impl Vocabulary<Iri=T, BlankId=B>,
		meta: M,
	) -> Meta<Nullable<json_ld_syntax::context::TermDefinition<M>>, M>
	where
		C: IntoSyntax<T, B, M>,
		M: Clone,
	{
		use json_ld_syntax::context::{
			definition::Key,
			term_definition::{Id, Type as SyntaxType, TypeKeyword},
		};

		fn term_into_id<T, B>(
			vocabulary: &impl Vocabulary<Iri=T, BlankId=B>,
			term: Term<T, B>,
		) -> Nullable<Id> {
			match term {
				Term::Null => Nullable::Null,
				Term::Keyword(k) => Nullable::Some(Id::Keyword(k)),
				Term::Ref(r) => Nullable::Some(Id::Term(r.with(vocabulary).to_string())),
			}
		}

		fn term_into_key<T, B>(vocabulary: &impl Vocabulary<Iri=T, BlankId=B>, term: Term<T, B>) -> Key {
			match term {
				Term::Null => panic!("invalid key"),
				Term::Keyword(k) => k.to_string().into(),
				Term::Ref(r) => r.with(vocabulary).to_string().into(),
			}
		}

		fn type_into_syntax<T>(vocabulary: &impl IriVocabulary<Iri=T>, ty: Type<T>) -> SyntaxType {
			match ty {
				Type::Id => SyntaxType::Keyword(TypeKeyword::Id),
				Type::Json => SyntaxType::Keyword(TypeKeyword::Json),
				Type::None => SyntaxType::Keyword(TypeKeyword::None),
				Type::Vocab => SyntaxType::Keyword(TypeKeyword::Vocab),
				Type::Ref(t) => SyntaxType::Term(vocabulary.iri(&t).unwrap().to_string()),
			}
		}

		let (id, reverse) = if self.reverse_property {
			(
				None,
				self.value.map(|t| {
					Entry::new(
						meta.clone(),
						Meta(term_into_key(vocabulary, t), meta.clone()),
					)
				}),
			)
		} else {
			(
				self.value.map(|t| {
					Entry::new(
						meta.clone(),
						Meta(term_into_id(vocabulary, t), meta.clone()),
					)
				}),
				None,
			)
		};

		let container = self.container.into_syntax(meta.clone());

		let expanded = json_ld_syntax::context::term_definition::Expanded {
			id,
			type_: self.typ.map(|t| {
				Entry::new(
					meta.clone(),
					Meta(
						Nullable::Some(type_into_syntax(vocabulary, t)),
						meta.clone(),
					),
				)
			}),
			context: self.context.map(|e| {
				Entry::new(
					e.key_metadata.clone(),
					Meta(
						Box::new(e.value.0.into_syntax(vocabulary, meta.clone())),
						e.value.1.clone(),
					),
				)
			}),
			reverse,
			index: self.index.clone(),
			language: self
				.language
				.map(|l| Entry::new(meta.clone(), Meta(l, meta.clone()))),
			direction: self
				.direction
				.map(|d| Entry::new(meta.clone(), Meta(d, meta.clone()))),
			container: container
				.map(|Meta(c, m)| Entry::new(meta.clone(), Meta(Nullable::Some(c), m))),
			nest: self.nest.clone(),
			prefix: if self.prefix {
				Some(Entry::new(meta.clone(), Meta(true, meta.clone())))
			} else {
				None
			},
			propagate: None,
			protected: if self.protected {
				Some(Entry::new(meta.clone(), Meta(true, meta.clone())))
			} else {
				None
			},
		};

		Meta(expanded.simplify(), meta)
	}
}

impl<T, B, C, M> Default for TermDefinition<T, B, C, M> {
	fn default() -> TermDefinition<T, B, C, M> {
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

pub struct ModuloProtectedField<'a, T, B, C, M>(&'a TermDefinition<T, B, C, M>);

impl<'a, 'b, T: PartialEq, B: PartialEq, C: StrippedPartialEq, M>
	StrippedPartialEq<ModuloProtectedField<'b, T, B, C, M>> for ModuloProtectedField<'a, T, B, C, M>
{
	fn stripped_eq(&self, other: &ModuloProtectedField<'b, T, B, C, M>) -> bool {
		// NOTE we ignore the `protected` flag.
		self.0.prefix == other.0.prefix
			&& self.0.reverse_property == other.0.reverse_property
			&& self.0.language == other.0.language
			&& self.0.direction == other.0.direction
			&& self.0.nest.stripped() == other.0.nest.stripped()
			&& self.0.index.stripped() == other.0.index.stripped()
			&& self.0.container == other.0.container
			&& self.0.base_url == other.0.base_url
			&& self.0.value == other.0.value
			&& self.0.typ == other.0.typ
			&& self.0.context.stripped() == other.0.context.stripped()
	}
}

impl<'a, T: Eq, B: Eq, C: StrippedEq, M> StrippedEq for ModuloProtectedField<'a, T, B, C, M> {}
