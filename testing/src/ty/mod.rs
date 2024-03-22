use crate::{
	vocab, vocab::IndexTerm, BlankIdIndex, Error, IndexVocabulary, IriIndex, TestSpec, Vocab,
};
use contextual::AsRefWithContext;
use json_ld::ValidId;
use proc_macro2::TokenStream;
use quote::quote;
use rdf_types::{
	dataset::{IndexedBTreeDataset, PatternMatchingDataset},
	vocabulary::{IriVocabulary, LiteralIndex, LiteralVocabulary},
};
use std::collections::HashMap;

mod parse;

pub use parse::{parse, Parsed, UnknownType};

pub enum Definition {
	Struct(Struct),
	Enum(Enum),
}

pub struct Struct {
	pub fields: HashMap<IriIndex, Field>,
}

pub struct Field {
	pub id: syn::Ident,
	pub ty: Type,
	pub required: bool,
	pub multiple: bool,
}

pub struct Enum {
	pub variants: HashMap<IriIndex, Variant>,
}

pub struct Variant {
	pub id: syn::Ident,
	pub data: Struct,
}

#[derive(Clone)]
pub enum Type {
	Bool,
	String,
	Iri,
	ProcessingMode,
	RdfDirection,
	Ref(syn::Ident),
}

impl Type {
	fn is_id(&self) -> bool {
		matches!(self, Self::Ref(_))
	}

	fn as_id(&self) -> Option<&syn::Ident> {
		match self {
			Self::Ref(r) => Some(r),
			_ => None,
		}
	}

	pub(crate) fn generate(
		&self,
		vocabulary: &IndexVocabulary,
		spec: &TestSpec,
		dataset: &IndexedBTreeDataset<IndexTerm>,
		value: &json_ld::rdf::Value<IriIndex, BlankIdIndex, LiteralIndex>,
	) -> Result<TokenStream, Box<Error>> {
		match self {
			Self::Bool => {
				let b = match value {
					json_ld::rdf::Value::Literal(l) => {
						let literal = vocabulary.literal(l).unwrap();
						if literal.type_
							== rdf_types::LiteralType::Any(IriIndex::Iri(Vocab::Xsd(
								vocab::Xsd::Boolean,
							))) {
							match literal.value {
								"true" => true,
								"false" => false,
								_ => {
									return Err(Box::new(Error::InvalidValue(self.clone(), *value)))
								}
							}
						} else {
							return Err(Box::new(Error::InvalidValue(self.clone(), *value)));
						}
					}
					_ => return Err(Box::new(Error::InvalidValue(self.clone(), *value))),
				};

				Ok(quote! { #b })
			}
			Self::String => {
				let s = match value {
					json_ld::rdf::Value::Literal(lit) => vocabulary.literal(lit).unwrap().value,
					json_ld::rdf::Value::Id(id) => id.as_ref_with(vocabulary),
				};

				Ok(quote! { #s })
			}
			Self::Iri => match value {
				json_ld::rdf::Value::Id(ValidId::Iri(i)) => {
					let s = vocabulary.iri(i).unwrap().as_str();
					Ok(quote! { ::static_iref::iri!(#s) })
				}
				_ => Err(Box::new(Error::InvalidValue(self.clone(), *value))),
			},
			Self::ProcessingMode => {
				let s = match value {
					json_ld::rdf::Value::Literal(l) => {
						let literal = vocabulary.literal(l).unwrap();
						if literal.type_
							== rdf_types::LiteralType::Any(IriIndex::Iri(Vocab::Xsd(
								vocab::Xsd::String,
							))) {
							literal.value
						} else {
							return Err(Box::new(Error::InvalidValue(self.clone(), *value)));
						}
					}
					_ => return Err(Box::new(Error::InvalidValue(self.clone(), *value))),
				};

				match json_ld::ProcessingMode::try_from(s) {
					Ok(p) => match p {
						json_ld::ProcessingMode::JsonLd1_0 => {
							Ok(quote! { ::json_ld::ProcessingMode::JsonLd1_0 })
						}
						json_ld::ProcessingMode::JsonLd1_1 => {
							Ok(quote! { ::json_ld::ProcessingMode::JsonLd1_1 })
						}
					},
					Err(_) => Err(Box::new(Error::InvalidValue(self.clone(), *value))),
				}
			}
			Self::RdfDirection => {
				let s = match value {
					json_ld::rdf::Value::Literal(l) => {
						let literal = vocabulary.literal(l).unwrap();
						if literal.type_
							== rdf_types::LiteralType::Any(IriIndex::Iri(Vocab::Xsd(
								vocab::Xsd::String,
							))) {
							literal.value
						} else {
							return Err(Box::new(Error::InvalidValue(self.clone(), *value)));
						}
					}
					_ => return Err(Box::new(Error::InvalidValue(self.clone(), *value))),
				};

				match json_ld::rdf::RdfDirection::try_from(s) {
					Ok(p) => match p {
						json_ld::rdf::RdfDirection::CompoundLiteral => {
							Ok(quote! { ::json_ld::rdf::RdfDirection::CompoundLiteral })
						}
						json_ld::rdf::RdfDirection::I18nDatatype => {
							Ok(quote! { ::json_ld::rdf::RdfDirection::I18nDatatype })
						}
					},
					Err(_) => Err(Box::new(Error::InvalidValue(self.clone(), *value))),
				}
			}
			Self::Ref(r) => match value {
				json_ld::rdf::Value::Id(id) => {
					let d = spec.types.get(r).unwrap();
					let mod_id = &spec.id;
					d.generate(
						vocabulary,
						spec,
						dataset,
						IndexTerm::Id(*id),
						quote! { #mod_id :: #r },
					)
				}
				_ => Err(Box::new(Error::InvalidValue(self.clone(), *value))),
			},
		}
	}
}

impl Struct {
	pub(crate) fn generate(
		&self,
		vocabulary: &IndexVocabulary,
		spec: &TestSpec,
		dataset: &IndexedBTreeDataset<IndexTerm>,
		id: IndexTerm,
		path: TokenStream,
	) -> Result<TokenStream, Box<Error>> {
		let mut fields = Vec::new();

		for (field_iri, field) in &self.fields {
			let ident = &field.id;
			let value =
				if *field_iri == IriIndex::Iri(Vocab::Rdf(vocab::Rdf::Type)) && field.ty.is_id() {
					if field.multiple || !field.required {
						return Err(Box::new(Error::InvalidTypeField));
					}

					let ty_id = field.ty.as_id().expect("not a reference");
					let ty = spec.types.get(ty_id).expect("undefined type");
					let mod_id = &spec.id;
					ty.generate(vocabulary, spec, dataset, id, quote! { #mod_id :: #ty_id })?
				} else {
					let field_predicate = IndexTerm::iri(*field_iri);
					let mut objects = dataset.quad_objects(None, &id, &field_predicate);

					if field.multiple {
						let mut items = Vec::new();

						for object in objects {
							items.push(field.ty.generate(vocabulary, spec, dataset, object)?)
						}

						quote! {
							&[ #(#items),* ]
						}
					} else if field.required {
						match objects.next() {
							Some(object) => field.ty.generate(vocabulary, spec, dataset, object)?,
							// None => return Err(Error::MissingRequiredValue(id, *field_iri))
							None => {
								quote! { ::core::default::Default::default() }
							}
						}
					} else {
						match objects.next() {
							Some(object) => {
								let value = field.ty.generate(vocabulary, spec, dataset, object)?;
								quote! { Some(#value) }
							}
							None => quote! { None },
						}
					}
				};

			fields.push(quote! { #ident: #value })
		}

		Ok(quote! { #path { #(#fields),* } })
	}
}

impl Definition {
	pub(crate) fn generate(
		&self,
		vocabulary: &IndexVocabulary,
		spec: &TestSpec,
		dataset: &IndexedBTreeDataset<IndexTerm>,
		id: IndexTerm,
		path: TokenStream,
	) -> Result<TokenStream, Box<Error>> {
		match self {
			Self::Struct(s) => s.generate(vocabulary, spec, dataset, id, path),
			Self::Enum(e) => {
				let mut variant = None;
				let node_types = dataset.quad_objects(
					None,
					&id,
					&IndexTerm::Id(ValidId::Iri(IriIndex::Iri(Vocab::Rdf(vocab::Rdf::Type)))),
				);

				for ty_iri in node_types {
					match ty_iri {
						json_ld::rdf::Value::Id(ValidId::Iri(ty_iri)) => {
							if let Some(v) = e.variants.get(ty_iri) {
								if variant.replace(v).is_some() {
									return Err(Box::new(Error::MultipleTypeVariants(id)));
								}
							}
						}
						_ => panic!("invalid type"),
					}
				}

				match variant {
					Some(variant) => {
						let variant_id = &variant.id;
						variant.data.generate(
							vocabulary,
							spec,
							dataset,
							id,
							quote! { #path :: #variant_id },
						)
					}
					None => Err(Box::new(Error::NoTypeVariants(id))),
				}
			}
		}
	}
}
