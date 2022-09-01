use crate::{
	id, object::value, Direction, Indexed, IndexedObject, Node, Object, Reference, ValidReference,
};
use contextual::DisplayWithContext;
use iref::{AsIri, Iri, IriBuf};
use json_ld_syntax::Entry;
use json_syntax::Print;
use langtag::LanguageTagBuf;
use locspan::Meta;
use rdf_types::{IriVocabularyMut, Vocabulary};
use smallvec::SmallVec;
use static_iref::iri;
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

pub const RDF_TYPE: Iri<'static> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
pub const RDF_FIRST: Iri<'static> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#first");
pub const RDF_REST: Iri<'static> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#rest");
pub const RDF_VALUE: Iri<'static> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#value");
pub const RDF_DIRECTION: Iri<'static> =
	iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#direction");
pub const RDF_JSON: Iri<'static> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#JSON");
/// IRI of the `http://www.w3.org/1999/02/22-rdf-syntax-ns#nil` value.
pub const RDF_NIL: Iri<'static> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#nil");

pub const XSD_BOOLEAN: Iri<'static> = iri!("http://www.w3.org/2001/XMLSchema#boolean");
pub const XSD_INTEGER: Iri<'static> = iri!("http://www.w3.org/2001/XMLSchema#integer");
pub const XSD_DOUBLE: Iri<'static> = iri!("http://www.w3.org/2001/XMLSchema#double");
pub const XSD_STRING: Iri<'static> = iri!("http://www.w3.org/2001/XMLSchema#string");

pub type Triple<T, B> = rdf_types::Triple<ValidReference<T, B>, ValidReference<T, B>, Value<T, B>>;

impl<T: Clone, B: Clone> Reference<T, B> {
	fn rdf_value(&self) -> Option<Value<T, B>> {
		match self {
			Reference::Valid(id) => Some(Value::Reference(id.clone())),
			Reference::Invalid(_) => None,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum RdfDirection {
	I18nDatatype,
	CompoundLiteral,
}

pub struct CompoundLiteralTriples<T, B> {
	id: ValidReference<T, B>,
	value: Option<Value<T, B>>,
	direction: Option<Value<T, B>>,
}

impl<T: Clone, B: Clone> CompoundLiteralTriples<T, B> {
	fn next(&mut self, vocabulary: &mut impl IriVocabularyMut<T>) -> Option<Triple<T, B>> {
		if let Some(value) = self.value.take() {
			return Some(rdf_types::Triple(
				self.id.clone(),
				ValidReference::Id(vocabulary.insert(RDF_VALUE)),
				value,
			));
		}

		if let Some(direction) = self.direction.take() {
			return Some(rdf_types::Triple(
				self.id.clone(),
				ValidReference::Id(vocabulary.insert(RDF_DIRECTION)),
				direction,
			));
		}

		None
	}
}

pub struct CompoundLiteral<T, B> {
	value: Value<T, B>,
	triples: Option<CompoundLiteralTriples<T, B>>,
}

impl<T: Clone, M> crate::object::Value<T, M> {
	fn rdf_value_in<B, N: IriVocabularyMut<T>, G: id::Generator<T, B, M, N>>(
		&self,
		vocabulary: &mut N,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<CompoundLiteral<T, B>> {
		match self {
			Self::Json(json) => Some(CompoundLiteral {
				value: Value::Literal(Literal::TypedString(
					json.compact_print().to_string().into(),
					vocabulary.insert(RDF_JSON),
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
								vocabulary.insert(i18n(language, *direction).as_iri()),
							)),
							triples: None,
						}),
						Some(RdfDirection::CompoundLiteral) => {
							let Meta(id, _) = generator.next(vocabulary);
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
								vocabulary.insert(XSD_STRING),
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

						(lit, Some(vocabulary.insert(XSD_BOOLEAN)))
					}
					value::Literal::Null => ("null".to_string().into(), None),
					value::Literal::Number(n) => {
						let rdf_ty = if n.is_i64() {
							vocabulary.insert(XSD_INTEGER)
						} else {
							vocabulary.insert(XSD_DOUBLE)
						};

						(n.as_str().to_string().into(), Some(rdf_ty))
					}
					value::Literal::String(s) => (s.to_string().into(), None),
				};

				let rdf_ty = match ty {
					Some(id) => Some(id.clone()),
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

impl<T: Clone, B: Clone, M> Node<T, B, M> {
	fn rdf_value(&self) -> Option<Value<T, B>> {
		self.id_entry()
			.map(Entry::as_value)
			.map(Meta::value)
			.and_then(Reference::rdf_value)
	}
}

impl<T: Clone, B: Clone, M> Object<T, B, M> {
	fn rdf_value_in<N: IriVocabularyMut<T>, G: id::Generator<T, B, M, N>>(
		&self,
		vocabulary: &mut N,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<CompoundValue<T, B, M>> {
		match self {
			Self::Value(value) => value
				.rdf_value_in(vocabulary, generator, rdf_direction)
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
						value: Value::Reference(ValidReference::Id(vocabulary.insert(RDF_NIL))),
						triples: None,
					})
				} else {
					let Meta(id, _) = generator.next(vocabulary);
					Some(CompoundValue {
						value: Value::Reference(Clone::clone(&id)),
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

pub struct CompoundValue<'a, T, B, M> {
	value: Value<T, B>,
	triples: Option<CompoundValueTriples<'a, T, B, M>>,
}

impl<'a, T: Clone, B: Clone, M> crate::quad::ObjectRef<'a, T, B, M> {
	pub fn rdf_value_in<N: IriVocabularyMut<T>, G: id::Generator<T, B, M, N>>(
		&self,
		vocabulary: &mut N,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<CompoundValue<'a, T, B, M>> {
		match self {
			Self::Object(object) => object.rdf_value_in(vocabulary, generator, rdf_direction),
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

enum ListItemTriples<'a, T, B, M> {
	NestedList(NestedListTriples<'a, T, B, M>),
	CompoundLiteral(CompoundLiteralTriples<T, B>),
}

struct NestedListTriples<'a, T, B, M> {
	head_ref: Option<ValidReference<T, B>>,
	previous: Option<ValidReference<T, B>>,
	iter: std::slice::Iter<'a, IndexedObject<T, B, M>>,
}

struct ListNode<'a, 'i, T, B, M> {
	id: &'i ValidReference<T, B>,
	object: &'a Indexed<Object<T, B, M>, M>,
}

impl<'a, T, B, M> NestedListTriples<'a, T, B, M> {
	fn new(list: &'a [IndexedObject<T, B, M>], head_ref: ValidReference<T, B>) -> Self {
		Self {
			head_ref: Some(head_ref),
			previous: None,
			iter: list.iter(),
		}
	}

	fn previous(&self) -> Option<&ValidReference<T, B>> {
		self.previous.as_ref()
	}

	/// Pull the next object of the list.
	///
	/// Uses the given generator to assign as id to the list element.
	fn next<N, G: id::Generator<T, B, M, N>>(
		&mut self,
		vocabulary: &mut N,
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

pub enum CompoundValueTriples<'a, T, B, M> {
	Literal(CompoundLiteralTriples<T, B>),
	List(ListTriples<'a, T, B, M>),
}

impl<'a, T, B, M> CompoundValueTriples<'a, T, B, M> {
	pub fn with<'n, N, G: id::Generator<T, B, M, N>>(
		self,
		vocabulary: &'n mut N,
		generator: G,
		rdf_direction: Option<RdfDirection>,
	) -> CompoundValueTriplesWith<'a, 'n, T, B, N, M, G> {
		CompoundValueTriplesWith {
			vocabulary,
			generator,
			rdf_direction,
			inner: self,
		}
	}

	pub fn next<N: IriVocabularyMut<T>, G: id::Generator<T, B, M, N>>(
		&mut self,
		vocabulary: &mut N,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<Triple<T, B>>
	where
		T: Clone,
		B: Clone,
	{
		match self {
			Self::Literal(l) => l.next(vocabulary),
			Self::List(l) => l.next(vocabulary, generator, rdf_direction),
		}
	}
}

pub struct CompoundValueTriplesWith<'a, 'n, T, B, N, M, G: id::Generator<T, B, M, N>> {
	vocabulary: &'n mut N,
	generator: G,
	rdf_direction: Option<RdfDirection>,
	inner: CompoundValueTriples<'a, T, B, M>,
}

impl<
		'a,
		'n,
		T: AsIri + Clone,
		B: Clone,
		M,
		N: IriVocabularyMut<T>,
		G: id::Generator<T, B, M, N>,
	> Iterator for CompoundValueTriplesWith<'a, 'n, T, B, N, M, G>
{
	type Item = Triple<T, B>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner
			.next(self.vocabulary, &mut self.generator, self.rdf_direction)
	}
}

/// Iterator over the RDF quads generated from a list of JSON-LD objects.
///
/// If the list contains nested lists, the iterator will also emit quads for those nested lists.
pub struct ListTriples<'a, T, B, M> {
	stack: SmallVec<[ListItemTriples<'a, T, B, M>; 2]>,
	pending: Option<Triple<T, B>>,
}

impl<'a, T, B, M> ListTriples<'a, T, B, M> {
	pub fn new(list: &'a [IndexedObject<T, B, M>], head_ref: ValidReference<T, B>) -> Self {
		let mut stack = SmallVec::new();
		stack.push(ListItemTriples::NestedList(NestedListTriples::new(
			list, head_ref,
		)));

		Self {
			stack,
			pending: None,
		}
	}

	pub fn with<'n, N, G: id::Generator<T, B, M, N>>(
		self,
		vocabulary: &'n mut N,
		generator: G,
		rdf_direction: Option<RdfDirection>,
	) -> ListTriplesWith<'a, 'n, T, B, N, M, G> {
		ListTriplesWith {
			vocabulary,
			generator,
			rdf_direction,
			inner: self,
		}
	}

	pub fn next<N: IriVocabularyMut<T>, G: id::Generator<T, B, M, N>>(
		&mut self,
		vocabulary: &mut N,
		generator: &mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Option<Triple<T, B>>
	where
		T: Clone,
		B: Clone,
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
									.rdf_value_in(vocabulary, generator, rdf_direction)
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
									ValidReference::Id(vocabulary.insert(RDF_FIRST)),
									compound_value.value,
								));

								if let Some(previous_id) = previous {
									break Some(rdf_types::Triple(
										previous_id,
										ValidReference::Id(vocabulary.insert(RDF_REST)),
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
									ValidReference::Id(vocabulary.insert(RDF_REST)),
									Value::Reference(ValidReference::Id(
										vocabulary.insert(RDF_NIL),
									)),
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

pub struct ListTriplesWith<'a, 'n, T, B, N, M, G: id::Generator<T, B, M, N>> {
	vocabulary: &'n mut N,
	generator: G,
	rdf_direction: Option<RdfDirection>,
	inner: ListTriples<'a, T, B, M>,
}

impl<
		'a,
		'n,
		T: AsIri + Clone,
		B: Clone,
		N: IriVocabularyMut<T>,
		M,
		G: id::Generator<T, B, M, N>,
	> Iterator for ListTriplesWith<'a, 'n, T, B, N, M, G>
{
	type Item = Triple<T, B>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner
			.next(self.vocabulary, &mut self.generator, self.rdf_direction)
	}
}

pub type Literal<T> = rdf_types::Literal<rdf_types::StringLiteral, T>;

fn i18n(language: Option<LanguageTagBuf>, direction: Direction) -> IriBuf {
	let iri = match &language {
		Some(language) => format!("https://www.w3.org/ns/i18n#{}_{}", language, direction),
		None => format!("https://www.w3.org/ns/i18n#{}", direction),
	};

	IriBuf::from_string(iri).unwrap()
}

/// RDF value.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Value<T, B> {
	Literal(Literal<T>),
	Reference(ValidReference<T, B>),
}

impl<T: fmt::Display, B: fmt::Display> fmt::Display for Value<T, B> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Literal(lit) => lit.rdf_display().fmt(f),
			Self::Reference(r) => r.rdf_display().fmt(f),
		}
	}
}

impl<T, B, N: Vocabulary<T, B>> DisplayWithContext<N> for Value<T, B> {
	fn fmt_with(&self, vocabulary: &N, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Literal(lit) => lit.fmt_with(vocabulary, f),
			Self::Reference(r) => r.fmt_with(vocabulary, f),
		}
	}
}

impl<T: fmt::Display> Display for Literal<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::String(s) => s.fmt(f),
			Self::TypedString(s, ty) => write!(f, "{}^^{}", s, ty),
			Self::LangString(s, t) => write!(f, "{}@{}", s, t),
		}
	}
}
