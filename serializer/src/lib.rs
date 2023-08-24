use std::hash::Hash;

use indexmap::IndexSet;
use locspan::Meta;
use rdf_types::{Term, literal, LanguageTagVocabulary, IriVocabularyMut};
use serde_ld::{SerializeLd, rdf_types::Vocabulary, LexicalRepresentation, RdfLiteral};
use json_ld_core::{ExpandedDocument, Indexed, Node, Object, Value, LangString, object::Literal};
use xsd_types::XsdDatatype;

pub enum Error {
	InvalidGraph,
	InvalidPredicate
}

/// Serialize the given Linked-Data value into a JSON-LD document.
pub fn serialize(
	value: &impl SerializeLd
) -> Result<ExpandedDocument, Error> {
	serialize_with(&mut (), &mut (), value)
}

/// Serialize the given Linked-Data value into a JSON-LD document.
pub fn serialize_with<V: Vocabulary, I>(
	vocabulary: &mut V,
	interpretation: &mut I,
	value: &impl SerializeLd<V, I>
) -> Result<ExpandedDocument<V::Iri, V::BlankId>, Error>
where
	V: IriVocabularyMut,
	V::Iri: Eq + Hash,
	V::BlankId: Eq + Hash
{
	let serializer = SerializeExpandedDocument {
		vocabulary,
		interpretation,
		result: ExpandedDocument::new()
	};

	value.serialize(serializer)
}

pub struct SerializeExpandedDocument<'a, V: Vocabulary, I> {
	vocabulary: &'a mut V,
	interpretation: &'a mut I,
	result: ExpandedDocument<V::Iri, V::BlankId>
}

impl<'a, V: Vocabulary, I> serde_ld::Serializer<V, I> for SerializeExpandedDocument<'a, V, I>
where
	V: IriVocabularyMut,
	V::Iri: Eq + Hash,
	V::BlankId: Eq + Hash
{
	type Ok = ExpandedDocument<V::Iri, V::BlankId>;
	type Error = Error;

	fn insert_default<T>(&mut self, value: &T) -> Result<(), Self::Error>
		where
			T: ?Sized + serde_ld::SerializeGraph<V, I>
	{
		let serializer = SerializeDefaultGraph {
			vocabulary: self.vocabulary,
			interpretation: self.interpretation,
			result: &mut self.result
		};

		value.serialize_graph(serializer)
	}

	fn insert<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + serde_ld::LexicalRepresentation<V, I> + serde_ld::SerializeGraph<V, I>
	{
		let node = match value.lexical_representation(self.interpretation, self.vocabulary) {
			Some(Term::Literal(_)) => return Err(Error::InvalidGraph),
			Some(Term::Id(id)) => Node::with_id(json_ld_core::Id::Valid(id)),
			None => Node::new()
		};

		let serializer = SerializeGraph {
			vocabulary: self.vocabulary,
			interpretation: self.interpretation,
			result: IndexSet::new()
		};

		let graph = value.serialize_graph(serializer)?;

		node.set_graph(Some(graph));
		self.result.insert(Meta::none(Indexed::new(
			Object::node(node),
			None
		)));

		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.result)
	}
}

pub struct SerializeDefaultGraph<'a, V: Vocabulary, I> {
	vocabulary: &'a mut V,
	interpretation: &'a mut I,
	result: &'a mut ExpandedDocument<V::Iri, V::BlankId>
}

impl<'a, V: Vocabulary, I> serde_ld::GraphSerializer<V, I> for SerializeDefaultGraph<'a, V, I>
where
	V: IriVocabularyMut,
	V::Iri: Eq + Hash,
	V::BlankId: Eq + Hash
{
	type Ok = ();
	type Error = Error;

	fn insert<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LexicalRepresentation<V, I> + serde_ld::SerializeSubject<V, I>
	{
		let node = match value.lexical_representation(self.interpretation, self.vocabulary) {
			Some(Term::Literal(lit)) => {
				let value = literal_to_value(self.vocabulary, lit);
				self.result.insert(Meta::none(Indexed::new(Object::Value(value), None)));
				return Ok(())
			}
			Some(Term::Id(id)) => {
				Node::with_id(json_ld_core::Id::Valid(id))
			},
			None => Node::new()
		};

		let serializer = SerializeNode {
			vocabulary: self.vocabulary,
			interpretation: self.interpretation,
			result: node
		};

		let node = value.serialize_subject(serializer)?;
		self.result.insert(Meta::none(Indexed::new(Object::node(node), None)));
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(())
	}
}

pub struct SerializeNode<'a, V: Vocabulary, I> {
	vocabulary: &'a mut V,
	interpretation: &'a mut I,
	result: Node<V::Iri, V::BlankId>
}

impl<'a, V: Vocabulary, I> serde_ld::SubjectSerializer<V, I> for SerializeNode<'a, V, I>
where
	V::Iri: Eq + Hash,
	V::BlankId: Eq + Hash
{
	type Ok = Node<V::Iri, V::BlankId>;
	type Error = Error;

	fn insert<L, T>(&mut self, predicate: &L, value: &T) -> Result<(), Self::Error>
	where
		L: ?Sized + LexicalRepresentation<V, I>,
		T: ?Sized + serde_ld::SerializePredicate<V, I>
	{
		let prop = match predicate.lexical_representation(self.interpretation, self.vocabulary) {
			Some(Term::Id(id)) => json_ld_core::Id::Valid(id),
			_ => return Err(Error::InvalidPredicate),
		};

		let serializer = SerializeObjects {
			vocabulary: self.vocabulary,
			interpretation: self.interpretation,
			result: Vec::new()
		};

		let objects = value.serialize_predicate(serializer)?;
		self.result.insert_all(prop, objects);

		Ok(()) 
	}

	fn graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LexicalRepresentation<V, I> + serde_ld::SerializeGraph<V, I>
	{
		todo!()
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.result)
	}
}

fn literal_to_value<V: IriVocabularyMut + LanguageTagVocabulary>(
	vocabulary: &mut V,
	lit: RdfLiteral<V>
) -> Value<V::Iri> {
	match lit {
		RdfLiteral::Any(s, ty) => {
			match ty {
				literal::Type::Any(iri) => {
					Value::Literal(
						Literal::String(s.into()),
						Some(iri)
					)
				}
				literal::Type::LangString(lang) => {
					let language = vocabulary.owned_language_tag(lang).ok().unwrap();
					Value::LangString(LangString::new(
						s.into(),
						Some(language.into()),
						None
					).unwrap())
				}
			}
		}
		RdfLiteral::Xsd(xsd) => {
			xsd_to_value(vocabulary, xsd)
		}
		RdfLiteral::Json(json) => {
			Value::Json(Meta::none(json))
		}
	}
}

fn xsd_to_value<V: IriVocabularyMut>(
	vocabulary: &mut V,
	value: xsd_types::Value
) -> Value<V::Iri> {
	let ty = value.type_();
	let number = match value {
		xsd_types::Value::Boolean(b) => {
			return Value::Literal(Literal::Boolean(b), None)
		}
		xsd_types::Value::String(s) => {
			return Value::Literal(Literal::String(s.into()), None)
		}
		xsd_types::Value::Decimal(v) => {
			v.to_string()
		}
		xsd_types::Value::Integer(v) => {
			v.to_string()
		}
		xsd_types::Value::NonPositiveInteger(v) => {
			v.to_string()
		}
		xsd_types::Value::NegativeInteger(v) => {
			v.to_string()
		}
		xsd_types::Value::Long(v) => {
			v.to_string()
		}
		xsd_types::Value::Int(v) => {
			v.to_string()
		}
		xsd_types::Value::Short(v) => {
			v.to_string()
		}
		xsd_types::Value::Byte(v) => {
			v.to_string()
		}
		xsd_types::Value::NonNegativeInteger(v) => {
			v.to_string()
		}
		xsd_types::Value::UnsignedLong(v) => {
			v.to_string()
		}
		xsd_types::Value::UnsignedInt(v) => {
			v.to_string()
		}
		xsd_types::Value::UnsignedShort(v) => {
			v.to_string()
		}
		xsd_types::Value::UnsignedByte(v) => {
			v.to_string()
		}
		xsd_types::Value::PositiveInteger(v) => {
			v.to_string()
		}
		other => {
			let ty = vocabulary.insert(ty.iri());
			return Value::Literal(
				Literal::String(other.to_string().into()),
				Some(ty)
			)
		}
	};

	match json_syntax::Number::new(&number) {
		Ok(_) => {
			let n = unsafe {
				json_syntax::NumberBuf::new_unchecked(number.into_bytes().into())
			};
			Value::Literal(Literal::Number(n), None)
		},
		Err(_) => {
			let ty = vocabulary.insert(ty.iri());
			Value::Literal(
				Literal::String(number.into()),
				Some(ty)
			)
		}
	}
}