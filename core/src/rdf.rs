use std::str::FromStr;

use crate::{id, object::value, Direction, Id, Indexed, IndexedObject, Node, Object, ValidId};
use iref::{Iri, IriBuf};
use json_ld_syntax::Entry;
use json_syntax::Print;
use langtag::LanguageTagBuf;
use locspan::Meta;
use rdf_types::{
	IriVocabularyMut, LanguageTagVocabulary, LanguageTagVocabularyMut, LiteralVocabularyMut,
	Vocabulary,
};
use smallvec::SmallVec;
use static_iref::iri;

mod quad;
pub use quad::*;

pub const RDF_TYPE: &'static Iri = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
pub const RDF_FIRST: &'static Iri = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#first");
pub const RDF_REST: &'static Iri = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#rest");
pub const RDF_VALUE: &'static Iri = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#value");
pub const RDF_DIRECTION: &'static Iri =
	iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#direction");
pub const RDF_JSON: &'static Iri = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#JSON");
/// IRI of the `http://www.w3.org/1999/02/22-rdf-syntax-ns#nil` value.
pub const RDF_NIL: &'static Iri = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#nil");

pub const XSD_BOOLEAN: &'static Iri = iri!("http://www.w3.org/2001/XMLSchema#boolean");
pub const XSD_INTEGER: &'static Iri = iri!("http://www.w3.org/2001/XMLSchema#integer");
pub const XSD_DOUBLE: &'static Iri = iri!("http://www.w3.org/2001/XMLSchema#double");
pub const XSD_STRING: &'static Iri = iri!("http://www.w3.org/2001/XMLSchema#string");

/// JSON-LD to RDF triple.
pub type Triple<T, B, L> = rdf_types::Triple<ValidId<T, B>, ValidId<T, B>, Value<T, B, L>>;

impl<T: Clone, B: Clone> Id<T, B> {
	fn rdf_value<L>(&self) -> Option<Value<T, B, L>> {
		match self {
			Id::Valid(id) => Some(Value::Id(id.clone())),
			Id::Invalid(_) => None,
		}
	}
}

/// Direction representation method.
///
/// Used by the RDF serializer to decide how to encode
/// [`Direction`](crate::Direction)s.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum RdfDirection {
	/// Encode direction in the string value type IRI using the
	/// `https://www.w3.org/ns/i18n#` prefix.
	///
	/// If a language tag is present the IRI will be of the form
	/// `https://www.w3.org/ns/i18n#language_direction` or simply
	/// `https://www.w3.org/ns/i18n#direction` otherwise where `direction` is
	/// either `rtl` or `ltr`.
	I18nDatatype,

	/// Encode the direction using a compound literal value.
	///
	/// In this case the direction tagged string is encoded with a fresh blank
	/// node identifier `_:b` and the following triples:
	/// ```nquads
	/// _:b http://www.w3.org/1999/02/22-rdf-syntax-ns#value value@language
	/// _:b http://www.w3.org/1999/02/22-rdf-syntax-ns#direction direction
	/// ```
	/// where `direction` is either `rtl` or `ltr`.
	CompoundLiteral,
}

#[derive(Debug, Clone)]
pub struct InvalidRdfDirection(pub String);

impl FromStr for RdfDirection {
	type Err = InvalidRdfDirection;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"i18n-datatype" => Ok(Self::I18nDatatype),
			"compound-literal" => Ok(Self::CompoundLiteral),
			_ => Err(InvalidRdfDirection(s.to_string())),
		}
	}
}

impl<'a> TryFrom<&'a str> for RdfDirection {
	type Error = InvalidRdfDirection;

	fn try_from(value: &'a str) -> Result<Self, Self::Error> {
		value.parse()
	}
}

/// Iterator over the triples of a compound literal representing a language
/// tagged string with direction.
pub struct CompoundLiteralTriples<T, B, L> {
	/// Compound literal identifier.
	id: ValidId<T, B>,

	/// String value.
	value: Option<Value<T, B, L>>,

	/// Direction value.
	direction: Option<Value<T, B, L>>,
}

impl<T: Clone, B: Clone, L: Clone> CompoundLiteralTriples<T, B, L> {
	fn next(&mut self, vocabulary: &mut impl IriVocabularyMut<Iri = T>) -> Option<Triple<T, B, L>> {
		if let Some(value) = self.value.take() {
			return Some(rdf_types::Triple(
				self.id.clone(),
				ValidId::Iri(vocabulary.insert(RDF_VALUE)),
				value,
			));
		}

		if let Some(direction) = self.direction.take() {
			return Some(rdf_types::Triple(
				self.id.clone(),
				ValidId::Iri(vocabulary.insert(RDF_DIRECTION)),
				direction,
			));
		}

		None
	}
}

/// Compound literal.
pub struct CompoundLiteral<T, B, L> {
	value: Value<T, B, L>,
	triples: Option<CompoundLiteralTriples<T, B, L>>,
}

impl<T: Clone, M> crate::object::Value<T, M> {
	fn rdf_value_with<
		V: Vocabulary<Iri = T> + IriVocabularyMut + LanguageTagVocabularyMut,
		G: id::Generator<V, M>,
	>(
		&self,
		vocabulary: &mut V,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<CompoundLiteral<T, V::BlankId, V::Literal>>
	where
		V: LiteralVocabularyMut<
			Type = rdf_types::literal::Type<V::Iri, V::LanguageTag>,
			Value = String,
		>,
	{
		match self {
			Self::Json(json) => {
				let ty = vocabulary.insert(RDF_JSON);
				Some(CompoundLiteral {
					value: Value::Literal(vocabulary.insert_owned_literal(Literal::new(
						json.compact_print().to_string(),
						rdf_types::literal::Type::Any(ty),
					))),
					triples: None,
				})
			}
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
						Some(RdfDirection::I18nDatatype) => {
							let ty = vocabulary.insert(i18n(language, *direction).as_iri());
							Some(CompoundLiteral {
								value: Value::Literal(vocabulary.insert_owned_literal(
									Literal::new(
										string.to_string(),
										rdf_types::literal::Type::Any(ty),
									),
								)),
								triples: None,
							})
						}
						Some(RdfDirection::CompoundLiteral) => {
							let Meta(id, _) = generator.next(vocabulary);
							Some(CompoundLiteral {
								value: id.into_term(),
								triples: None,
							})
						}
						None => match language {
							Some(language) => {
								let tag = vocabulary.insert_owned_language_tag(language);
								Some(CompoundLiteral {
									value: Value::Literal(vocabulary.insert_owned_literal(
										Literal::new(
											string.to_string(),
											rdf_types::literal::Type::LangString(tag),
										),
									)),
									triples: None,
								})
							}
							None => {
								let ty = vocabulary.insert(XSD_STRING);
								Some(CompoundLiteral {
									value: Value::Literal(vocabulary.insert_owned_literal(
										Literal::new(
											string.to_string(),
											rdf_types::literal::Type::Any(ty),
										),
									)),
									triples: None,
								})
							}
						},
					},
					None => match language {
						Some(language) => {
							let tag = vocabulary.insert_owned_language_tag(language);
							Some(CompoundLiteral {
								value: Value::Literal(vocabulary.insert_owned_literal(
									Literal::new(
										string.to_string(),
										rdf_types::literal::Type::LangString(tag),
									),
								)),
								triples: None,
							})
						}
						None => {
							let ty = vocabulary.insert(XSD_STRING);
							Some(CompoundLiteral {
								value: Value::Literal(vocabulary.insert_owned_literal(
									Literal::new(
										string.to_string(),
										rdf_types::literal::Type::Any(ty),
									),
								)),
								triples: None,
							})
						}
					},
				}
			}
			Self::Literal(lit, ty) => {
				let (rdf_lit, prefered_rdf_ty) = match lit {
					value::Literal::Boolean(b) => {
						let lit = if *b {
							"true".to_string()
						} else {
							"false".to_string()
						};

						(lit, Some(vocabulary.insert(XSD_BOOLEAN)))
					}
					value::Literal::Null => ("null".to_string(), None),
					value::Literal::Number(n) => {
						if n.is_i64()
							&& !ty
								.as_ref()
								.map(|t| vocabulary.iri(t).unwrap() == XSD_DOUBLE)
								.unwrap_or(false)
						{
							(n.to_string(), Some(vocabulary.insert(XSD_INTEGER)))
						} else {
							(
								pretty_dtoa::dtoa(n.as_f64_lossy(), XSD_CANONICAL_FLOAT),
								Some(vocabulary.insert(XSD_DOUBLE)),
							)
						}
					}
					value::Literal::String(s) => (s.to_string(), None),
				};

				let rdf_ty = match ty {
					Some(id) => Some(id.clone()),
					None => prefered_rdf_ty,
				};

				Some(CompoundLiteral {
					value: match rdf_ty {
						Some(ty) => Value::Literal(vocabulary.insert_owned_literal(Literal::new(
							rdf_lit,
							rdf_types::literal::Type::Any(ty),
						))),
						None => {
							let ty = vocabulary.insert(XSD_STRING);
							Value::Literal(vocabulary.insert_owned_literal(Literal::new(
								rdf_lit,
								rdf_types::literal::Type::Any(ty),
							)))
						}
					},
					triples: None,
				})
			}
		}
	}
}

// <https://www.w3.org/TR/xmlschema11-2/#f-doubleLexmap>
const XSD_CANONICAL_FLOAT: pretty_dtoa::FmtFloatConfig = pretty_dtoa::FmtFloatConfig::default()
	.force_e_notation()
	.capitalize_e(true);

impl<T: Clone, B: Clone, M> Node<T, B, M> {
	fn rdf_value<L>(&self) -> Option<Value<T, B, L>> {
		self.id_entry()
			.map(Entry::as_value)
			.map(Meta::value)
			.and_then(Id::rdf_value)
	}
}

impl<T: Clone, B: Clone, M> Object<T, B, M> {
	fn rdf_value_with<
		V: Vocabulary<Iri = T, BlankId = B> + IriVocabularyMut + LanguageTagVocabularyMut,
		G: id::Generator<V, M>,
	>(
		&self,
		vocabulary: &mut V,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<CompoundValue<T, B, V::Literal, M>>
	where
		V: LiteralVocabularyMut<
			Type = rdf_types::literal::Type<V::Iri, V::LanguageTag>,
			Value = String,
		>,
	{
		match self {
			Self::Value(value) => value
				.rdf_value_with(vocabulary, generator, rdf_direction)
				.map(|compound_value| CompoundValue {
					value: compound_value.value,
					triples: compound_value.triples.map(CompoundValueTriples::literal),
				}),
			Self::Node(node) => node.rdf_value().map(|value| CompoundValue {
				value,
				triples: None,
			}),
			Self::List(list) => {
				if list.is_empty() {
					Some(CompoundValue {
						value: Value::Id(ValidId::Iri(vocabulary.insert(RDF_NIL))),
						triples: None,
					})
				} else {
					let Meta(id, _) = generator.next(vocabulary);
					Some(CompoundValue {
						value: Clone::clone(&id).into_term(),
						triples: Some(CompoundValueTriples::List(ListTriples::new(
							list.as_slice(),
							id,
						))),
					})
				}
			}
		}
	}
}

pub struct CompoundValue<'a, T, B, L, M> {
	value: Value<T, B, L>,
	triples: Option<CompoundValueTriples<'a, T, B, L, M>>,
}

impl<'a, T: Clone, B: Clone, M> crate::quad::ObjectRef<'a, T, B, M> {
	pub fn rdf_value_with<
		V: Vocabulary<Iri = T, BlankId = B> + IriVocabularyMut + LanguageTagVocabularyMut,
		G: id::Generator<V, M>,
	>(
		&self,
		vocabulary: &mut V,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<CompoundValue<'a, T, B, V::Literal, M>>
	where
		V: LiteralVocabularyMut<
			Type = rdf_types::literal::Type<V::Iri, V::LanguageTag>,
			Value = String,
		>,
	{
		match self {
			Self::Object(object) => object.rdf_value_with(vocabulary, generator, rdf_direction),
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

enum ListItemTriples<'a, T, B, L, M> {
	NestedList(NestedListTriples<'a, T, B, M>),
	CompoundLiteral(Box<CompoundLiteralTriples<T, B, L>>),
}

struct NestedListTriples<'a, T, B, M> {
	head_ref: Option<ValidId<T, B>>,
	previous: Option<ValidId<T, B>>,
	iter: std::slice::Iter<'a, IndexedObject<T, B, M>>,
}

struct ListNode<'a, 'i, T, B, M> {
	id: &'i ValidId<T, B>,
	object: &'a Indexed<Object<T, B, M>, M>,
}

impl<'a, T, B, M> NestedListTriples<'a, T, B, M> {
	fn new(list: &'a [IndexedObject<T, B, M>], head_ref: ValidId<T, B>) -> Self {
		Self {
			head_ref: Some(head_ref),
			previous: None,
			iter: list.iter(),
		}
	}

	fn previous(&self) -> Option<&ValidId<T, B>> {
		self.previous.as_ref()
	}

	/// Pull the next object of the list.
	///
	/// Uses the given generator to assign as id to the list element.
	fn next<V: Vocabulary<Iri = T, BlankId = B>, G: id::Generator<V, M>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut G,
	) -> Option<ListNode<'a, '_, T, B, M>> {
		if let Some(next) = self.iter.next() {
			let id = match self.head_ref.take() {
				Some(id) => id,
				None => generator.next(vocabulary).into_value(),
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

pub enum CompoundValueTriples<'a, T, B, L, M> {
	Literal(Box<CompoundLiteralTriples<T, B, L>>),
	List(ListTriples<'a, T, B, L, M>),
}

impl<'a, T, B, L, M> CompoundValueTriples<'a, T, B, L, M> {
	pub fn literal(l: CompoundLiteralTriples<T, B, L>) -> Self {
		Self::Literal(Box::new(l))
	}

	pub fn with<
		'n,
		V: Vocabulary<Iri = T, BlankId = B, Literal = L> + LanguageTagVocabulary,
		G: id::Generator<V, M>,
	>(
		self,
		vocabulary: &'n mut V,
		generator: G,
		rdf_direction: Option<RdfDirection>,
	) -> CompoundValueTriplesWith<'a, 'n, V, M, G> {
		CompoundValueTriplesWith {
			vocabulary,
			generator,
			rdf_direction,
			inner: self,
		}
	}

	pub fn next<
		V: Vocabulary<Iri = T, BlankId = B, Literal = L> + IriVocabularyMut + LanguageTagVocabularyMut,
		G: id::Generator<V, M>,
	>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<Triple<T, B, L>>
	where
		T: Clone,
		B: Clone,
		L: Clone,
		V: LiteralVocabularyMut<Type = rdf_types::literal::Type<T, V::LanguageTag>, Value = String>,
	{
		match self {
			Self::Literal(l) => l.next(vocabulary),
			Self::List(l) => l.next(vocabulary, generator, rdf_direction),
		}
	}
}

pub struct CompoundValueTriplesWith<
	'a,
	'n,
	N: Vocabulary + LanguageTagVocabulary,
	M,
	G: id::Generator<N, M>,
> {
	vocabulary: &'n mut N,
	generator: G,
	rdf_direction: Option<RdfDirection>,
	inner: CompoundValueTriples<'a, N::Iri, N::BlankId, N::Literal, M>,
}

impl<
		'a,
		'n,
		M,
		N: Vocabulary + IriVocabularyMut + LanguageTagVocabularyMut,
		G: id::Generator<N, M>,
	> Iterator for CompoundValueTriplesWith<'a, 'n, N, M, G>
where
	N::Iri: AsRef<Iri> + Clone,
	N::BlankId: Clone,
	N::Literal: Clone,
	N: LiteralVocabularyMut<
		Type = rdf_types::literal::Type<N::Iri, N::LanguageTag>,
		Value = String,
	>,
{
	type Item = Triple<N::Iri, N::BlankId, N::Literal>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner
			.next(self.vocabulary, &mut self.generator, self.rdf_direction)
	}
}

/// Iterator over the RDF quads generated from a list of JSON-LD objects.
///
/// If the list contains nested lists, the iterator will also emit quads for those nested lists.
pub struct ListTriples<'a, T, B, L, M> {
	stack: SmallVec<[ListItemTriples<'a, T, B, L, M>; 2]>,
	pending: Option<Triple<T, B, L>>,
}

impl<'a, T, B, L, M> ListTriples<'a, T, B, L, M> {
	pub fn new(list: &'a [IndexedObject<T, B, M>], head_ref: ValidId<T, B>) -> Self {
		let mut stack = SmallVec::new();
		stack.push(ListItemTriples::NestedList(NestedListTriples::new(
			list, head_ref,
		)));

		Self {
			stack,
			pending: None,
		}
	}

	pub fn with<
		'n,
		V: Vocabulary<Iri = T, BlankId = B, Literal = L> + LanguageTagVocabulary,
		G: id::Generator<V, M>,
	>(
		self,
		vocabulary: &'n mut V,
		generator: G,
		rdf_direction: Option<RdfDirection>,
	) -> ListTriplesWith<'a, 'n, V, M, G> {
		ListTriplesWith {
			vocabulary,
			generator,
			rdf_direction,
			inner: self,
		}
	}

	pub fn next<
		V: Vocabulary<Iri = T, BlankId = B, Literal = L> + IriVocabularyMut + LanguageTagVocabularyMut,
		G: id::Generator<V, M>,
	>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<Triple<T, B, L>>
	where
		T: Clone,
		B: Clone,
		L: Clone,
		V: LiteralVocabularyMut<
			Type = rdf_types::literal::Type<V::Iri, V::LanguageTag>,
			Value = String,
		>,
	{
		loop {
			if let Some(pending) = self.pending.take() {
				break Some(pending);
			}

			match self.stack.last_mut() {
				Some(ListItemTriples::CompoundLiteral(lit)) => match lit.next(vocabulary) {
					Some(triple) => break Some(triple),
					None => {
						self.stack.pop();
					}
				},
				Some(ListItemTriples::NestedList(list)) => {
					let previous = list.previous().cloned();
					match list.next(vocabulary, generator) {
						Some(node) => {
							if let Some(compound_value) =
								node.object
									.rdf_value_with(vocabulary, generator, rdf_direction)
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
									ValidId::Iri(vocabulary.insert(RDF_FIRST)),
									compound_value.value,
								));

								if let Some(previous_id) = previous {
									break Some(rdf_types::Triple(
										previous_id,
										ValidId::Iri(vocabulary.insert(RDF_REST)),
										id.into_term(),
									));
								}
							}
						}
						None => {
							self.stack.pop();
							if let Some(previous_id) = previous {
								break Some(rdf_types::Triple(
									previous_id,
									ValidId::Iri(vocabulary.insert(RDF_REST)),
									Value::Id(ValidId::Iri(vocabulary.insert(RDF_NIL))),
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

pub struct ListTriplesWith<'a, 'n, V: Vocabulary + LanguageTagVocabulary, M, G: id::Generator<V, M>>
{
	vocabulary: &'n mut V,
	generator: G,
	rdf_direction: Option<RdfDirection>,
	inner: ListTriples<'a, V::Iri, V::BlankId, V::Literal, M>,
}

impl<
		'a,
		'n,
		N: Vocabulary + IriVocabularyMut + LanguageTagVocabularyMut,
		M,
		G: id::Generator<N, M>,
	> Iterator for ListTriplesWith<'a, 'n, N, M, G>
where
	N::Iri: AsRef<Iri> + Clone,
	N::BlankId: Clone,
	N::Literal: Clone,
	N: LiteralVocabularyMut<
		Type = rdf_types::literal::Type<N::Iri, N::LanguageTag>,
		Value = String,
	>,
{
	type Item = Triple<N::Iri, N::BlankId, N::Literal>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner
			.next(self.vocabulary, &mut self.generator, self.rdf_direction)
	}
}

pub type Literal<T, L> = rdf_types::Literal<rdf_types::literal::Type<T, L>>;

fn i18n(language: Option<LanguageTagBuf>, direction: Direction) -> IriBuf {
	let iri = match &language {
		Some(language) => format!("https://www.w3.org/ns/i18n#{language}_{direction}"),
		None => format!("https://www.w3.org/ns/i18n#{direction}"),
	};

	IriBuf::new(iri).unwrap()
}

pub type Value<T, B, L> = rdf_types::Object<ValidId<T, B>, L>;
