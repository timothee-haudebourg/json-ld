use crate::{id, object::value, Direction, Id, Indexed, Node, Object, Reference, ValidReference};
use generic_json::{Json, JsonHash};
use iref::{Iri, IriBuf};
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
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RdfSyntax {
	Type,
	First,
	Rest,
	Value,
	Direction,
}

impl RdfSyntax {
	fn as_iri(&self) -> Iri<'static> {
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

pub struct Triple<T: Id>(ValidReference<T>, Property<T>, Value<T>);

impl<T: Id> Reference<T> {
	fn rdf_value(&self) -> Option<Value<T>> {
		match self {
			Reference::Id(id) => Some(Value::Reference(ValidReference::Id(id.clone()))),
			Reference::Blank(id) => Some(Value::Reference(ValidReference::Blank(id.clone()))),
			Reference::Invalid(_) => None,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
			return Some(Triple(
				self.id.clone(),
				Property::Rdf(RdfSyntax::Value),
				value,
			));
		}

		if let Some(direction) = self.direction.take() {
			return Some(Triple(
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

impl<J: Json + ToString, T: Id> crate::object::Value<J, T> {
	fn rdf_value<G: id::Generator<T>>(
		&self,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<CompoundLiteral<T>> {
		match self {
			Self::Json(json) => Some(CompoundLiteral {
				value: Value::Literal(Literal::Typed(
					LiteralValue::String(json.to_string()),
					LiteralType::Json,
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
							value: Value::Literal(Literal::Typed(
								LiteralValue::String(string.to_string()),
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
									string.to_string(),
									language,
								)),
								triples: None,
							}),
							None => Some(CompoundLiteral {
								value: Value::Literal(Literal::Untyped(LiteralValue::String(
									string.to_string(),
								))),
								triples: None,
							}),
						},
					},
					None => match language {
						Some(language) => Some(CompoundLiteral {
							value: Value::Literal(Literal::LangString(
								string.to_string(),
								language,
							)),
							triples: None,
						}),
						None => Some(CompoundLiteral {
							value: Value::Literal(Literal::Typed(
								LiteralValue::String(string.to_string()),
								LiteralType::String,
							)),
							triples: None,
						}),
					},
				}
			}
			Self::Literal(lit, ty) => {
				let (rdf_lit, prefered_rdf_ty) = match lit {
					value::Literal::Boolean(b) => (LiteralValue::Bool(*b), Some(LiteralType::Bool)),
					value::Literal::Null => (LiteralValue::Null, None),
					value::Literal::Number(n) => {
						use generic_json::Number as JsonNumber;

						let (rdf_number, rdf_ty) = if let Some(n) = n.as_i64() {
							(Number::Integer(n), LiteralType::Integer)
						} else if let Some(n) = n.as_i32() {
							(Number::Integer(n as i64), LiteralType::Integer)
						} else if let Some(f) = n.as_f64() {
							(Number::Double(f), LiteralType::Double)
						} else if let Some(f) = n.as_f32() {
							(Number::Double(f as f64), LiteralType::Double)
						} else {
							(Number::Double(n.as_f64_lossy()), LiteralType::Double)
						};

						(LiteralValue::Number(rdf_number), Some(rdf_ty))
					}
					value::Literal::String(s) => (LiteralValue::String(s.to_string()), None),
				};

				let rdf_ty = match ty {
					Some(id) => Some(LiteralType::from(id.clone())),
					None => prefered_rdf_ty,
				};

				let rdf_lit = match (rdf_lit, &rdf_ty) {
					(LiteralValue::Number(Number::Integer(i)), Some(LiteralType::Double)) => {
						LiteralValue::Number(Number::Double(i as f64))
					}
					(rdf_lit, _) => rdf_lit,
				};

				Some(CompoundLiteral {
					value: match rdf_ty {
						Some(ty) => Value::Literal(Literal::Typed(rdf_lit, ty)),
						None => Value::Literal(Literal::Untyped(rdf_lit)),
					},
					triples: None,
				})
			}
		}
	}
}

impl<J: JsonHash, T: Id> Node<J, T> {
	fn rdf_value(&self) -> Option<Value<T>> {
		self.id().and_then(Reference::rdf_value)
	}
}

impl<J: JsonHash + ToString, T: Id> Object<J, T> {
	fn rdf_value<G: id::Generator<T>>(
		&self,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<CompoundValue<J, T>> {
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

pub struct CompoundValue<'a, J: JsonHash, T: Id> {
	value: Value<T>,
	triples: Option<CompoundValueTriples<'a, J, T>>,
}

impl<'a, J: JsonHash + ToString, T: Id> crate::quad::ObjectRef<'a, J, T> {
	pub fn rdf_value<G: id::Generator<T>>(
		&self,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<CompoundValue<'a, J, T>> {
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

enum ListItemTriples<'a, J: JsonHash, T: Id> {
	NestedList(NestedListTriples<'a, J, T>),
	CompoundLiteral(CompoundLiteralTriples<T>),
}

struct NestedListTriples<'a, J: JsonHash, T: Id> {
	head_ref: Option<ValidReference<T>>,
	previous: Option<ValidReference<T>>,
	iter: std::slice::Iter<'a, Indexed<Object<J, T>>>,
}

struct ListNode<'a, 'i, J: JsonHash, T: Id> {
	id: &'i ValidReference<T>,
	object: &'a Indexed<Object<J, T>>,
}

impl<'a, J: JsonHash, T: Id> NestedListTriples<'a, J, T> {
	fn new(list: &'a [Indexed<Object<J, T>>], head_ref: ValidReference<T>) -> Self {
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
	fn next<G: id::Generator<T>>(&mut self, generator: &mut G) -> Option<ListNode<'a, '_, J, T>> {
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

pub enum CompoundValueTriples<'a, J: JsonHash, T: Id> {
	Literal(CompoundLiteralTriples<T>),
	List(ListTriples<'a, J, T>),
}

impl<'a, J: JsonHash + ToString, T: Id> CompoundValueTriples<'a, J, T> {
	pub fn with<G: id::Generator<T>>(
		self,
		generator: G,
		rdf_direction: Option<RdfDirection>,
	) -> CompoundValueTriplesWith<'a, J, T, G> {
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

pub struct CompoundValueTriplesWith<'a, J: JsonHash, T: Id, G: id::Generator<T>> {
	generator: G,
	rdf_direction: Option<RdfDirection>,
	inner: CompoundValueTriples<'a, J, T>,
}

impl<'a, J: JsonHash + ToString, T: Id, G: id::Generator<T>> Iterator
	for CompoundValueTriplesWith<'a, J, T, G>
{
	type Item = Triple<T>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next(&mut self.generator, self.rdf_direction)
	}
}

/// Iterator over the RDF quads generated from a list of JSON-LD objects.
///
/// If the list contains nested lists, the iterator will also emit quads for those nested lists.
pub struct ListTriples<'a, J: JsonHash, T: Id> {
	stack: SmallVec<[ListItemTriples<'a, J, T>; 2]>,
	pending: Option<Triple<T>>,
}

impl<'a, J: JsonHash + ToString, T: Id> ListTriples<'a, J, T> {
	pub fn new(list: &'a [Indexed<Object<J, T>>], head_ref: ValidReference<T>) -> Self {
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
	) -> ListTriplesWith<'a, J, T, G> {
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

								self.pending = Some(Triple(
									id.clone(),
									Property::Rdf(RdfSyntax::First),
									compound_value.value,
								));

								if let Some(previous_id) = previous {
									break Some(Triple(
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
								break Some(Triple(
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

pub struct ListTriplesWith<'a, J: JsonHash, T: Id, G: id::Generator<T>> {
	generator: G,
	rdf_direction: Option<RdfDirection>,
	inner: ListTriples<'a, J, T>,
}

impl<'a, J: JsonHash + ToString, T: Id, G: id::Generator<T>> Iterator
	for ListTriplesWith<'a, J, T, G>
{
	type Item = Triple<T>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next(&mut self.generator, self.rdf_direction)
	}
}

pub enum Number {
	Integer(i64),
	Double(f64),
}

static XSD_DOUBLE_CONFIG: pretty_dtoa::FmtFloatConfig = pretty_dtoa::FmtFloatConfig::default()
	.force_e_notation()
	.add_point_zero(true)
	.capitalize_e(true);

impl fmt::Display for Number {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Integer(n) => n.fmt(f),
			Self::Double(d) => {
				if d.is_nan() {
					write!(f, "NaN")
				} else if d.is_infinite() {
					if d.is_sign_positive() {
						write!(f, "INF")
					} else {
						write!(f, "-INF")
					}
				} else {
					fmt::Display::fmt(&pretty_dtoa::dtoa(*d, XSD_DOUBLE_CONFIG), f)
				}
			}
		}
	}
}

pub enum LiteralValue {
	Null,
	Bool(bool),
	Number(Number),
	String(String),
}

impl fmt::Display for LiteralValue {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Null => "\"null\"".fmt(f),
			Self::Bool(b) => write!(f, "\"{}\"", b),
			Self::Number(n) => write!(f, "\"{}\"", n),
			Self::String(s) => s.rdf_display().fmt(f),
		}
	}
}

pub enum Literal<T> {
	Untyped(LiteralValue),
	Typed(LiteralValue, LiteralType<T>),
	LangString(String, LanguageTagBuf),
}

impl<T: Id> fmt::Display for Literal<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Untyped(lit) => lit.fmt(f),
			Self::Typed(lit, ty) => write!(f, "{}^^<{}>", lit, ty.as_iri()),
			Self::LangString(s, language) => write!(f, "{}@{}", s.rdf_display(), language),
		}
	}
}

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

pub enum LiteralType<T> {
	Bool,
	Integer,
	Double,
	String,
	Json,
	I18n(Box<I18nType>),
	Other(T),
}

impl<T: Id> From<T> for LiteralType<T> {
	fn from(id: T) -> Self {
		if id.as_iri() == "http://www.w3.org/2001/XMLSchema#boolean" {
			Self::Bool
		} else if id.as_iri() == "http://www.w3.org/2001/XMLSchema#integer" {
			Self::Integer
		} else if id.as_iri() == "http://www.w3.org/2001/XMLSchema#double" {
			Self::Double
		} else if id.as_iri() == "http://www.w3.org/2001/XMLSchema#string" {
			Self::String
		} else if id.as_iri() == "://www.w3.org/1999/02/22-rdf-syntax-ns#JSON" {
			Self::Json
		} else {
			Self::Other(id)
		}
	}
}

impl<T: Id> LiteralType<T> {
	fn as_iri(&self) -> Iri {
		match self {
			Self::Bool => iri!("http://www.w3.org/2001/XMLSchema#boolean"),
			Self::Integer => iri!("http://www.w3.org/2001/XMLSchema#integer"),
			Self::Double => iri!("http://www.w3.org/2001/XMLSchema#double"),
			Self::String => iri!("http://www.w3.org/2001/XMLSchema#string"),
			Self::Json => iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#JSON"),
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
pub enum Value<T: Id> {
	Nil,
	Literal(Literal<T>),
	Reference(ValidReference<T>),
}

impl<T: Id> fmt::Display for Value<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Nil => "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>".fmt(f),
			Self::Literal(lit) => lit.fmt(f),
			Self::Reference(r) => r.rdf_display().fmt(f),
		}
	}
}
