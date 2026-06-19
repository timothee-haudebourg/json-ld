use iref::Iri;
use json_syntax::{JsonNumberBuf, JsonValue, PrintJson};
use linked_data::{LinkedDataSerializer, SerializeLinkedData};
use rdf_types::{Literal, Term, RDF_JSON};
use xsd_types::{Double, ParseXsd, XSD_BOOLEAN, XSD_DOUBLE, XSD_INTEGER, XSD_STRING};

use crate::{object::value::LiteralType, LenientLangTag, ValueObject};

impl SerializeLinkedData for ValueObject {
	fn serialize_rdf<S>(&self, mut serializer: S, _: Option<&Term>) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		let literal = match self {
			Self::Literal(lit) => {
				if lit.is_json() {
					Literal::new(lit.value.compact_print().to_string(), RDF_JSON)
				} else {
					let ty = match &lit.type_ {
						Some(LiteralType::Iri(iri)) => iri.as_iri(),
						_ => match &lit.value {
							JsonValue::Null => XSD_STRING,
							JsonValue::Boolean(_) => XSD_BOOLEAN,
							JsonValue::Number(n) => {
								if n.as_i64().is_some() {
									XSD_INTEGER
								} else {
									XSD_DOUBLE
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
						other => Literal::new(other.compact_print().to_string(), ty),
					}
				}
			}
			Self::LangString(s) => match s.language().and_then(LenientLangTag::as_well_formed) {
				Some(tag) => Literal::new(s.as_str(), tag),
				None => Literal::new(s.as_str(), XSD_STRING),
			},
		};

		serializer.serialize_resource(Some(Term::literal(literal)))?;
		serializer.end()
	}
}

fn canonical_number(n: &JsonNumberBuf, ty: &Iri) -> String {
	if ty == XSD_DOUBLE || n.has_decimal_point() {
		if let Ok(d) = Double::parse_xsd(n) {
			return d.to_string();
		}
	};

	n.to_string()
}
