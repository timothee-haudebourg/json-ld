use iref::{Iri, IriRef};
use langtag::LanguageTag;
use rdf_types::BlankId;
use locspan::Loc;
use crate::Keyword;

use super::*;

pub trait AnyContextEntry {
	type Source;
	type Span;

	type Definition: AnyContextDefinition<Self::Source, Self::Span>;
	type Definitions<'a>: Iterator<Item=Loc<ContextRef<'a, Self::Definition>, Self::Source, Self::Span>> where Self: 'a;

	fn as_entry_ref(&self) -> ContextEntryRef<Self::Source, Self::Span, Self::Definition, Self::Definitions<'_>>;
}

impl<S: Clone, P: Clone> AnyContextEntry for ContextEntry<S, P> {
	type Source = S;
	type Span = P;

	type Definition = ContextDefinition<S, P>;
	type Definitions<'a> = ManyContexts<'a, S, P> where S: 'a, P: 'a;

	fn as_entry_ref(&self) -> ContextEntryRef<S, P> {
		self.into()
	}
}

/// Reference to a context entry.
pub enum ContextEntryRef<'a, S, P, D=ContextDefinition<S, P>, M=ManyContexts<'a, S, P>> {
	One(Loc<ContextRef<'a, D>, S, P>),
	Many(M)
}

impl<'a, S: Clone, P: Clone, D, M: Iterator<Item=Loc<ContextRef<'a, D>, S, P>>> IntoIterator for ContextEntryRef<'a, S, P, D, M> {
	type Item = Loc<ContextRef<'a, D>, S, P>;
	type IntoIter = ContextEntryIter<'a, S, P, D, M>;

	fn into_iter(self) -> Self::IntoIter {
		match self {
			Self::One(i) => ContextEntryIter::One(Some(i)),
			Self::Many(m) => ContextEntryIter::Many(m)
		}
	}
}

impl<'a, S: Clone, P: Clone> From<&'a ContextEntry<S, P>> for ContextEntryRef<'a, S, P> {
	fn from(e: &'a ContextEntry<S, P>) -> Self {
		match e {
			ContextEntry::One(c) => Self::One(c.borrow_value().cast()),
			ContextEntry::Many(m) => Self::Many(ManyContexts(m.iter()))
		}
	}
}

pub struct ManyContexts<'a, S, P>(std::slice::Iter<'a, Loc<Context<S, P>, S, P>>);

impl<'a, S: Clone, P: Clone> Iterator for ManyContexts<'a, S, P> {
	type Item = Loc<ContextRef<'a, ContextDefinition<S, P>>, S, P>;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|c| c.borrow_value().cast())
	}
}

pub enum ContextEntryIter<'a, S, P, D=ContextDefinition<S, P>, M=ManyContexts<'a, S, P>> {
	One(Option<Loc<ContextRef<'a, D>, S, P>>),
	Many(M)
}

impl<'a, S, P, D, M: Iterator<Item=Loc<ContextRef<'a, D>, S, P>>> Iterator for ContextEntryIter<'a, S, P, D, M> {
	type Item = Loc<ContextRef<'a, D>, S, P>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::One(i) => i.take(),
			Self::Many(m) => m.next()
		}
	}
}

/// Reference to context.
pub enum ContextRef<'a, D> {
	Null,
	IriRef(IriRef<'a>),
	Definition(&'a D)
}

impl<'a, S, P> From<&'a Context<S, P>> for ContextRef<'a, ContextDefinition<S, P>> {
	fn from(c: &'a Context<S, P>) -> Self {
		match c {
			Context::Null => ContextRef::Null,
			Context::IriRef(i) => ContextRef::IriRef(i.as_iri_ref()),
			Context::Definition(d) => ContextRef::Definition(d)
		}
	}
}

pub trait AnyContextDefinition<S, P>: Sized {
	type Bindings<'a>: Iterator<Item=TermBindingRef<'a, S, P, Self>> where Self: 'a, S: 'a, P: 'a;

	fn base(&self) -> Option<Loc<Nullable<IriRef>, S, P>>;
	fn import(&self) -> Option<Loc<IriRef, S, P>>;
	fn language(&self) -> Option<Loc<Nullable<LanguageTag>, S, P>>;
	fn direction(&self) -> Option<Loc<Direction, S, P>>;
	fn propagate(&self) -> Option<Loc<bool, S, P>>;
	fn protected(&self) -> Option<Loc<bool, S, P>>;
	fn type_(&self) -> Option<Loc<ContextType<S, P>, S, P>>;
	fn version(&self) -> Option<Loc<Version, S, P>>;
	fn vocab(&self) -> Option<Loc<Nullable<VocabRef<S, P>>, S, P>>;
	fn bindings(&self) -> Self::Bindings<'_>;
}

impl<S: Clone, P: Clone> AnyContextDefinition<S, P> for ContextDefinition<S, P> {
	type Bindings<'a> = Bindings<'a, S, P> where S: 'a, P: 'a;

	fn base(&self) -> Option<Loc<Nullable<IriRef>, S, P>> {
		self.base.as_ref().map(|v| v.borrow_value().map(|v| v.as_ref().map(|v| v.as_iri_ref())))
	}

	fn import(&self) -> Option<Loc<IriRef, S, P>> {
		self.import.as_ref().map(|v| v.borrow_value().cast())
	}

	fn language(&self) -> Option<Loc<Nullable<LanguageTag>, S, P>> {
		self.language.as_ref().map(|v| v.borrow_value().map(|v| v.as_ref().map(|v| v.as_ref())))
	}

	fn direction(&self) -> Option<Loc<Direction, S, P>> {
		self.direction.clone()
	}

	fn propagate(&self) -> Option<Loc<bool, S, P>> {
		self.propagate.clone()
	}

	fn protected(&self) -> Option<Loc<bool, S, P>> {
		self.protected.clone()
	}

	fn type_(&self) -> Option<Loc<ContextType<S, P>, S, P>> {
		self.type_.clone()
	}

	fn version(&self) -> Option<Loc<Version, S, P>> {
		self.version.clone()
	}

	fn vocab(&self) -> Option<Loc<Nullable<VocabRef<S, P>>, S, P>> {
		self.vocab.as_ref().map(|v| v.borrow_value().map(|v| v.as_ref().cast()))
	}

	fn bindings(&self) -> Self::Bindings<'_> {
		Bindings(self.bindings.iter())
	}
}

pub struct Bindings<'a, S, P>(std::slice::Iter<'a, TermBinding<S, P>>);

impl<'a, S: Clone, P: Clone> Iterator for Bindings<'a, S, P> {
	type Item = TermBindingRef<'a, S, P>;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(Into::into)
	}
}

pub struct TermBindingRef<'a, S, P, C=ContextDefinition<S, P>> {
	pub term: Loc<&'a str, S, P>,
	pub definition: Nullable<TermDefinitionRef<'a, S, P, C>>
}

impl<'a, S: Clone, P: Clone> From<&'a TermBinding<S, P>> for TermBindingRef<'a, S, P> {
	fn from(b: &'a TermBinding<S, P>) -> Self {
		Self {
			term: b.term.borrow_value().map(String::as_str),
			definition: b.definition.as_ref().cast()
		}
	}
}

/// Term definition.
pub enum TermDefinitionRef<'a, S, P, C=ContextDefinition<S, P>> {
	Iri(Iri<'a>),
	CompactIri(CompactIriRef<'a, S, P>),
	Blank(&'a BlankId),
	Expanded(ExpandedTermDefinitionRef<'a, S, P, C>)
}

impl<'a, S: Clone, P: Clone> From<&'a TermDefinition<S, P>> for TermDefinitionRef<'a, S, P> {
	fn from(d: &'a TermDefinition<S, P>) -> Self {
		match d {
			TermDefinition::Iri(i) => Self::Iri(i.as_iri()),
			TermDefinition::CompactIri(c) => Self::CompactIri(c.into()),
			TermDefinition::Blank(b) => Self::Blank(b),
			TermDefinition::Expanded(e) => Self::Expanded(e.into())
		}
	}
}

/// Expanded term definition.
pub struct ExpandedTermDefinitionRef<'a, S, P, C=ContextDefinition<S, P>> {
	pub id: Option<Loc<Nullable<IdRef<'a, S, P>>, S, P>>,
	pub type_: Option<Loc<Nullable<TermDefinitionTypeRef<'a, S, P>>, S, P>>,
	pub context: Option<Loc<&'a C, S, P>>,
	pub reverse: Option<Loc<ReverseRef<'a, S, P>, S, P>>,
	pub index: Option<Loc<IndexRef<'a, S, P>, S, P>>,
	pub language: Option<Loc<Nullable<LanguageTag<'a>>, S, P>>,
	pub container: Option<Loc<Nullable<Container>, S, P>>,
	pub nest: Option<Loc<NestRef<'a>, S, P>>,
	pub prefix: Option<Loc<bool, S, P>>,
	pub propagate: Option<Loc<bool, S, P>>,
	pub protected: Option<Loc<bool, S, P>>
}

impl<'a, S: Clone, P: Clone> From<&'a ExpandedTermDefinition<S, P>> for ExpandedTermDefinitionRef<'a, S, P> {
	fn from(d: &'a ExpandedTermDefinition<S, P>) -> Self {
		Self {
			id: d.id.as_ref().map(|v| v.borrow_value().map(|v| v.as_ref().cast())),
			type_: d.type_.as_ref().map(|v| v.borrow_value().map(|v| v.as_ref().cast())),
			context: d.context.as_ref().map(|v| v.borrow_value().cast()),
			reverse: d.reverse.as_ref().map(|v| v.borrow_value().cast()),
			index: d.index.as_ref().map(|v| v.borrow_value().cast()),
			language: d.language.as_ref().map(|v| v.borrow_value().map(|v| v.as_ref().map(|v| v.as_ref()))),
			container: d.container.clone(),
			nest: d.nest.as_ref().map(|v| v.borrow_value().cast()),
			prefix: d.prefix.clone(),
			propagate: d.propagate.clone(),
			protected: d.protected.clone()
		}
	}
}

pub enum NestRef<'a> {
	Nest,
	Term(&'a str)
}

impl<'a> From<&'a Nest> for NestRef<'a> {
	fn from(n: &'a Nest) -> Self {
		match n {
			Nest::Nest => Self::Nest,
			Nest::Term(t) => Self::Term(t)
		}
	}
}

pub enum IndexRef<'a, S, P> {
	Iri(Iri<'a>),
	CompactIri(CompactIriRef<'a, S, P>),
	Term(&'a str),
}

impl<'a, S: Clone, P: Clone> From<&'a Index<S, P>> for IndexRef<'a, S, P> {
	fn from(i: &'a Index<S, P>) -> Self {
		match i {
			Index::Iri(i) => Self::Iri(i.as_iri()),
			Index::CompactIri(c) => Self::CompactIri(c.into()),
			Index::Term(t) => Self::Term(t)
		}
	}
}

pub enum IdRef<'a, S, P> {
	Iri(Iri<'a>),
	Blank(&'a BlankId),
	CompactIri(CompactIriRef<'a, S, P>),
	Term(&'a str),
	Keyword(Keyword)
}

impl<'a, S: Clone, P: Clone> From<&'a Id<S, P>> for IdRef<'a, S, P> {
	fn from(i: &'a Id<S, P>) -> Self {
		match i {
			Id::Iri(i) => Self::Iri(i.as_iri()),
			Id::Blank(b) => Self::Blank(b),
			Id::CompactIri(c) => Self::CompactIri(c.into()),
			Id::Term(t) => Self::Term(t),
			Id::Keyword(k) => Self::Keyword(*k)
		}
	}
}

pub enum ReverseRef<'a, S, P> {
	Iri(Iri<'a>),
	Blank(&'a BlankId),
	CompactIri(CompactIriRef<'a, S, P>),
	Term(&'a str)
}

impl<'a, S: Clone, P: Clone> From<&'a Reverse<S, P>> for ReverseRef<'a, S, P> {
	fn from(r: &'a Reverse<S, P>) -> Self {
		match r {
			Reverse::Iri(i) => Self::Iri(i.as_iri()),
			Reverse::Blank(b) => Self::Blank(b),
			Reverse::CompactIri(c) => Self::CompactIri(c.into()),
			Reverse::Term(t) => Self::Term(t)
		}
	}
}

pub enum TermDefinitionTypeRef<'a, S, P> {
	Iri(Iri<'a>),
	CompactIri(CompactIriRef<'a, S, P>),
	Term(&'a str),
	Keyword(TypeKeyword)
}

impl<'a, S: Clone, P: Clone> From<&'a TermDefinitionType<S, P>> for TermDefinitionTypeRef<'a, S, P> {
	fn from(t: &'a TermDefinitionType<S, P>) -> Self {
		match t {
			TermDefinitionType::Iri(i) => Self::Iri(i.as_iri()),
			TermDefinitionType::CompactIri(c) => Self::CompactIri(c.into()),
			TermDefinitionType::Term(t) => Self::Term(t),
			TermDefinitionType::Keyword(k) => Self::Keyword(*k)
		}
	}
}

pub enum Expandable<'a, S, P> {
	IriRef(IriRef<'a>),
	CompactIri(CompactIriRef<'a, S, P>),
	Blank(&'a BlankId),
	Keyword(Keyword),
	KeywordLike(&'a str),
	Term(&'a str)
}

pub enum VocabRef<'a, S, P> {
	IriRef(IriRef<'a>),
	CompactIri(CompactIriRef<'a, S, P>),
	Blank(&'a BlankId),
	Term(&'a str)
}

impl<'a, S: Clone, P: Clone> From<&'a Vocab<S, P>> for VocabRef<'a, S, P> {
	fn from(v: &'a Vocab<S, P>) -> Self {
		match v {
			Vocab::IriRef(i) => Self::IriRef(i.as_iri_ref()),
			Vocab::Blank(b) => Self::Blank(b),
			Vocab::CompactIri(c) => Self::CompactIri(c.into()),
			Vocab::Term(t) => Self::Term(t)
		}
	}
}

pub struct CompactIriRef<'a, S, P> {
	pub prefix: Option<Loc<&'a str, S, P>>,
	pub suffix: Loc<&'a str, S, P>
}

impl<'a, S: Clone, P: Clone> From<&'a CompactIri<S, P>> for CompactIriRef<'a, S, P> {
	fn from(i: &'a CompactIri<S, P>) -> Self {
		Self {
			prefix: i.prefix.as_ref().map(|v| v.borrow_value().map(String::as_str)),
			suffix: i.suffix.borrow_value().map(String::as_str)
		}
	}
}