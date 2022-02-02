use crate::{
	id, lang::LenientLanguageTagBuf, object::value, Direction, Id, Indexed, Node, Object,
	Reference, ValidReference,
};
use generic_json::{Json, JsonHash};
use smallvec::SmallVec;
use std::convert::TryFrom;

mod quad;

pub use quad::*;

/// Property reference.
pub enum PropertyRef<'a, T: Id> {
	Type,
	First,
	Rest,
	Value,
	Direction,
	Other(&'a ValidReference<T>),
}

pub enum Property<T: Id> {
	Type,
	First,
	Rest,
	Value,
	Direction,
	Other(ValidReference<T>),
}

impl<'a, T: Id> TryFrom<Property<T>> for PropertyRef<'a, T> {
	type Error = ValidReference<T>;

	fn try_from(p: Property<T>) -> Result<Self, ValidReference<T>> {
		match p {
			Property::Type => Ok(PropertyRef::Type),
			Property::First => Ok(PropertyRef::First),
			Property::Rest => Ok(PropertyRef::Rest),
			Property::Value => Ok(PropertyRef::Value),
			Property::Direction => Ok(PropertyRef::Direction),
			Property::Other(p) => Err(p),
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
			return Some(Triple(self.id.clone(), Property::Value, value));
		}

		if let Some(direction) = self.direction.take() {
			return Some(Triple(self.id.clone(), Property::Direction, direction));
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
		mut generator: G,
		rdf_direction: RdfDirection,
	) -> Option<CompoundLiteral<T>> {
		match self {
			Self::Json(json) => Some(CompoundLiteral {
				value: Value::Literal(Literal::String(json.to_string()), Some(LiteralType::Json)),
				triples: None,
			}),
			Self::LangString(lang_string) => {
				let (string, language, direction) = lang_string.parts();

				match language {
					Some(language) => match direction {
						Some(direction) => match rdf_direction {
							RdfDirection::I18nDatatype => Some(CompoundLiteral {
								value: Value::Literal(
									Literal::String(string.to_string()),
									Some(LiteralType::I18n(language.clone(), *direction)),
								),
								triples: None,
							}),
							RdfDirection::CompoundLiteral => {
								let id = generator.next();
								Some(CompoundLiteral {
									value: Value::Reference(id),
									triples: None,
								})
							}
						},
						None => Some(CompoundLiteral {
							value: Value::LangString(string.to_string(), language.clone()),
							triples: None,
						}),
					},
					None => Some(CompoundLiteral {
						value: Value::Literal(
							Literal::String(string.to_string()),
							Some(LiteralType::String),
						),
						triples: None,
					}),
				}
			}
			Self::Literal(lit, ty) => {
				let (rdf_lit, prefered_rdf_ty) = match lit {
					value::Literal::Boolean(b) => (Literal::Bool(*b), Some(LiteralType::Bool)),
					value::Literal::Null => (Literal::Null, None),
					value::Literal::Number(n) => {
						use generic_json::Number as JsonNumber;

						let (rdf_number, rdf_ty) = if let Some(n) = n.as_u64() {
							(Number::Integer(n), LiteralType::Integer)
						} else if let Some(n) = n.as_u32() {
							(Number::Integer(n as u64), LiteralType::Integer)
						} else if let Some(f) = n.as_f64() {
							(Number::Double(f), LiteralType::Double)
						} else if let Some(f) = n.as_f32() {
							(Number::Double(f as f64), LiteralType::Double)
						} else {
							(Number::Double(n.as_f64_lossy()), LiteralType::Double)
						};

						(Literal::Number(rdf_number), Some(rdf_ty))
					}
					value::Literal::String(s) => (Literal::String(s.to_string()), None),
				};

				let rdf_ty = match ty {
					Some(id) => Some(LiteralType::Other(id.clone())),
					None => prefered_rdf_ty,
				};

				let rdf_lit = match (rdf_lit, &rdf_ty) {
					(Literal::Number(Number::Integer(i)), Some(LiteralType::Double)) => {
						Literal::Number(Number::Double(i as f64))
					}
					(rdf_lit, _) => rdf_lit,
				};

				Some(CompoundLiteral {
					value: Value::Literal(rdf_lit, rdf_ty),
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
		mut generator: G,
		rdf_direction: RdfDirection,
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
		generator: G,
		rdf_direction: RdfDirection,
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
		rdf_direction: RdfDirection,
	) -> CompoundValueTriplesWith<'a, J, T, G> {
		CompoundValueTriplesWith {
			generator,
			rdf_direction,
			inner: self,
		}
	}

	pub fn next<G: id::Generator<T>>(
		&mut self,
		generator: G,
		rdf_direction: RdfDirection,
	) -> Option<Triple<T>> {
		match self {
			Self::Literal(l) => l.next(),
			Self::List(l) => l.next(generator, rdf_direction),
		}
	}
}

pub struct CompoundValueTriplesWith<'a, J: JsonHash, T: Id, G: id::Generator<T>> {
	generator: G,
	rdf_direction: RdfDirection,
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
		rdf_direction: RdfDirection,
	) -> ListTriplesWith<'a, J, T, G> {
		ListTriplesWith {
			generator,
			rdf_direction,
			inner: self,
		}
	}

	pub fn next<G: id::Generator<T>>(
		&mut self,
		mut generator: G,
		rdf_direction: RdfDirection,
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
					match list.next(&mut generator) {
						Some(node) => {
							if let Some(compound_value) =
								node.object.rdf_value(&mut generator, rdf_direction)
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

								self.pending =
									Some(Triple(id.clone(), Property::First, compound_value.value));

								if let Some(previous_id) = previous {
									break Some(Triple(
										previous_id,
										Property::Rest,
										Value::Reference(id),
									));
								}
							}
						}
						None => {
							if let Some(previous_id) = previous {
								break Some(Triple(previous_id, Property::Rest, Value::Nil));
							}

							self.stack.pop();
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
	rdf_direction: RdfDirection,
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
	Integer(u64),
	Double(f64),
}

pub enum Literal {
	Null,
	Bool(bool),
	Number(Number),
	String(String),
}

pub enum LiteralType<T> {
	Bool,
	Integer,
	Double,
	String,
	Json,
	I18n(LenientLanguageTagBuf, Direction),
	Other(T),
}

/// RDF value.
pub enum Value<T: Id> {
	Nil,
	Literal(Literal, Option<LiteralType<T>>),
	LangString(String, LenientLanguageTagBuf),
	Reference(ValidReference<T>),
}
