use json_syntax::Print;
use linked_data::{LinkedDataSerializer, SerializeLinkedData};
use rdf_types::{Literal, Term, RDF_JSON};
use xsd_types::{XSD_BOOLEAN, XSD_DOUBLE, XSD_INTEGER, XSD_STRING};

use crate::{object::LiteralValue, LenientLangTag, ValueObject};

impl SerializeLinkedData for ValueObject {
	fn serialize_rdf<S>(&self, mut serializer: S, _: Option<&Term>) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		let literal = match self {
			Self::Literal(l, ty) => {
				let ty = ty.as_deref().unwrap_or_else(|| match l {
					LiteralValue::Null => XSD_STRING,
					LiteralValue::Boolean(_) => XSD_BOOLEAN,
					LiteralValue::Number(n) => {
						if n.as_i64().is_some() {
							XSD_INTEGER
						} else {
							XSD_DOUBLE
						}
					}
					LiteralValue::String(_) => XSD_STRING,
				});

				match l {
					LiteralValue::Null => Literal::new("null", ty),
					LiteralValue::Boolean(b) => {
						Literal::new(xsd_types::Boolean(*b).to_string(), ty)
					}
					LiteralValue::Number(n) => Literal::new(n.to_string(), ty),
					LiteralValue::String(s) => Literal::new(s.as_str(), ty),
				}
			}
			Self::LangString(s) => match s.language().and_then(LenientLangTag::as_well_formed) {
				Some(tag) => Literal::new(s.as_str(), tag),
				None => Literal::new(s.as_str(), XSD_STRING),
			},
			Self::Json(json) => Literal::new(json.compact_print().to_string(), RDF_JSON),
		};

		serializer.serialize_resource(Term::literal(literal))?;
		serializer.end()
	}
}
