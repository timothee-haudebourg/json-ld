use linked_data::{
	xsd_types, CowRdfTerm, LinkedData, LinkedDataGraph, LinkedDataPredicateObjects,
	LinkedDataResource, LinkedDataSubject, RdfLiteral, RdfLiteralRef, ResourceInterpretation,
};
use rdf_types::{Interpretation, LiteralTypeRef, Term, Vocabulary};

use crate::{object::Literal, Value};

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
				Literal::Number(n) => match ty {
					Some(ty) => match typed_number_interpretation(vocabulary, ty, n) {
						Some(value) => CowRdfTerm::Owned(Term::Literal(RdfLiteral::Xsd(value))),
						None => CowRdfTerm::Borrowed(Term::Literal(RdfLiteralRef::Any(
							n.as_str(),
							LiteralTypeRef::Any(ty),
						))),
					},
					None => {
						let value = match n.as_i64() {
							Some(i) => xsd_types::Value::Integer(i.into()),
							None => xsd_types::Value::Double(n.as_f64_lossy().into()),
						};

						CowRdfTerm::Owned(Term::Literal(RdfLiteral::Xsd(value)))
					}
				},
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

fn typed_number_interpretation<V: Vocabulary>(
	vocabulary: &V,
	ty: &V::Iri,
	n: &json_syntax::Number,
) -> Option<xsd_types::Value> {
	let iri = vocabulary.iri(ty)?;
	let xsd_ty = xsd_types::Datatype::from_iri(iri)?;
	xsd_ty.parse(n).ok()
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
