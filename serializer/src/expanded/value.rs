use json_ld_core::{object::Literal, LangString, Value};
use linked_data::RdfLiteral;
use locspan::Meta;
use rdf_types::{literal, IriVocabularyMut, LanguageTagVocabulary};
use xsd_types::{XsdDatatype, XSD_STRING};

pub fn literal_to_value<V: IriVocabularyMut + LanguageTagVocabulary>(
    vocabulary: &mut V,
    lit: RdfLiteral<V>,
) -> Value<V::Iri> {
    match lit {
        RdfLiteral::Any(s, ty) => match ty {
            literal::Type::Any(iri) => {
                let literal_ty = if vocabulary.iri(&iri).unwrap() == XSD_STRING {
                    None
                } else {
                    Some(iri)
                };

                Value::Literal(Literal::String(s.into()), literal_ty)
            }
            literal::Type::LangString(lang) => {
                let language = vocabulary.owned_language_tag(lang).ok().unwrap();
                Value::LangString(LangString::new(s.into(), Some(language.into()), None).unwrap())
            }
        },
        RdfLiteral::Xsd(xsd) => xsd_to_value(vocabulary, xsd),
        RdfLiteral::Json(json) => Value::Json(Meta::none(json)),
    }
}

fn xsd_to_value<V: IriVocabularyMut>(vocabulary: &mut V, value: xsd_types::Value) -> Value<V::Iri> {
    let ty = value.type_();
    let number = match value {
        xsd_types::Value::Boolean(b) => return Value::Literal(Literal::Boolean(b), None),
        xsd_types::Value::String(s) => return Value::Literal(Literal::String(s.into()), None),
        xsd_types::Value::Decimal(v) => v.to_string(),
        xsd_types::Value::Integer(v) => v.to_string(),
        xsd_types::Value::NonPositiveInteger(v) => v.to_string(),
        xsd_types::Value::NegativeInteger(v) => v.to_string(),
        xsd_types::Value::Long(v) => v.to_string(),
        xsd_types::Value::Int(v) => v.to_string(),
        xsd_types::Value::Short(v) => v.to_string(),
        xsd_types::Value::Byte(v) => v.to_string(),
        xsd_types::Value::NonNegativeInteger(v) => v.to_string(),
        xsd_types::Value::UnsignedLong(v) => v.to_string(),
        xsd_types::Value::UnsignedInt(v) => v.to_string(),
        xsd_types::Value::UnsignedShort(v) => v.to_string(),
        xsd_types::Value::UnsignedByte(v) => v.to_string(),
        xsd_types::Value::PositiveInteger(v) => v.to_string(),
        other => {
            let ty = vocabulary.insert(ty.iri());
            return Value::Literal(Literal::String(other.to_string().into()), Some(ty));
        }
    };

    match json_syntax::Number::new(&number) {
        Ok(_) => {
            let n = unsafe { json_syntax::NumberBuf::new_unchecked(number.into_bytes().into()) };
            Value::Literal(Literal::Number(n), None)
        }
        Err(_) => {
            let ty = vocabulary.insert(ty.iri());
            Value::Literal(Literal::String(number.into()), Some(ty))
        }
    }
}
