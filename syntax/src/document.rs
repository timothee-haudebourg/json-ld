use crate::{
	Nullable,
	Key,
	Json,
	Number,
	Entry,
	ContextEntry,
	CompactIriBuf,
	LenientLanguageTagBuf,
	Direction
};
use iref::{IriRefBuf, IriBuf};
use rdf_types::BlankIdBuf;

/// JSON-LD document.
pub enum Document<S, P> {
	Node(Node<S, P>),
	Graph(Graph<S, P>),
	Array(Vec<Node<S, P>>)
}

pub enum OneOrMany<T> {
	One(Box<T>),
	Many(Vec<T>)
}

/// Node object.
pub struct Node<S, P> {
	/// `@context` entry.
	pub context: Option<Entry<ContextEntry<S, P>, S, P>>,

	/// `@id` entry.
	pub id: Option<Entry<Id, S, P>>,

	/// `@graph` entry.
	pub graph: Option<Entry<OneOrMany<Node<S, P>>, S, P>>,

	/// `@type` entry.
	pub type_: Option<Entry<OneOrMany<Type>, S, P>>,

	/// `@reverse` entry.
	pub reverse: Option<Entry<Reverse<S, P>, S, P>>,

	/// `@included` entry.
	pub included: Option<Entry<IncludedBlock, S, P>>,

	/// `@index` entry.
	pub index: Option<Entry<String, S, P>>,

	/// `@nest` entry.
	pub nest: Option<Entry<OneOrMany<Nested<S, P>>, S, P>>,

	pub entries: Entries<S, P>
}

/// Graph object.
pub struct Graph<S, P> {
	/// `@context` entry.
	pub context: Option<Entry<ContextEntry<S, P>, S, P>>,

	/// `@graph` entry.
	pub graph: Entry<OneOrMany<Node<S, P>>, S, P>
}

/// Node identifier.
pub enum Id {
	IriRef(IriRefBuf),
	CompactIri(CompactIriBuf),
	Blank(BlankIdBuf) // TODO useful?
}

/// Node type.
pub enum Type {
	IriRef(IriRefBuf),
	CompactIri(CompactIriBuf),
	Blank(BlankIdBuf), // TODO useful?
	Term(String)
}

pub struct Reverse<S, P>(::indexmap::IndexMap<Key, OneOrMany<ReverseProperty<S, P>>>);

pub enum ReverseProperty<S, P> {
	IriRef(IriRefBuf),
	CompactIri(CompactIriBuf),
	Blank(BlankIdBuf),
	Node(Node<S, P>)
}

pub struct IncludedBlock;

pub enum Nested<S, P> {
	Node(Node<S, P>)
}

pub struct Entries<S, P>(::indexmap::IndexMap<Key, OneOrMany<NodeEntry<S, P>>>);

pub enum NodeEntry<S, P> {
	Normal(OneOrMany<NormalNodeEntry<S, P>>),
	LanguageMap(LanguageMap),
	IndexMap(IndexMap),
	IncludedBlock(IncludedBlock),
	IdMap(IdMap),
	TypeMap(TypeMap)
}

pub enum NormalNodeEntry<S, P> {
	Null,
	Boolean(bool),
	Number(Number),
	String(String),
	Node(Node<S, P>),
	Graph(Graph<S, P>),
	Value(Value<S, P>),
	List(List<S, P>),
	Set(Set<S, P>)
}

/// Value object.
pub struct Value<S, P> {
	/// `@value` entry.
	pub value: Entry<Json<S, P>, S, P>,

	/// `@type` entry.
	pub type_: Option<Entry<Nullable<ValueType>, S, P>>,

	/// `@language` entry.
	pub language: Option<Entry<Nullable<LenientLanguageTagBuf>, S, P>>,

	/// `@direction` entry.
	pub direction: Option<Entry<Nullable<Direction>, S, P>>,

	/// `@index` entry.
	pub index: Option<Entry<String, S, P>>
}

/// Node type.
pub enum ValueType {
	Iri(IriBuf),
	CompactIri(CompactIriBuf),
	Term(String),
	Json
}

pub struct List<S, P> {
	/// `@list` entry.
	pub list: Entry<String, S, P>,

	/// `@index` entry.
	pub index: Option<Entry<String, S, P>>
}

pub struct Set<S, P> {
	// ...
}