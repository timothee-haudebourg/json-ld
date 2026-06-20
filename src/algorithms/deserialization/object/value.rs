use json_syntax::{JsonNumberBuf, JsonValue, PrintJson};
use linked_data::{ser::SerializeLinkedDataWith, LinkedDataSerializer, SerializeLinkedData};
use rdf_syntax::Iri;
use rdf_syntax::{Literal, Term, RDF_JSON};
use xsd_types::{Double, ParseXsd, XSD_BOOLEAN, XSD_DOUBLE, XSD_INTEGER, XSD_STRING};

use crate::Lenient;
use crate::{object::value::LiteralType, ValueObject};

use super::super::RdfSerializationOptions;

impl SerializeLinkedDataWith<RdfSerializationOptions> for ValueObject {
	fn serialize_rdf_with<S>(
		&self,
		_opts: RdfSerializationOptions,
		mut serializer: S,
		_: Option<&Term>,
	) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		let literal = match self {
			Self::Literal(lit) => {
				if lit.is_json() {
					Literal::new(
						lit.value.canonicalized().compact_print().to_string(),
						RDF_JSON,
					)
				} else {
					let ty = match &lit.type_ {
						Some(LiteralType::Iri(iri)) => iri.as_iri(),
						_ => match &lit.value {
							JsonValue::Null => XSD_STRING,
							JsonValue::Boolean(_) => XSD_BOOLEAN,
							JsonValue::Number(n) => {
								let n = n.trimmed();
								if n.has_decimal_point() || n.has_exponent() {
									XSD_DOUBLE
								} else {
									XSD_INTEGER
								}
							}
							JsonValue::String(_) => XSD_STRING,
							_ => XSD_STRING,
						},
					};

					match &lit.value {
						JsonValue::Null => Literal::new("null", ty),
						JsonValue::Boolean(b) => {
							Literal::new(xsd_types::Boolean(*b).to_string(), ty)
						}
						JsonValue::Number(n) => Literal::new(canonical_number(n, ty), ty),
						JsonValue::String(s) => Literal::new(s.as_str(), ty),
						other => {
							Literal::new(other.canonicalized().compact_print().to_string(), ty)
						}
					}
				}
			}
			Self::LangString(s) => match s.language() {
				Some(lang) => match lang {
					Lenient::Valid(tag) => Literal::new(s.as_str(), tag),
					Lenient::Invalid(_) => return serializer.end(),
				},
				None => Literal::new(s.as_str(), XSD_STRING),
			},
		};

		serializer.serialize_resource(Some(Term::literal(literal)))?;
		serializer.end()
	}
}

impl SerializeLinkedData for ValueObject {
	fn serialize_rdf<S>(&self, serializer: S, graph: Option<&Term>) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		self.serialize_rdf_with(RdfSerializationOptions::default(), serializer, graph)
	}
}

fn canonical_number(n: &JsonNumberBuf, ty: &Iri) -> String {
	let n = n.trimmed();

	if ty == XSD_DOUBLE || n.has_decimal_point() || n.has_exponent() {
		if let Ok(d) = Double::parse_xsd(n) {
			return d.to_string();
		}
	}

	n.to_string()
}
