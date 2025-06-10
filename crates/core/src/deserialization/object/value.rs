use linked_data::{
	xsd_types, CowRdfTerm, LinkedData, LinkedDataGraph, LinkedDataPredicateObjects,
	LinkedDataResource, LinkedDataSubject, RdfLiteral, RdfLiteralRef, ResourceInterpretation,
};
use rdf_types::{Interpretation, LiteralTypeRef, Term, Vocabulary};

use crate::{
	object::Literal,
	rdf::{XSD_DOUBLE, XSD_INTEGER, XSD_UNSIGNED_INT},
	Value,
};

impl<V: Vocabulary, I: Interpretation> LinkedDataResource<I, V> for Value<V::Iri> {
	fn interpretation(
		&self,
		vocabulary: &mut V,
		_interpretation: &mut I,
	) -> ResourceInterpretation<I, V> {
		log::info!("Found interpretation");
		let term = match self {
			Self::Literal(l, ty) => match l {
				Literal::Null => CowRdfTerm::Owned(Term::Literal(RdfLiteral::Xsd(
					xsd_types::Value::String("null".to_string()),
				))),
				Literal::Boolean(b) => CowRdfTerm::Owned(Term::Literal(RdfLiteral::Xsd(
					xsd_types::Value::Boolean((*b).into()),
				))),
				Literal::Number(n) => {
					#[derive(Clone, Copy, Default, PartialEq)]
					enum NumericType {
						Integer,
						UnsignedInt,
						Double,
						#[default]
						Unknown,
					}

					impl NumericType {
						pub fn matches(self, other: Self) -> bool {
							self == other || self == Self::Unknown
						}
					}

					let ty = ty
						.as_ref()
						.and_then(|t| vocabulary.iri(t))
						.map(|iri| {
							log::info!("Parsing Numeric Type: {iri:?}");

							if iri == XSD_INTEGER {
								NumericType::Integer
							} else if iri == XSD_DOUBLE {
								NumericType::Double
							} else if iri == XSD_UNSIGNED_INT {
								log::info!("Found Unsigned Integer");
								NumericType::UnsignedInt
							} else {
								NumericType::Unknown
							}
						})
						.unwrap_or_default();

					let value = match n.as_u64() {
						Some(u) if ty.matches(NumericType::UnsignedInt) => {
							if u <= u32::MAX as u64 {
								xsd_types::Value::UnsignedInt(u as u32)
							} else {
								// If the value is too large for u32, fall back to double
								xsd_types::Value::Double(n.as_f64_lossy().into())
							}
						}
						_ => match n.as_i64() {
							Some(i) if ty.matches(NumericType::Integer) => {
								xsd_types::Value::Integer(i.into())
							}
							_ => xsd_types::Value::Double(n.as_f64_lossy().into()),
						},
					};

					CowRdfTerm::Owned(Term::Literal(RdfLiteral::Xsd(value)))
				}
				Literal::String(s) => CowRdfTerm::Borrowed(Term::Literal(match ty {
					Some(ty) => RdfLiteralRef::Any(s.as_str(), LiteralTypeRef::Any(ty)),
					None => RdfLiteralRef::Xsd(xsd_types::ValueRef::String(s)),
				})),
			},
			Self::LangString(s) => match s.language().and_then(|l| l.as_well_formed()) {
				Some(tag) => CowRdfTerm::Owned(Term::Literal(RdfLiteral::Any(
					s.as_str().to_owned(),
					rdf_types::LiteralType::LangString(tag.to_owned()),
				))),
				None => CowRdfTerm::Borrowed(Term::Literal(RdfLiteralRef::Xsd(
					xsd_types::ValueRef::String(s.as_str()),
				))),
			},
			Self::Json(json) => CowRdfTerm::Borrowed(Term::Literal(RdfLiteralRef::Json(json))),
		};

		ResourceInterpretation::Uninterpreted(Some(term))
	}
}

impl<T, V: Vocabulary, I: Interpretation> LinkedDataSubject<I, V> for Value<T> {
	fn visit_subject<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::SubjectVisitor<I, V>,
	{
		visitor.end()
	}
}

impl<T, V: Vocabulary, I: Interpretation> LinkedDataPredicateObjects<I, V> for Value<T> {
	fn visit_objects<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::PredicateObjectsVisitor<I, V>,
	{
		visitor.end()
	}
}

impl<V: Vocabulary, I: Interpretation> LinkedDataGraph<I, V> for Value<V::Iri> {
	fn visit_graph<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::GraphVisitor<I, V>,
	{
		visitor.subject(self)?;
		visitor.end()
	}
}

impl<V: Vocabulary, I: Interpretation> LinkedData<I, V> for Value<V::Iri> {
	fn visit<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::Visitor<I, V>,
	{
		visitor.default_graph(self)?;
		visitor.end()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::rdf::XSD_UNSIGNED_INT;
	use json_syntax::NumberBuf;
	use rdf_types::vocabulary::{
		BlankIdIndex, IndexVocabulary, IriIndex, IriVocabularyMut, LiteralIndex,
	};
	use std::str::FromStr;

	struct DummyInterpretation;
	impl Interpretation for DummyInterpretation {
		type Resource = ();
	}

	#[test]
	fn test_unsigned_int_deserialization() {
		// Create a vocabulary
		let mut vocabulary = IndexVocabulary::<IriIndex, BlankIdIndex, LiteralIndex>::new();

		// Create a number value with XSD_UNSIGNED_INT type
		let unsigned_int_str = "42";
		let number = NumberBuf::from_str(unsigned_int_str).unwrap();
		let unsigned_int_type = vocabulary.insert(XSD_UNSIGNED_INT);

		// Create a Value with the number and XSD_UNSIGNED_INT type
		let value = Value::Literal(Literal::Number(number), Some(unsigned_int_type));

		// Set up interpretation
		let mut interpretation = DummyInterpretation;

		// Get the resource interpretation
		let resource_interpretation = value.interpretation(&mut vocabulary, &mut interpretation);

		// Extract the term from the resource interpretation
		match resource_interpretation {
			ResourceInterpretation::Uninterpreted(Some(term)) => {
				// Check that the term is a literal with the correct XSD type
				match term {
					CowRdfTerm::Owned(Term::Literal(RdfLiteral::Xsd(xsd_value))) => match xsd_value
					{
						xsd_types::Value::UnsignedInt(value) => {
							assert_eq!(value.to_string(), unsigned_int_str);
						}
						other => {
							panic!("Expected UnsignedInt, got {:?}", other);
						}
					},
					_ => {
						panic!("Expected RdfLiteral::Xsd, got something else");
					}
				}
			}
			_ => {
				panic!("Expected Uninterpreted with Some term");
			}
		}
	}

	#[test]
	fn test_unsigned_int_overflow_deserialization() {
		// Create a vocabulary
		let mut vocabulary = IndexVocabulary::<IriIndex, BlankIdIndex, LiteralIndex>::new();

		// Create a number that's too large for u32 but valid for u64
		let large_unsigned_str = "4294967296"; // u32::MAX + 1
		let number = NumberBuf::from_str(large_unsigned_str).unwrap();
		let unsigned_int_type = vocabulary.insert(XSD_UNSIGNED_INT);

		// Create a Value with the number and XSD_UNSIGNED_INT type
		let value = Value::Literal(Literal::Number(number), Some(unsigned_int_type));

		// Set up interpretation
		let mut interpretation = DummyInterpretation;

		// Get the resource interpretation
		let resource_interpretation = value.interpretation(&mut vocabulary, &mut interpretation);

		// Extract the term from the resource interpretation
		match resource_interpretation {
			ResourceInterpretation::Uninterpreted(Some(term)) => {
				// Check that the term is a literal with Double type (fallback)
				match term {
					CowRdfTerm::Owned(Term::Literal(RdfLiteral::Xsd(xsd_value))) => match xsd_value
					{
						xsd_types::Value::Double(_) => {
							println!("Successfully handled overflow")
						}
						other => {
							panic!("Expected Double (fallback for overflow), got {:?}", other);
						}
					},
					_ => {
						panic!("Expected RdfLiteral::Xsd, got something else");
					}
				}
			}
			_ => {
				panic!("Expected Uninterpreted with Some term");
			}
		}
	}
}
