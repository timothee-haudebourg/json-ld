//! This library provides the `test_suite` derive macro
//! that can generate Rust test suites from a JSON-LD document.
use async_std::task;
use contextual::{DisplayWithContext, WithContext};
use iref::{IriBuf, IriRefBuf};
use json_ld::{Expand, FsLoader, ValidId};
use proc_macro2::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use rdf_types::{
	dataset::IndexedBTreeDataset,
	vocabulary::{IriVocabulary, IriVocabularyMut, LiteralIndex},
	Quad,
};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use syn::parse::ParseStream;
use syn::spanned::Spanned;

mod vocab;
use vocab::{BlankIdIndex, IndexQuad, IndexTerm, IriIndex, Vocab};
mod ty;
use ty::{Type, UnknownType};

type IndexVocabulary = rdf_types::vocabulary::IndexVocabulary<IriIndex, BlankIdIndex>;

struct MountAttribute {
	_paren: syn::token::Paren,
	prefix: IriBuf,
	_comma: syn::token::Comma,
	target: PathBuf,
}

impl syn::parse::Parse for MountAttribute {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let content;
		let _paren = syn::parenthesized!(content in input);

		let prefix: syn::LitStr = content.parse()?;
		let prefix = IriBuf::new(prefix.value())
			.map_err(|e| content.error(format!("invalid IRI `{}`", e.0)))?;

		let _comma = content.parse()?;

		let target: syn::LitStr = content.parse()?;

		Ok(Self {
			_paren,
			prefix,
			_comma,
			target: target.value().into(),
		})
	}
}

struct IriAttribute {
	_paren: syn::token::Paren,
	iri: IriBuf,
}

impl syn::parse::Parse for IriAttribute {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let content;
		let _paren = syn::parenthesized!(content in input);

		let iri: syn::LitStr = content.parse()?;
		let iri = IriBuf::new(iri.value())
			.map_err(|e| content.error(format!("invalid IRI `{}`", e.0)))?;

		Ok(Self { _paren, iri })
	}
}

struct IriArg {
	iri: IriBuf,
}

impl syn::parse::Parse for IriArg {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let iri: syn::LitStr = input.parse()?;
		let iri =
			IriBuf::new(iri.value()).map_err(|e| input.error(format!("invalid IRI `{}`", e.0)))?;

		Ok(Self { iri })
	}
}

struct PrefixBinding {
	_paren: syn::token::Paren,
	prefix: String,
	_eq: syn::token::Eq,
	iri: IriBuf,
}

impl syn::parse::Parse for PrefixBinding {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let content;
		let _paren = syn::parenthesized!(content in input);

		let prefix: syn::LitStr = content.parse()?;

		let _eq = content.parse()?;

		let iri: syn::LitStr = content.parse()?;
		let iri = IriBuf::new(iri.value())
			.map_err(|e| content.error(format!("invalid IRI `{}`", e.0)))?;

		Ok(Self {
			_paren,
			prefix: prefix.value(),
			_eq,
			iri,
		})
	}
}

struct IgnoreAttribute {
	_paren: syn::token::Paren,
	iri_ref: IriRefBuf,
	_comma: syn::token::Comma,
	_see: syn::Ident,
	_eq: syn::token::Eq,
	link: String,
}

impl syn::parse::Parse for IgnoreAttribute {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let content;
		let _paren = syn::parenthesized!(content in input);

		let iri_ref: syn::LitStr = content.parse()?;
		let iri_ref = IriRefBuf::new(iri_ref.value())
			.map_err(|e| content.error(format!("invalid IRI reference `{}`", e.0)))?;

		let _comma = content.parse()?;

		let _see = content.parse()?;

		let _eq = content.parse()?;

		let link: syn::LitStr = content.parse()?;
		let link = link.value();

		Ok(Self {
			_paren,
			iri_ref,
			_comma,
			_see,
			_eq,
			link,
		})
	}
}

struct TestSpec {
	id: syn::Ident,
	prefix: String,
	suite: IriIndex,
	types: HashMap<syn::Ident, ty::Definition>,
	type_map: HashMap<IriIndex, syn::Ident>,
	ignore: HashMap<IriIndex, String>,
}

struct InvalidIri(String);

fn expand_iri(
	vocabulary: &mut IndexVocabulary,
	bindings: &mut HashMap<String, IriIndex>,
	iri: IriBuf,
) -> Result<IriIndex, InvalidIri> {
	match iri.as_str().split_once(':') {
		Some((prefix, suffix)) => match bindings.get(prefix) {
			Some(prefix) => {
				let mut result = vocabulary.iri(prefix).unwrap().to_string();
				result.push_str(suffix);

				match iref::Iri::new(&result) {
					Ok(iri) => Ok(vocabulary.insert(iri)),
					Err(_) => Err(InvalidIri(iri.to_string())),
				}
			}
			None => Ok(vocabulary.insert(iri.as_iri())),
		},
		None => Ok(vocabulary.insert(iri.as_iri())),
	}
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn test_suite(
	args: proc_macro::TokenStream,
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	let mut input = syn::parse_macro_input!(input as syn::ItemMod);
	let mut vocabulary = IndexVocabulary::new();

	match task::block_on(derive_test_suite(&mut vocabulary, &mut input, args)) {
		Ok(tokens) => quote! { #input #tokens }.into(),
		Err(e) => {
			proc_macro_error::abort_call_site!(
				"test suite generation failed: {}",
				(*e).with(&vocabulary)
			)
		}
	}
}

async fn derive_test_suite(
	vocabulary: &mut IndexVocabulary,
	input: &mut syn::ItemMod,
	args: proc_macro::TokenStream,
) -> Result<TokenStream, Box<Error>> {
	let mut loader = FsLoader::default();
	let spec = parse_input(vocabulary, &mut loader, input, args)?;
	generate_test_suite(vocabulary, loader, spec).await
}

fn parse_input(
	vocabulary: &mut IndexVocabulary,
	loader: &mut FsLoader,
	input: &mut syn::ItemMod,
	args: proc_macro::TokenStream,
) -> Result<TestSpec, Box<Error>> {
	let suite: IriArg = syn::parse(args).map_err(|e| Box::new(e.into()))?;
	let base = suite.iri;
	let suite = vocabulary.insert(base.as_iri());

	let mut bindings: HashMap<String, IriIndex> = HashMap::new();
	let mut ignore: HashMap<IriIndex, String> = HashMap::new();

	let attrs = std::mem::take(&mut input.attrs);
	for attr in attrs {
		if attr.path.is_ident("mount") {
			let mount: MountAttribute = syn::parse2(attr.tokens).map_err(|e| Box::new(e.into()))?;
			loader.mount(mount.prefix.as_iri().to_owned(), mount.target)
		} else if attr.path.is_ident("iri_prefix") {
			let attr: PrefixBinding = syn::parse2(attr.tokens).map_err(|e| Box::new(e.into()))?;
			bindings.insert(attr.prefix, vocabulary.insert(attr.iri.as_iri()));
		} else if attr.path.is_ident("ignore_test") {
			let attr: IgnoreAttribute = syn::parse2(attr.tokens).map_err(|e| Box::new(e.into()))?;
			let resolved = attr.iri_ref.resolved(base.as_iri());
			ignore.insert(vocabulary.insert(resolved.as_iri()), attr.link);
		} else {
			input.attrs.push(attr)
		}
	}

	let mut type_map = HashMap::new();
	let mut types = HashMap::new();
	if let Some((_, items)) = input.content.as_mut() {
		for item in items {
			match item {
				syn::Item::Struct(s) => {
					types.insert(
						s.ident.clone(),
						ty::Definition::Struct(parse_struct_type(
							vocabulary,
							&mut bindings,
							&mut type_map,
							s,
						)?),
					);
				}
				syn::Item::Enum(e) => {
					types.insert(
						e.ident.clone(),
						ty::Definition::Enum(parse_enum_type(
							vocabulary,
							&mut bindings,
							&mut type_map,
							e,
						)?),
					);
				}
				_ => (),
			}
		}
	}

	let prefix = test_prefix(&input.ident.to_string());
	Ok(TestSpec {
		id: input.ident.clone(),
		prefix,
		suite,
		types,
		type_map,
		ignore,
	})
}

fn parse_struct_type(
	vocabulary: &mut IndexVocabulary,
	bindings: &mut HashMap<String, IriIndex>,
	type_map: &mut HashMap<IriIndex, syn::Ident>,
	s: &mut syn::ItemStruct,
) -> Result<ty::Struct, Box<Error>> {
	let mut fields = HashMap::new();

	let attrs = std::mem::take(&mut s.attrs);
	for attr in attrs {
		if attr.path.is_ident("iri") {
			let attr: IriAttribute = syn::parse2(attr.tokens).map_err(|e| Box::new(e.into()))?;
			let iri = expand_iri(vocabulary, bindings, attr.iri).map_err(|e| Box::new(e.into()))?;
			type_map.insert(iri, s.ident.clone());
		} else {
			s.attrs.push(attr)
		}
	}

	for field in &mut s.fields {
		let span = field.span();

		let id = match field.ident.clone() {
			Some(id) => id,
			None => {
				proc_macro_error::abort!(span, "only named fields are supported")
			}
		};

		let mut iri: Option<IriIndex> = None;
		let attrs = std::mem::take(&mut field.attrs);
		for attr in attrs {
			if attr.path.is_ident("iri") {
				let attr: IriAttribute =
					syn::parse2(attr.tokens).map_err(|e| Box::new(e.into()))?;
				iri = Some(
					expand_iri(vocabulary, bindings, attr.iri).map_err(|e| Box::new(e.into()))?,
				)
			} else {
				field.attrs.push(attr)
			}
		}

		match iri {
			Some(iri) => {
				let ty_span = field.ty.span();
				match ty::parse(field.ty.clone()) {
					Ok(ty::Parsed {
						ty,
						required,
						multiple,
					}) => {
						fields.insert(
							iri,
							ty::Field {
								id,
								ty,
								required,
								multiple,
							},
						);
					}
					Err(UnknownType) => {
						proc_macro_error::abort!(ty_span, "unknown type")
					}
				}
			}
			None => {
				proc_macro_error::abort!(span, "no IRI specified for field")
			}
		}
	}

	Ok(ty::Struct { fields })
}

fn parse_enum_type(
	vocabulary: &mut IndexVocabulary,
	bindings: &mut HashMap<String, IriIndex>,
	type_map: &mut HashMap<IriIndex, syn::Ident>,
	e: &mut syn::ItemEnum,
) -> Result<ty::Enum, Box<Error>> {
	let mut variants = HashMap::new();

	let attrs = std::mem::take(&mut e.attrs);
	for attr in attrs {
		if attr.path.is_ident("iri") {
			let attr: IriAttribute = syn::parse2(attr.tokens).map_err(|e| Box::new(e.into()))?;
			let iri = expand_iri(vocabulary, bindings, attr.iri).map_err(|e| Box::new(e.into()))?;
			type_map.insert(iri, e.ident.clone());
		} else {
			e.attrs.push(attr)
		}
	}

	for variant in &mut e.variants {
		let span = variant.span();
		let mut iri: Option<IriIndex> = None;
		let attrs = std::mem::take(&mut variant.attrs);
		for attr in attrs {
			if attr.path.is_ident("iri") {
				let attr: IriAttribute =
					syn::parse2(attr.tokens).map_err(|e| Box::new(e.into()))?;
				iri = Some(
					expand_iri(vocabulary, bindings, attr.iri).map_err(|e| Box::new(e.into()))?,
				)
			} else {
				variant.attrs.push(attr)
			}
		}

		match iri {
			Some(iri) => {
				let mut fields = HashMap::new();

				for field in &mut variant.fields {
					let field_span = field.span();
					let id = match field.ident.clone() {
						Some(id) => id,
						None => {
							proc_macro_error::abort!(field_span, "only named fields are supported")
						}
					};

					let mut field_iri: Option<IriIndex> = None;
					let attrs = std::mem::take(&mut field.attrs);
					for attr in attrs {
						if attr.path.is_ident("iri") {
							let attr: IriAttribute =
								syn::parse2(attr.tokens).map_err(|e| Box::new(e.into()))?;
							field_iri = Some(
								expand_iri(vocabulary, bindings, attr.iri)
									.map_err(|e| Box::new(e.into()))?,
							)
						} else {
							field.attrs.push(attr)
						}
					}

					let field_iri = match field_iri {
						Some(iri) => iri,
						None => {
							proc_macro_error::abort!(field_span, "no IRI specified for field")
						}
					};

					let ty_span = field.ty.span();
					match ty::parse(field.ty.clone()) {
						Ok(ty::Parsed {
							ty,
							required,
							multiple,
						}) => {
							fields.insert(
								field_iri,
								ty::Field {
									id,
									ty,
									required,
									multiple,
								},
							);
						}
						Err(UnknownType) => {
							proc_macro_error::abort!(ty_span, "unknown type")
						}
					}
				}

				variants.insert(
					iri,
					ty::Variant {
						id: variant.ident.clone(),
						data: ty::Struct { fields },
					},
				);
			}
			None => {
				proc_macro_error::abort!(span, "no IRI specified for variant")
			}
		}
	}

	Ok(ty::Enum { variants })
}

enum Error {
	Parse(syn::Error),
	Load(json_ld::loader::fs::Error),
	Expand(json_ld::expansion::Error),
	InvalidIri(String),
	InvalidValue(
		Type,
		json_ld::rdf::Value<IriIndex, BlankIdIndex, LiteralIndex>,
	),
	InvalidTypeField,
	NoTypeVariants(IndexTerm),
	MultipleTypeVariants(IndexTerm),
}

impl From<syn::Error> for Error {
	fn from(e: syn::Error) -> Self {
		Self::Parse(e)
	}
}

impl From<InvalidIri> for Error {
	fn from(InvalidIri(s): InvalidIri) -> Self {
		Self::InvalidIri(s)
	}
}

impl DisplayWithContext<IndexVocabulary> for Error {
	fn fmt_with(&self, vocabulary: &IndexVocabulary, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use fmt::Display;
		match self {
			Self::Parse(e) => e.fmt(f),
			Self::Load(e) => e.fmt(f),
			Self::Expand(e) => e.fmt(f),
			Self::InvalidIri(i) => write!(f, "invalid IRI `{i}`"),
			Self::InvalidValue(ty, value) => {
				write!(f, "invalid value {} for type {ty}", value.with(vocabulary))
			}
			Self::InvalidTypeField => write!(f, "invalid type field"),
			Self::NoTypeVariants(r) => {
				write!(f, "no type variants defined for `{}`", r.with(vocabulary))
			}
			Self::MultipleTypeVariants(r) => write!(
				f,
				"multiple type variants defined for `{}`",
				r.with(vocabulary)
			),
		}
	}
}

async fn generate_test_suite(
	vocabulary: &mut IndexVocabulary,
	mut loader: FsLoader,
	spec: TestSpec,
) -> Result<TokenStream, Box<Error>> {
	use json_ld::{Loader, RdfQuads};

	let json_ld = loader
		.load_with(vocabulary, spec.suite)
		.await
		.map_err(Error::Load)?;

	let mut expanded_json_ld: json_ld::ExpandedDocument<IriIndex, BlankIdIndex> = json_ld
		.expand_with(vocabulary, &mut loader)
		.await
		.map_err(Error::Expand)?;

	let mut generator = rdf_types::generator::Blank::new();
	expanded_json_ld.identify_all_with(vocabulary, &mut generator);

	let rdf_quads = expanded_json_ld.rdf_quads_with(vocabulary, &mut generator, None);
	let dataset: IndexedBTreeDataset<IndexTerm> = rdf_quads.map(quad_to_owned).collect();

	let mut tests = HashMap::new();

	for Quad(subject, predicate, object, graph) in &dataset {
		if graph.is_none() {
			if let IndexTerm::Id(ValidId::Iri(id)) = subject {
				if *predicate
					== IndexTerm::Id(ValidId::Iri(IriIndex::Iri(Vocab::Rdf(vocab::Rdf::Type))))
				{
					if let json_ld::rdf::Value::Id(ValidId::Iri(ty)) = object {
						if let Some(type_id) = spec.type_map.get(ty) {
							match spec.ignore.get(id) {
								Some(link) => {
									println!(
										"    {} test `{}` (see {})",
										yansi::Paint::yellow("Ignoring").bold(),
										vocabulary.iri(id).unwrap(),
										link
									);
								}
								None => {
									tests.insert(*id, type_id);
								}
							}
						}
					}
				}
			}
		}
	}

	let id = &spec.id;
	let mut tokens = TokenStream::new();
	for (test, type_id) in tests {
		let ty = spec.types.get(type_id).unwrap();
		let cons = ty.generate(
			vocabulary,
			&spec,
			&dataset,
			IndexTerm::iri(test),
			quote! { #id :: #type_id },
		)?;

		let func_name = func_name(
			&spec.prefix,
			vocabulary.iri(&test).unwrap().fragment().unwrap().as_str(),
		);
		let func_id = quote::format_ident!("{}", func_name);

		tokens.extend(quote! {
			#[test]
			fn #func_id() {
				#cons.run()
			}
		})
	}

	Ok(tokens)
}

fn test_prefix(name: &str) -> String {
	let mut segments = Vec::new();
	let mut buffer = String::new();

	for c in name.chars() {
		if c.is_uppercase() && !buffer.is_empty() {
			segments.push(buffer);
			buffer = String::new();
		}

		buffer.push(c.to_lowercase().next().unwrap())
	}

	if !buffer.is_empty() {
		segments.push(buffer)
	}

	if segments.len() > 1 && segments.last().unwrap() == "test" {
		segments.pop();
	}

	let mut result = String::new();

	for segment in segments {
		result.push_str(&segment);
		result.push('_')
	}

	result
}

fn func_name(prefix: &str, id: &str) -> String {
	let mut name = prefix.to_string();
	name.push_str(id);
	name
}

fn quad_to_owned(
	rdf_types::Quad(subject, predicate, object, graph): json_ld::rdf::QuadRef<
		IriIndex,
		BlankIdIndex,
		LiteralIndex,
	>,
) -> IndexQuad {
	Quad(
		IndexTerm::Id(*subject.as_ref()),
		IndexTerm::Id(*predicate.as_ref()),
		object,
		graph.copied().map(IndexTerm::Id),
	)
}
