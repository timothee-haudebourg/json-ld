use crate::{id, object::value, Direction, Id, Indexed, Node, Object, Reference, ValidReference};
use iref::{Iri, IriBuf};
use json_syntax::Print;
use langtag::LanguageTagBuf;
use smallvec::SmallVec;
use static_iref::iri;
use std::convert::TryFrom;
use std::fmt;

mod quad;
pub use quad::*;

/// RDF display.
pub trait Display {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;

	fn rdf_display(&self) -> Displayed<Self> {
		Displayed(self)
	}
}

pub struct Displayed<'a, T: ?Sized>(&'a T);

impl<'a, T: ?Sized + Display> fmt::Display for Displayed<'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl Display for String {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "\"")?;

		for c in self.chars() {
			match c {
				'\"' => write!(f, "\\\"")?,
				'\\' => write!(f, "\\\\")?,
				'\t' => write!(f, "\\t")?,
				'\r' => write!(f, "\\r")?,
				'\n' => write!(f, "\\n")?,
				c => std::fmt::Display::fmt(&c, f)?,
			}
		}

		write!(f, "\"")
	}
}

/// Members of the <http://www.w3.org/1999/02/22-rdf-syntax-ns#> graph.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum RdfSyntax {
	Type,
	First,
	Rest,
	Value,
	Direction,
}

impl RdfSyntax {
	pub fn as_iri(&self) -> Iri<'static> {
		match self {
			Self::Type => iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
			Self::First => iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#first"),
			Self::Rest => iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#rest"),
			Self::Value => iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#value"),
			Self::Direction => iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#direction"),
		}
	}
}

impl fmt::Display for RdfSyntax {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "<{}>", self.as_iri())
	}
}

/// RDF property reference.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum PropertyRef<'a, T: Id> {
	Rdf(RdfSyntax),
	Other(&'a ValidReference<T>),
}

impl<'a, T: Id> PropertyRef<'a, T> {
	pub fn as_iri(&self) -> Option<Iri<'a>> {
		match self {
			Self::Rdf(s) => Some(s.as_iri()),
			Self::Other(r) => r.as_iri(),
		}
	}
}

impl<'a, T: Id> fmt::Display for PropertyRef<'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Rdf(s) => s.fmt(f),
			Self::Other(r) => r.rdf_display().fmt(f),
		}
	}
}

/// RDF property.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Property<T: Id> {
	Rdf(RdfSyntax),
	Other(ValidReference<T>),
}

impl<T: Id> Property<T> {
	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Self::Rdf(s) => Some(s.as_iri()),
			Self::Other(r) => r.as_iri(),
		}
	}
}

impl<'a, T: Id> TryFrom<Property<T>> for PropertyRef<'a, T> {
	type Error = ValidReference<T>;

	fn try_from(p: Property<T>) -> Result<Self, ValidReference<T>> {
		match p {
			Property::Rdf(s) => Ok(Self::Rdf(s)),
			Property::Other(p) => Err(p),
		}
	}
}

impl<T: Id> fmt::Display for Property<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Rdf(s) => s.fmt(f),
			Self::Other(r) => r.rdf_display().fmt(f),
		}
	}
}

pub type Triple<T> = rdf_types::Triple<ValidReference<T>, Property<T>, Value<T>>;

impl<T: Id> Reference<T> {
	fn rdf_value(&self) -> Option<Value<T>> {
		match self {
			Reference::Id(id) => Some(Value::Reference(ValidReference::Id(id.clone()))),
			Reference::Blank(id) => Some(Value::Reference(ValidReference::Blank(id.clone()))),
			Reference::Invalid(_) => None,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum RdfDirection {
	I18nDatatype,
	CompoundLiteral,
}

pub struct CompoundLiteralTriples<T: Id> {
	id: ValidReference<T>,
	value: Option<Value<T>>,
	direction: Option<Value<T>>,
}

impl<T: Id> Iterator for CompoundLiteralTriples<T> {
	type Item = Triple<T>;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(value) = self.value.take() {
			return Some(rdf_types::Triple(
				self.id.clone(),
				Property::Rdf(RdfSyntax::Value),
				value,
			));
		}

		if let Some(direction) = self.direction.take() {
			return Some(rdf_types::Triple(
				self.id.clone(),
				Property::Rdf(RdfSyntax::Direction),
				direction,
			));
		}

		None
	}
}

pub struct CompoundLiteral<T: Id> {
	value: Value<T>,
	triples: Option<CompoundLiteralTriples<T>>,
}

impl<T: Id, M> crate::object::Value<T, M> {
	fn rdf_value<G: id::Generator<T>>(
		&self,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<CompoundLiteral<T>> {
		match self {
			Self::Json(json) => Some(CompoundLiteral {
				value: Value::Literal(Literal::TypedString(
					json.compact_print().to_string().into(),
					LiteralType::Rdfs(RdfsLiteralType::Json),
				)),
				triples: None,
			}),
			Self::LangString(lang_string) => {
				let (string, language, direction) = lang_string.parts();

				let language = match language {
					Some(language) => match language.as_language_tag() {
						Some(tag) => Some(tag.cloned()),
						None => return None,
					},
					None => None,
				};

				match direction {
					Some(direction) => match rdf_direction {
						Some(RdfDirection::I18nDatatype) => Some(CompoundLiteral {
							value: Value::Literal(Literal::TypedString(
								string.to_string().into(),
								LiteralType::I18n(Box::new(I18nType::new(language, *direction))),
							)),
							triples: None,
						}),
						Some(RdfDirection::CompoundLiteral) => {
							let id = generator.next();
							Some(CompoundLiteral {
								value: Value::Reference(id),
								triples: None,
							})
						}
						None => match language {
							Some(language) => Some(CompoundLiteral {
								value: Value::Literal(Literal::LangString(
									string.to_string().into(),
									language,
								)),
								triples: None,
							}),
							None => Some(CompoundLiteral {
								value: Value::Literal(Literal::String(string.to_string().into())),
								triples: None,
							}),
						},
					},
					None => match language {
						Some(language) => Some(CompoundLiteral {
							value: Value::Literal(Literal::LangString(
								string.to_string().into(),
								language,
							)),
							triples: None,
						}),
						None => Some(CompoundLiteral {
							value: Value::Literal(Literal::TypedString(
								string.to_string().into(),
								LiteralType::Xsd(XsdLiteralType::String),
							)),
							triples: None,
						}),
					},
				}
			}
			Self::Literal(lit, ty) => {
				let (rdf_lit, prefered_rdf_ty) = match lit {
					value::Literal::Boolean(b) => {
						let lit = if *b {
							"true".to_string().into()
						} else {
							"false".to_string().into()
						};

						(lit, Some(LiteralType::Xsd(XsdLiteralType::Boolean)))
					}
					value::Literal::Null => ("null".to_string().into(), None),
					value::Literal::Number(n) => {
						let rdf_ty = if n.is_i64() {
							LiteralType::Xsd(XsdLiteralType::Integer)
						} else {
							LiteralType::Xsd(XsdLiteralType::Double)
						};

						(n.as_str().to_string().into(), Some(rdf_ty))
					}
					value::Literal::String(s) => (s.to_string().into(), None),
				};

				let rdf_ty = match ty {
					Some(id) => Some(LiteralType::from(id.clone())),
					None => prefered_rdf_ty,
				};

				Some(CompoundLiteral {
					value: match rdf_ty {
						Some(ty) => Value::Literal(Literal::TypedString(rdf_lit, ty)),
						None => Value::Literal(Literal::String(rdf_lit)),
					},
					triples: None,
				})
			}
		}
	}
}

impl<T: Id, M> Node<T, M> {
	fn rdf_value(&self) -> Option<Value<T>> {
		self.id().and_then(Reference::rdf_value)
	}
}

impl<T: Id, M> Object<T, M> {
	fn rdf_value<G: id::Generator<T>>(
		&self,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<CompoundValue<T, M>> {
		match self {
			Self::Value(value) => value
				.rdf_value(generator, rdf_direction)
				.map(|compound_value| CompoundValue {
					value: compound_value.value,
					triples: compound_value.triples.map(CompoundValueTriples::Literal),
				}),
			Self::Node(node) => node.rdf_value().map(|value| CompoundValue {
				value,
				triples: None,
			}),
			Self::List(list) => {
				if list.is_empty() {
					Some(CompoundValue {
						value: Value::Nil,
						triples: None,
					})
				} else {
					let id = generator.next();
					Some(CompoundValue {
						value: Value::Reference(Clone::clone(&id)),
						triples: Some(CompoundValueTriples::List(ListTriples::new(list, id))),
					})
				}
			}
		}
	}
}

pub struct CompoundValue<'a, T: Id, M> {
	value: Value<T>,
	triples: Option<CompoundValueTriples<'a, T, M>>,
}

impl<'a, T: Id, M> crate::quad::ObjectRef<'a, T, M> {
	pub fn rdf_value<G: id::Generator<T>>(
		&self,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<CompoundValue<'a, T, M>> {
		match self {
			Self::Object(object) => object.rdf_value(generator, rdf_direction),
			Self::Node(node) => node.rdf_value().map(|value| CompoundValue {
				value,
				triples: None,
			}),
			Self::Ref(r) => r.rdf_value().map(|value| CompoundValue {
				value,
				triples: None,
			}),
		}
	}
}

enum ListItemTriples<'a, T: Id, M> {
	NestedList(NestedListTriples<'a, T, M>),
	CompoundLiteral(CompoundLiteralTriples<T>),
}

struct NestedListTriples<'a, T: Id, M> {
	head_ref: Option<ValidReference<T>>,
	previous: Option<ValidReference<T>>,
	iter: std::slice::Iter<'a, Indexed<Object<T, M>>>,
}

struct ListNode<'a, 'i, T: Id, M> {
	id: &'i ValidReference<T>,
	object: &'a Indexed<Object<T, M>>,
}

impl<'a, T: Id, M> NestedListTriples<'a, T, M> {
	fn new(list: &'a [Indexed<Object<T, M>>], head_ref: ValidReference<T>) -> Self {
		Self {
			head_ref: Some(head_ref),
			previous: None,
			iter: list.iter(),
		}
	}

	fn previous(&self) -> Option<&ValidReference<T>> {
		self.previous.as_ref()
	}

	/// Pull the next object of the list.
	///
	/// Uses the given generator to assign as id to the list element.
	fn next<G: id::Generator<T>>(&mut self, generator: &mut G) -> Option<ListNode<'a, '_, T, M>> {
		if let Some(next) = self.iter.next() {
			let id = match self.head_ref.take() {
				Some(id) => id,
				None => generator.next(),
			};

			self.previous = Some(id);
			Some(ListNode {
				object: next,
				id: self.previous.as_ref().unwrap(),
			})
		} else {
			None
		}
	}
}

pub enum CompoundValueTriples<'a, T: Id, M> {
	Literal(CompoundLiteralTriples<T>),
	List(ListTriples<'a, T, M>),
}

impl<'a, T: Id, M> CompoundValueTriples<'a, T, M> {
	pub fn with<G: id::Generator<T>>(
		self,
		generator: G,
		rdf_direction: Option<RdfDirection>,
	) -> CompoundValueTriplesWith<'a, T, M, G> {
		CompoundValueTriplesWith {
			generator,
			rdf_direction,
			inner: self,
		}
	}

	pub fn next<G: id::Generator<T>>(
		&mut self,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<Triple<T>> {
		match self {
			Self::Literal(l) => l.next(),
			Self::List(l) => l.next(generator, rdf_direction),
		}
	}
}

pub struct CompoundValueTriplesWith<'a, T: Id, M, G: id::Generator<T>> {
	generator: G,
	rdf_direction: Option<RdfDirection>,
	inner: CompoundValueTriples<'a, T, M>,
}

impl<'a, T: Id, M, G: id::Generator<T>> Iterator for CompoundValueTriplesWith<'a, T, M, G> {
	type Item = Triple<T>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next(&mut self.generator, self.rdf_direction)
	}
}

/// Iterator over the RDF quads generated from a list of JSON-LD objects.
///
/// If the list contains nested lists, the iterator will also emit quads for those nested lists.
pub struct ListTriples<'a, T: Id, M> {
	stack: SmallVec<[ListItemTriples<'a, T, M>; 2]>,
	pending: Option<Triple<T>>,
}

impl<'a, T: Id, M> ListTriples<'a, T, M> {
	pub fn new(list: &'a [Indexed<Object<T, M>>], head_ref: ValidReference<T>) -> Self {
		let mut stack = SmallVec::new();
		stack.push(ListItemTriples::NestedList(NestedListTriples::new(
			list, head_ref,
		)));

		Self {
			stack,
			pending: None,
		}
	}

	pub fn with<G: id::Generator<T>>(
		self,
		generator: G,
		rdf_direction: Option<RdfDirection>,
	) -> ListTriplesWith<'a, T, M, G> {
		ListTriplesWith {
			generator,
			rdf_direction,
			inner: self,
		}
	}

	pub fn next<G: id::Generator<T>>(
		&mut self,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<Triple<T>> {
		loop {
			if let Some(pending) = self.pending.take() {
				break Some(pending);
			}

			match self.stack.last_mut() {
				Some(ListItemTriples::CompoundLiteral(lit)) => match lit.next() {
					Some(triple) => break Some(triple),
					None => {
						self.stack.pop();
					}
				},
				Some(ListItemTriples::NestedList(list)) => {
					let previous = list.previous().cloned();
					match list.next(generator) {
						Some(node) => {
							if let Some(compound_value) =
								node.object.rdf_value(generator, rdf_direction)
							{
								let id = node.id.clone();

								if let Some(compound_triples) = compound_value.triples {
									match compound_triples {
										CompoundValueTriples::List(list) => {
											self.stack.extend(list.stack.into_iter())
										}
										CompoundValueTriples::Literal(lit) => {
											self.stack.push(ListItemTriples::CompoundLiteral(lit))
										}
									}
								}

								self.pending = Some(rdf_types::Triple(
									id.clone(),
									Property::Rdf(RdfSyntax::First),
									compound_value.value,
								));

								if let Some(previous_id) = previous {
									break Some(rdf_types::Triple(
										previous_id,
										Property::Rdf(RdfSyntax::Rest),
										Value::Reference(id),
									));
								}
							}
						}
						None => {
							self.stack.pop();
							if let Some(previous_id) = previous {
								break Some(rdf_types::Triple(
									previous_id,
									Property::Rdf(RdfSyntax::Rest),
									Value::Nil,
								));
							}
						}
					}
				}
				None => break None,
			}
		}
	}
}

pub struct ListTriplesWith<'a, T: Id, M, G: id::Generator<T>> {
	generator: G,
	rdf_direction: Option<RdfDirection>,
	inner: ListTriples<'a, T, M>,
}

impl<'a, T: Id, M, G: id::Generator<T>> Iterator for ListTriplesWith<'a, T, M, G> {
	type Item = Triple<T>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next(&mut self.generator, self.rdf_direction)
	}
}

// #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
// pub enum LiteralValue {
// 	Null,
// 	Bool(bool),
// 	Number(NumberBuf),
// 	String(String),
// }

// impl fmt::Display for LiteralValue {
// 	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// 		match self {
// 			Self::Null => "\"null\"".fmt(f),
// 			Self::Bool(b) => write!(f, "\"{}\"", b),
// 			Self::Number(n) => write!(f, "\"{}\"", n),
// 			Self::String(s) => s.rdf_display().fmt(f),
// 		}
// 	}
// }

// #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
// pub enum Literal<T> {
// 	Untyped(LiteralValue),
// 	Typed(LiteralValue, LiteralType<T>),
// 	LangString(String, LanguageTagBuf),
// }

// impl<T: Id> fmt::Display for Literal<T> {
// 	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// 		match self {
// 			Self::Untyped(lit) => lit.fmt(f),
// 			Self::Typed(lit, ty) => write!(f, "{}^^<{}>", lit, ty.as_iri()),
// 			Self::LangString(s, language) => write!(f, "{}@{}", s.rdf_display(), language),
// 		}
// 	}
// }

pub type Literal<T> = rdf_types::Literal<rdf_types::StringLiteral, LiteralType<T>>;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct I18nType {
	language: Option<LanguageTagBuf>,
	direction: Direction,
	iri: IriBuf,
}

impl I18nType {
	pub fn new(language: Option<LanguageTagBuf>, direction: Direction) -> Self {
		let iri = match &language {
			Some(language) => format!("https://www.w3.org/ns/i18n#{}_{}", language, direction),
			None => format!("https://www.w3.org/ns/i18n#{}", direction),
		};

		Self {
			language,
			direction,
			iri: IriBuf::from_string(iri).unwrap(),
		}
	}

	pub fn language(&self) -> Option<&LanguageTagBuf> {
		self.language.as_ref()
	}

	pub fn direction(&self) -> Direction {
		self.direction
	}

	pub fn as_iri(&self) -> Iri {
		self.iri.as_iri()
	}
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum XsdLiteralType {
	Boolean,
	Integer,
	Double,
	String,
}

impl XsdLiteralType {
	pub fn as_iri(&self) -> Iri {
		match self {
			Self::Boolean => iri!("http://www.w3.org/2001/XMLSchema#boolean"),
			Self::Integer => iri!("http://www.w3.org/2001/XMLSchema#integer"),
			Self::Double => iri!("http://www.w3.org/2001/XMLSchema#double"),
			Self::String => iri!("http://www.w3.org/2001/XMLSchema#string"),
		}
	}
}

pub struct UnknownXsdLiteralType;

impl<'a> TryFrom<Iri<'a>> for XsdLiteralType {
	type Error = UnknownXsdLiteralType;

	fn try_from(iri: Iri<'a>) -> Result<Self, Self::Error> {
		if iri == iri!("http://www.w3.org/2001/XMLSchema#boolean") {
			Ok(Self::Boolean)
		} else if iri == iri!("http://www.w3.org/2001/XMLSchema#integer") {
			Ok(Self::Integer)
		} else if iri == iri!("http://www.w3.org/2001/XMLSchema#double") {
			Ok(Self::Double)
		} else if iri == iri!("http://www.w3.org/2001/XMLSchema#string") {
			Ok(Self::String)
		} else {
			Err(UnknownXsdLiteralType)
		}
	}
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum RdfsLiteralType {
	Json,
}

impl RdfsLiteralType {
	pub fn as_iri(&self) -> Iri {
		match self {
			Self::Json => iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#JSON"),
		}
	}
}

pub struct UnknownRdfsLiteralType;

impl<'a> TryFrom<Iri<'a>> for RdfsLiteralType {
	type Error = UnknownRdfsLiteralType;

	fn try_from(iri: Iri<'a>) -> Result<Self, Self::Error> {
		if iri == iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#JSON") {
			Ok(Self::Json)
		} else {
			Err(UnknownRdfsLiteralType)
		}
	}
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum LiteralType<T> {
	Rdfs(RdfsLiteralType),
	Xsd(XsdLiteralType),
	I18n(Box<I18nType>),
	Other(T),
}

impl LiteralType<IriBuf> {
	pub fn from_iri(iri: IriBuf) -> Self {
		match RdfsLiteralType::try_from(iri.as_iri()) {
			Ok(t) => Self::Rdfs(t),
			Err(_) => match XsdLiteralType::try_from(iri.as_iri()) {
				Ok(t) => Self::Xsd(t),
				Err(_) => {
					// TODO from I18n
					Self::Other(iri)
				}
			},
		}
	}
}

impl<T: Id> From<T> for LiteralType<T> {
	fn from(id: T) -> Self {
		match RdfsLiteralType::try_from(id.as_iri()) {
			Ok(t) => Self::Rdfs(t),
			Err(_) => match XsdLiteralType::try_from(id.as_iri()) {
				Ok(t) => Self::Xsd(t),
				Err(_) => {
					// TODO from I18n
					Self::Other(id)
				}
			},
		}
	}
}

impl<T: Id> LiteralType<T> {
	fn as_iri(&self) -> Iri {
		match self {
			Self::Rdfs(ty) => ty.as_iri(),
			Self::Xsd(ty) => ty.as_iri(),
			Self::I18n(ty) => ty.as_iri(),
			Self::Other(t) => t.as_iri(),
		}
	}
}

impl<T: Id> fmt::Display for LiteralType<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_iri().fmt(f)
	}
}

/// RDF value.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Value<T: Id> {
	Nil,
	Literal(Literal<T>),
	Reference(ValidReference<T>),
}

/// IRI of the `http://www.w3.org/1999/02/22-rdf-syntax-ns#nil` value.
pub const NIL_IRI: Iri<'static> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#nil");

impl<T: Id> fmt::Display for Value<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Nil => write!(f, "<{}>", NIL_IRI),
			Self::Literal(lit) => lit.rdf_display().fmt(f),
			Self::Reference(r) => r.rdf_display().fmt(f),
		}
	}
}

impl<T: Id> Display for Literal<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::String(s) => s.fmt(f),
			Self::TypedString(s, ty) => write!(f, "{}^^{}", s, ty),
			Self::LangString(s, t) => write!(f, "{}@{}", s, t),
		}
	}
}
