use iref::{Iri, IriBuf};
use json_ld::{
	context::{self, Loader as ContextLoader, Local, ProcessingOptions},
	expansion,
	Document, FsLoader, Loader, ProcessingMode,
	rdf,
	rdf::Display
};
use std::collections::BTreeSet;
use serde_json::Value;
use static_iref::iri;

#[derive(Clone, Copy)]
struct Options<'a> {
	processing_mode: ProcessingMode,
	context: Option<Iri<'a>>,
}

impl<'a> From<Options<'a>> for expansion::Options {
	fn from(options: Options<'a>) -> expansion::Options {
		expansion::Options {
			processing_mode: options.processing_mode,
			ordered: false,
			..expansion::Options::default()
		}
	}
}

impl<'a> From<Options<'a>> for ProcessingOptions {
	fn from(options: Options<'a>) -> ProcessingOptions {
		ProcessingOptions {
			processing_mode: options.processing_mode,
			..ProcessingOptions::default()
		}
	}
}

async fn positive_test(
	options: Options<'_>,
	input_url: Iri<'_>,
	base_url: Iri<'_>,
	output_url: Iri<'_>,
) {
	let mut loader = FsLoader::<Value>::new(|s| serde_json::from_str(s));
	loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");

	let input = loader.load(input_url).await.unwrap();
	let mut input_context: context::Json<Value, IriBuf> = context::Json::new(Some(base_url));

	if let Some(context_url) = options.context {
		let local_context = loader
			.load_context(context_url)
			.await
			.unwrap()
			.into_context();
		input_context = local_context
			.process_with(&input_context, &mut loader, Some(base_url), options.into())
			.await
			.unwrap()
			.into_inner();
	}

	let expected_output = async_std::fs::read_to_string(loader.filepath(output_url.as_iri_ref()).unwrap()).await.expect("unable to read file");

	let doc = input
		.expand_with(Some(base_url), &input_context, &mut loader, options.into())
		.await
		.unwrap();

	let generator = json_ld::id::generator::Blank::new();
	
	let mut lines = BTreeSet::new();
	for rdf::QuadRef(graph, subject, property, object) in doc.rdf_quads(generator, rdf::RdfDirection::I18nDatatype) {
		match graph {
			Some(graph) => lines.insert(format!("{} {} {} {} .\n", subject.rdf_display(), property, object, graph)),
			None => lines.insert(format!("{} {} {} .\n", subject.rdf_display(), property, object))
		};
	}

	let output = itertools::join(lines, "");
	assert_eq!(output, expected_output)
}

#[async_std::test]
async fn to_rdf_0001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0001-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0001-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0001-out.nq");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0002-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0002-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0002-out.nq");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0003() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0003-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0003-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0003-out.nq");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None
		},
		input_url,
		base_url,
		output_url
	).await
}