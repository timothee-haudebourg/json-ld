use linked_data::{
	xsd_types, CowRdfTerm, LinkedData, LinkedDataGraph, LinkedDataPredicateObjects,
	LinkedDataResource, LinkedDataSubject, RdfLiteral, RdfLiteralRef, ResourceInterpretation,
};
use rdf_types::{Interpretation, Term, Vocabulary};

use crate::{
	object::Literal,
	rdf::{XSD_DOUBLE, XSD_INTEGER},
	Value,
};

impl<V: Vocabulary, I: Interpretation> LinkedDataResource<I, V> for Value<V::Iri> {
	fn interpretation(
		&self,
		vocabulary: &mut V,
		_interpretation: &mut I,
	) -> ResourceInterpretation<I, V> {
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
							if iri == XSD_INTEGER {
								NumericType::Integer
							} else if iri == XSD_DOUBLE {
								NumericType::Double
							} else {
								NumericType::Unknown
							}
						})
						.unwrap_or_default();

					let value = match n.as_i64() {
						Some(i) if ty.matches(NumericType::Integer) => {
							xsd_types::Value::Integer(i.into())
						}
						_ => xsd_types::Value::Double(n.as_f64_lossy().into()),
					};

					CowRdfTerm::Owned(Term::Literal(RdfLiteral::Xsd(value)))
				}
				Literal::String(s) => match ty {
					Some(ty) => CowRdfTerm::from_str(vocabulary, s.as_str(), ty),
					None => CowRdfTerm::Borrowed(Term::Literal(RdfLiteralRef::Xsd(
						xsd_types::ValueRef::String(s),
					))),
				},
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
