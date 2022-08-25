use crate::{vocab, BlankIdIndex, Error, IriIndex, TestSpec, Vocab};
use grdf::Dataset;
use json_ld::{BlankIdNamespace, IndexNamespace, IriNamespace, ValidReference};
use proc_macro2::TokenStream;
use quote::quote;
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
	Ref(syn::Ident),
}

impl Type {
	fn is_reference(&self) -> bool {
		matches!(self, Self::Ref(_))
	}

	fn as_reference(&self) -> Option<&syn::Ident> {
		match self {
			Self::Ref(r) => Some(r),
			_ => None,
		}
	}

	pub(crate) fn generate(
		&self,
		namespace: &IndexNamespace,
		spec: &TestSpec,
		dataset: &OwnedDataset,
		value: &json_ld::rdf::Value<IriIndex, BlankIdIndex>,
	) -> Result<TokenStream, Error> {
		match self {
			Self::Bool => {
				let b = match value {
					json_ld::rdf::Value::Literal(rdf_types::Literal::TypedString(
						s,
						IriIndex::Iri(Vocab::Xsd(vocab::Xsd::Boolean)),
					)) => match s.as_str() {
						"true" => true,
						"false" => false,
						_ => return Err(Error::InvalidValue(self.clone(), value.clone())),
					},
					_ => return Err(Error::InvalidValue(self.clone(), value.clone())),
				};

				Ok(quote! { #b })
			}
			Self::String => {
				let s = match value {
					json_ld::rdf::Value::Literal(lit) => lit.string_literal().as_str(),
					json_ld::rdf::Value::Reference(ValidReference::Id(i)) => {
						namespace.iri(i).unwrap().into_str()
					}
					json_ld::rdf::Value::Reference(ValidReference::Blank(i)) => {
						namespace.blank_id(i).unwrap().as_str()
					}
				};

				Ok(quote! { #s })
			}
			Self::Iri => match value {
				json_ld::rdf::Value::Reference(ValidReference::Id(i)) => {
					let s = namespace.iri(i).unwrap().into_str();
					Ok(quote! { ::static_iref::iri!(#s) })
				}
				_ => Err(Error::InvalidValue(self.clone(), value.clone())),
			},
			Self::ProcessingMode => {
				let s = match value {
					json_ld::rdf::Value::Literal(rdf_types::Literal::String(s)) => s.as_str(),
					json_ld::rdf::Value::Literal(rdf_types::Literal::TypedString(
						s,
						IriIndex::Iri(Vocab::Xsd(vocab::Xsd::String)),
					)) => s.as_str(),
					_ => return Err(Error::InvalidValue(self.clone(), value.clone())),
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
					Err(_) => Err(Error::InvalidValue(self.clone(), value.clone())),
				}
			}
			Self::Ref(r) => match value {
				json_ld::rdf::Value::Reference(id) => {
					let d = spec.types.get(r).unwrap();
					let mod_id = &spec.id;
					d.generate(namespace, spec, dataset, *id, quote! { #mod_id :: #r })
				}
				_ => Err(Error::InvalidValue(self.clone(), value.clone())),
			},
		}
	}
}

impl Struct {
	pub(crate) fn generate(
		&self,
		namespace: &json_ld::IndexNamespace,
		spec: &TestSpec,
		dataset: &OwnedDataset,
		id: ValidReference<IriIndex, BlankIdIndex>,
		path: TokenStream,
	) -> Result<TokenStream, Error> {
		let mut fields = Vec::new();

		for (field_iri, field) in &self.fields {
			let ident = &field.id;
			let value = if *field_iri == IriIndex::Iri(Vocab::Rdf(vocab::Rdf::Type))
				&& field.ty.is_reference()
			{
				if field.multiple || !field.required {
					return Err(Error::InvalidTypeField);
				}

				let ty_id = field.ty.as_reference().expect("not a reference");
				let ty = spec.types.get(ty_id).expect("undefined type");
				let mod_id = &spec.id;
				ty.generate(namespace, spec, dataset, id, quote! { #mod_id :: #ty_id })?
			} else {
				let mut objects = dataset
					.default_graph()
					.objects(&id, &ValidReference::Id(*field_iri));

				if field.multiple {
					let mut items = Vec::new();

					for object in objects {
						items.push(field.ty.generate(namespace, spec, dataset, object)?)
					}

					quote! {
						&[ #(#items),* ]
					}
				} else if field.required {
					match objects.next() {
						Some(object) => field.ty.generate(namespace, spec, dataset, object)?,
						// None => return Err(Error::MissingRequiredValue(id, *field_iri))
						None => {
							quote! { ::core::default::Default::default() }
						}
					}
				} else {
					match objects.next() {
						Some(object) => {
							let value = field.ty.generate(namespace, spec, dataset, object)?;
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

type OwnedDataset<'a> = grdf::HashDataset<
	ValidReference<IriIndex, BlankIdIndex>,
	ValidReference<IriIndex, BlankIdIndex>,
	json_ld::rdf::Value<IriIndex, BlankIdIndex>,
	&'a ValidReference<IriIndex, BlankIdIndex>,
>;

impl Definition {
	pub(crate) fn generate(
		&self,
		namespace: &json_ld::IndexNamespace,
		spec: &TestSpec,
		dataset: &OwnedDataset,
		id: ValidReference<IriIndex, BlankIdIndex>,
		path: TokenStream,
	) -> Result<TokenStream, Error> {
		match self {
			Self::Struct(s) => s.generate(namespace, spec, dataset, id, path),
			Self::Enum(e) => {
				let mut variant = None;
				let node_types = dataset.default_graph().objects(
					&id,
					&ValidReference::Id(IriIndex::Iri(Vocab::Rdf(vocab::Rdf::Type))),
				);

				for ty_iri in node_types {
					match ty_iri {
						json_ld::rdf::Value::Reference(ValidReference::Id(ty_iri)) => {
							if let Some(v) = e.variants.get(ty_iri) {
								if variant.replace(v).is_some() {
									return Err(Error::MultipleTypeVariants(id));
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
							namespace,
							spec,
							dataset,
							id,
							quote! { #path :: #variant_id },
						)
					}
					None => Err(Error::NoTypeVariants(id)),
				}
			}
		}
	}
}
