use iref::{Iri, IriBuf};
use json_ld::{
	context::{self, ContextLoader, Local, ProcessingOptions},
	expansion, rdf,
	rdf::Display,
	util::AsJson,
	Document, ErrorCode, FsLoader, Loader, ProcessingMode,
};
use locspan::Loc;
use serde_json::Value;
use static_iref::iri;
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
struct Options<'a> {
	processing_mode: ProcessingMode,
	context: Option<Iri<'a>>,
	rdf_direction: Option<rdf::RdfDirection>,
	produce_generalized_rdf: bool,
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

fn infallible<T>(t: T) -> Result<T, std::convert::Infallible> {
	Ok(t)
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

	let expected_output =
		async_std::fs::read_to_string(loader.filepath(output_url.as_iri_ref()).unwrap())
			.await
			.expect("unable to read file");

	let doc = input
		.expand_with(Some(base_url), &input_context, &mut loader, options.into())
		.await
		.unwrap();

	let mut generator = json_ld::id::generator::Blank::new_with_prefix("b".to_string());
	let node_map = doc
		.generate_node_map(&mut generator)
		.expect("unable to generate node map");

	let mut lines = BTreeSet::new();
	for rdf::QuadRef(graph, subject, property, object) in
		node_map.rdf_quads(&mut generator, options.rdf_direction)
	{
		if options.produce_generalized_rdf || property.as_iri().is_some() {
			match graph {
				Some(graph) => lines.insert(format!(
					"{} {} {} {} .\n",
					subject.rdf_display(),
					property,
					object,
					graph.rdf_display()
				)),
				None => lines.insert(format!(
					"{} {} {} .\n",
					subject.rdf_display(),
					property,
					object
				)),
			};
		}
	}

	let output = itertools::join(lines, "");

	let mut success = output == expected_output;

	if !success {
		let parsed_output = parse_nquads(&output);
		let parsed_expected_output = parse_nquads(&expected_output);
		success = parsed_output.is_isomorphic_to(&parsed_expected_output);
		if !success {
			let json: Value = doc.as_json();
			eprintln!(
				"expanded:\n{}",
				serde_json::to_string_pretty(&json).unwrap()
			);
			eprintln!("expected:\n{}", expected_output);
			eprintln!("found:\n{}", output);
		}
	}

	assert!(success)
}

fn parse_nquads(buffer: &str) -> grdf::BTreeDataset {
	eprintln!("parse:\n{}", buffer);

	use locspan::Strip;
	use nquads_syntax::Parse;
	let mut lexer = nquads_syntax::Lexer::new(
		(),
		nquads_syntax::lexing::Utf8Decoded::new(buffer.chars().map(infallible)).peekable(),
	);

	match nquads_syntax::GrdfDocument::parse(&mut lexer) {
		Ok(Loc(nquads, _)) => nquads.into_iter().map(Strip::strip).collect(),
		Err(Loc(e, _)) => {
			panic!("parse error: {:?}", e)
		}
	}
}

async fn negative_test(
	options: Options<'_>,
	input_url: Iri<'_>,
	base_url: Iri<'_>,
	error_code: ErrorCode,
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

	let result = input
		.expand_with(Some(base_url), &input_context, &mut loader, options.into())
		.await;

	match result {
		Ok(output) => {
			let output_json: Value = output.as_json();
			println!(
				"output=\n{}",
				serde_json::to_string_pretty(&output_json).unwrap()
			);
			panic!(
				"expansion succeeded where it should have failed with code: {}",
				error_code
			)
		}
		Err(e) => {
			assert_eq!(e.code(), error_code)
		}
	}
}

#[async_std::test]
async fn to_rdf_0001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0001-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0001-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0001-out.nq");
	println!("Plain literal with URIs");
	println!("Tests generation of a triple using full URIs and a plain literal.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
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
	println!("Plain literal with CURIE from default context");
	println!("Tests generation of a triple using a CURIE defined in the default context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
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
	println!("Default subject is BNode");
	println!("Tests that a BNode is created if no explicit subject is set.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0004() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0004-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0004-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0004-out.nq");
	println!("Literal with language tag");
	println!("Tests that a plain literal is created with a language tag.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0005() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0005-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0005-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0005-out.nq");
	println!("Extended character set literal");
	println!("Tests that a literal may be created using extended characters.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0006() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0006-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0006-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0006-out.nq");
	println!("Typed literal");
	println!("Tests creation of a literal with a datatype.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0007() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0007-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0007-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0007-out.nq");
	println!("Tests 'a' generates rdf:type and object is implicit IRI");
	println!("Verify that 'a' is an alias for rdf:type, and the object is created as an IRI.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0008() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0008-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0008-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0008-out.nq");
	println!("Test prefix defined in @context");
	println!("Generate an IRI using a prefix defined within an @context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0009() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0009-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0009-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0009-out.nq");
	println!("Test using an empty suffix");
	println!("An empty suffix may be used.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0010() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0010-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0010-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0010-out.nq");
	println!("Test object processing defines object");
	println!("A property referencing an associative array gets object from subject of array.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0011() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0011-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0011-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0011-out.nq");
	println!("Test object processing defines object with implicit BNode");
	println!("If no @ is specified, a BNode is created, and will be used as the object of an enclosing property.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0012() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0012-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0012-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0012-out.nq");
	println!("Multiple Objects for a Single Property");
	println!("Tests that Multiple Objects are for a Single Property using array syntax.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0013() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0013-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0013-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0013-out.nq");
	println!("Creation of an empty list");
	println!("Tests that @list: [] generates an empty list.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0014() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0014-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0014-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0014-out.nq");
	println!("Creation of a list with single element");
	println!("Tests that @list generates a list.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0015() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0015-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0015-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0015-out.nq");
	println!("Creation of a list with multiple elements");
	println!("Tests that list with multiple elements.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0016() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0016-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0016-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0016-out.nq");
	println!("Empty IRI expands to resource location");
	println!("Expanding an empty IRI uses the test file location.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0017() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0017-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0017-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0017-out.nq");
	println!("Relative IRI expands relative resource location");
	println!("Expanding a relative IRI uses the test file location.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0018() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0018-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0018-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0018-out.nq");
	println!("Frag ID expands relative resource location");
	println!("Expanding a fragment uses the test file location.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0019() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0019-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0019-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0019-out.nq");
	println!("Test type coercion to anyURI");
	println!("Tests coercion of object to anyURI when specified.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0020() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0020-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0020-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0020-out.nq");
	println!("Test type coercion to typed literal");
	println!("Tests coercion of object to a typed literal when specified.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0022() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0022-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0022-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0022-out.nq");
	println!("Test coercion of double value");
	println!("Tests that a decimal value generates a xsd:double typed literal;.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0023() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0023-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0023-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0023-out.nq");
	println!("Test coercion of integer value");
	println!("Tests that a decimal value generates a xsd:integer typed literal.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0024() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0024-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0024-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0024-out.nq");
	println!("Test coercion of boolean value");
	println!("Tests that a decimal value generates a xsd:boolean typed literal.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0025() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0025-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0025-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0025-out.nq");
	println!("Test list coercion with single element");
	println!("Tests that an array with a single element on a property with @list coercion creates an RDF Collection.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0026() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0026-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0026-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0026-out.nq");
	println!("Test creation of multiple types");
	println!("Tests that @type with an array of types creates multiple types.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0027() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0027-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0027-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0027-out.nq");
	println!("Simple named graph (Wikidata)");
	println!("Using @graph with other keys places triples in a named graph.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0028() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0028-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0028-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0028-out.nq");
	println!("Simple named graph");
	println!("Signing a graph.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0029() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0029-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0029-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0029-out.nq");
	println!("named graph with embedded named graph");
	println!("Tests that named graphs containing named graphs flatten to single level of graph naming.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0030() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0030-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0030-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0030-out.nq");
	println!("top-level graph with string subject reference");
	println!("Tests graphs containing subject references as strings.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0031() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0031-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0031-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0031-out.nq");
	println!("Reverse property");
	println!("Tests conversion of reverse properties.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0032() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0032-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0032-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0032-out.nq");
	println!("@context reordering");
	println!("Tests that generated triples do not depend on order of @context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0033() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0033-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0033-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0033-out.nq");
	println!("@id reordering");
	println!("Tests that generated triples do not depend on order of @id.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0034() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0034-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0034-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0034-out.nq");
	println!("context properties reordering");
	println!("Tests that generated triples do not depend on order of properties inside @context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0035() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0035-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0035-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0035-out.nq");
	println!("non-fractional numbers converted to xsd:double");
	println!("xsd:double's canonical lexical is used when converting numbers without fraction that are coerced to xsd:double");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0036() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0036-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0036-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0036-out.nq");
	println!("Use nodeMapGeneration bnode labels");
	println!("The toRDF algorithm does not relabel blank nodes; it reuses the counter from the nodeMapGeneration to generate new ones");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0113() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0113-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0113-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0113-out.nq");
	println!("Dataset with a IRI named graph");
	println!("Basic use of creating a named graph using an IRI name");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0114() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0114-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0114-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0114-out.nq");
	println!("Dataset with a IRI named graph");
	println!("Basic use of creating a named graph using a BNode name");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0115() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0115-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0115-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0115-out.nq");
	println!("Dataset with a default and two named graphs");
	println!("Dataset with a default and two named graphs (IRI and BNode)");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0116() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0116-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0116-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0116-out.nq");
	println!("Dataset from node with embedded named graph");
	println!("Embedding @graph in a node creates a named graph");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0117() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0117-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0117-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0117-out.nq");
	println!("Dataset from node with embedded named graph (bnode)");
	println!("Embedding @graph in a node creates a named graph. Graph name is created if there is no subject");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0119() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0119-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0119-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0119-out.nq");
	println!("Blank nodes with reverse properties");
	println!("Proper (re-)labeling of blank nodes if used with reverse properties.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0120() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0120-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0120-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0120-out.nq");
	println!("IRI Resolution (0)");
	println!("IRI resolution according to RFC3986.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0121() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0121-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0121-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0121-out.nq");
	println!("IRI Resolution (1)");
	println!("IRI resolution according to RFC3986.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0122() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0122-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0122-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0122-out.nq");
	println!("IRI Resolution (2)");
	println!("IRI resolution according to RFC3986.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0123() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0123-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0123-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0123-out.nq");
	println!("IRI Resolution (3)");
	println!("IRI resolution according to RFC3986.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0124() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0124-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0124-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0124-out.nq");
	println!("IRI Resolution (4)");
	println!("IRI resolution according to RFC3986.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0125() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0125-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0125-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0125-out.nq");
	println!("IRI Resolution (5)");
	println!("IRI resolution according to RFC3986.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0126() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0126-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0126-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0126-out.nq");
	println!("IRI Resolution (6)");
	println!("IRI resolution according to RFC3986.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0127() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0127-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0127-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0127-out.nq");
	println!("IRI Resolution (7)");
	println!("IRI resolution according to RFC3986.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0128() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0128-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0128-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0128-out.nq");
	println!("IRI Resolution (8)");
	println!("IRI resolution according to RFC3986.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0129() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0129-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0129-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0129-out.nq");
	println!("IRI Resolution (9)");
	println!("IRI resolution according to RFC3986.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0130() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0130-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0130-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0130-out.nq");
	println!("IRI Resolution (10)");
	println!("IRI resolution according to RFC3986.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0131() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0131-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0131-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0131-out.nq");
	println!("IRI Resolution (11)");
	println!("IRI resolution according to RFC3986.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_0132() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0132-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0132-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/0132-out.nq");
	println!("IRI Resolution (12)");
	println!("IRI resolution according to RFC3986.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c001-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c001-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c001-out.nq");
	println!("adding new term");
	println!("Expansion using a scoped context uses term scope for selecting proper term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c002-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c002-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c002-out.nq");
	println!("overriding a term");
	println!("Expansion using a scoped context uses term scope for selecting proper term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c003() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c003-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c003-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c003-out.nq");
	println!("property and value with different terms mapping to the same expanded property");
	println!("Expansion using a scoped context uses term scope for selecting proper term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c004() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c004-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c004-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c004-out.nq");
	println!("deep @context affects nested nodes");
	println!("Expansion using a scoped context uses term scope for selecting proper term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c005() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c005-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c005-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c005-out.nq");
	println!("scoped context layers on intemediate contexts");
	println!("Expansion using a scoped context uses term scope for selecting proper term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c006() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c006-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c006-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c006-out.nq");
	println!("adding new term");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c007() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c007-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c007-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c007-out.nq");
	println!("overriding a term");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c008() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c008-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c008-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c008-out.nq");
	println!("alias of @type");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c009() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c009-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c009-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c009-out.nq");
	println!("deep @type-scoped @context does NOT affect nested nodes");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c010() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c010-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c010-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c010-out.nq");
	println!("scoped context layers on intemediate contexts");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c011() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c011-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c011-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c011-out.nq");
	println!("orders @type terms when applying scoped contexts");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c012() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c012-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c012-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c012-out.nq");
	println!("deep property-term scoped @context in @type-scoped @context affects nested nodes");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c013() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c013-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c013-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c013-out.nq");
	println!("type maps use scoped context from type index and not scoped context from containing");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c014() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c014-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c014-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c014-out.nq");
	println!("type-scoped context nullification");
	println!("type-scoped context nullification");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c015() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c015-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c015-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c015-out.nq");
	println!("type-scoped base");
	println!("type-scoped base");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c016() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c016-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c016-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c016-out.nq");
	println!("type-scoped vocab");
	println!("type-scoped vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c017() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c017-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c017-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c017-out.nq");
	println!("multiple type-scoped contexts are properly reverted");
	println!("multiple type-scoped contexts are property reverted");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c018() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c018-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c018-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c018-out.nq");
	println!("multiple type-scoped types resolved against previous context");
	println!("multiple type-scoped types resolved against previous context");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c019() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c019-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c019-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c019-out.nq");
	println!("type-scoped context with multiple property scoped terms");
	println!("type-scoped context with multiple property scoped terms");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c020() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c020-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c020-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c020-out.nq");
	println!("type-scoped value");
	println!("type-scoped value");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c021() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c021-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c021-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c021-out.nq");
	println!("type-scoped value mix");
	println!("type-scoped value mix");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c022() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c022-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c022-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c022-out.nq");
	println!("type-scoped property-scoped contexts including @type:@vocab");
	println!("type-scoped property-scoped contexts including @type:@vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c023() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c023-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c023-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c023-out.nq");
	println!("composed type-scoped property-scoped contexts including @type:@vocab");
	println!("composed type-scoped property-scoped contexts including @type:@vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c024() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c024-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c024-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c024-out.nq");
	println!("type-scoped + property-scoped + values evaluates against previous context");
	println!("type-scoped + property-scoped + values evaluates against previous context");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c025() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c025-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c025-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c025-out.nq");
	println!("type-scoped + graph container");
	println!("type-scoped + graph container");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c026() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c026-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c026-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c026-out.nq");
	println!("@propagate: true on type-scoped context");
	println!("type-scoped context with @propagate: true survive node-objects");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c027() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c027-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c027-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c027-out.nq");
	println!("@propagate: false on property-scoped context");
	println!("property-scoped context with @propagate: false do not survive node-objects");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c028() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c028-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c028-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c028-out.nq");
	println!("@propagate: false on embedded context");
	println!("embedded context with @propagate: false do not survive node-objects");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c029() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c029-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c029-in.jsonld");
	println!("@propagate is invalid in 1.0");
	println!("@propagate is invalid in 1.0");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidContextEntry
	).await
}

#[async_std::test]
async fn to_rdf_c030() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c030-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c030-in.jsonld");
	println!("@propagate must be boolean valued");
	println!("@propagate must be boolean valued");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidPropagateValue
	).await
}

#[async_std::test]
async fn to_rdf_c031() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c031-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c031-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c031-out.nq");
	println!("@context resolutions respects relative URLs.");
	println!("URL resolution follows RFC3986");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c032() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c032-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c032-in.jsonld");
	println!("Unused embedded context with error.");
	println!("An embedded context which is never used should still be checked.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidScopedContext
	).await
}

#[async_std::test]
async fn to_rdf_c033() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c033-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c033-in.jsonld");
	println!("Unused context with an embedded context error.");
	println!("An unused context with an embedded context should still be checked.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidScopedContext
	).await
}

#[async_std::test]
async fn to_rdf_c034() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c034-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c034-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c034-out.nq");
	println!("Remote scoped context.");
	println!("Scoped contexts may be externally loaded.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c035() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c035-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c035-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c035-out.nq");
	println!("Term scoping with embedded contexts.");
	println!("Terms should make use of @vocab relative to the scope in which the term was defined.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_c036() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c036-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c036-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/c036-out.nq");
	println!("Expansion with empty property-scoped context.");
	println!("Adding a minimal/empty property-scoped context should not affect expansion of terms defined in the outer scope.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_di01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di01-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di01-out.nq");
	println!("Expand string using default and term directions");
	println!("Strings are coerced to have @direction based on default and term direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_di02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di02-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di02-out.nq");
	println!("Expand string using default and term directions and languages");
	println!("Strings are coerced to have @direction based on default and term direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_di03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di03-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di03-out.nq");
	println!("expand list values with @direction");
	println!("List values where the term has @direction are used in expansion.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_di04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di04-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di04-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di04-out.nq");
	println!("simple language map with term direction");
	println!("Term selection with language maps and @direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_di05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di05-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di05-out.nq");
	println!("simple language mapwith overriding term direction");
	println!("Term selection with language maps and @direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_di06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di06-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di06-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di06-out.nq");
	println!("simple language mapwith overriding null direction");
	println!("Term selection with language maps and @direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_di07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di07-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di07-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di07-out.nq");
	println!("simple language map with mismatching term direction");
	println!("Term selection with language maps and @direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_di08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di08-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/di08-in.jsonld");
	println!("@direction must be one of ltr or rtl");
	println!("Generate an error if @direction has illegal value.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidBaseDirection
	).await
}

#[async_std::test]
async fn to_rdf_e001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e001-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e001-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e001-out.nq");
	println!("drop free-floating nodes");
	println!("Free-floating nodes do not generate RDF triples (from expand-0001)");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e002-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e002-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e002-out.nq");
	println!("basic");
	println!("Basic RDF conversion (from expand-0002)");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e003() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e003-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e003-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e003-out.nq");
	println!("drop null and unmapped properties");
	println!("Properties mapped to null or which are never mapped are dropped (from expand-0003)");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e004() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e004-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e004-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e004-out.nq");
	println!("optimize @set, keep empty arrays");
	println!("RDF version of expand-0004");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e005() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e005-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e005-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e005-out.nq");
	println!("do not expand aliased @id/@type");
	println!("RDF version of expand-0005");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e006() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e006-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e006-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e006-out.nq");
	println!("alias keywords");
	println!("RDF version of expand-0006");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e007() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e007-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e007-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e007-out.nq");
	println!("date type-coercion");
	println!("Type-coerced dates generate typed literals (from expand-0007)");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e008() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e008-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e008-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e008-out.nq");
	println!("@value with @language");
	println!("RDF version of expand-0008");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e009() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e009-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e009-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e009-out.nq");
	println!("@graph with terms");
	println!("RDF version of expand-0009");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e010() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e010-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e010-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e010-out.nq");
	println!("native types");
	println!("Native types generate typed literals (from expand-0010)");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e011() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e011-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e011-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e011-out.nq");
	println!("coerced @id");
	println!("RDF version of expand-0011");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e012() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e012-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e012-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e012-out.nq");
	println!("@graph with embed");
	println!("RDF version of expand-0012");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e013() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e013-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e013-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e013-out.nq");
	println!("expand already expanded");
	println!("RDF version of expand-0013");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e015() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e015-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e015-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e015-out.nq");
	println!("collapse set of sets, keep empty lists");
	println!("RDF version of expand-0015");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e016() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e016-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e016-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e016-out.nq");
	println!("context reset");
	println!("RDF version of expand-0016");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e017() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e017-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e017-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e017-out.nq");
	println!("@graph and @id aliased");
	println!("RDF version of expand-0017");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e018() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e018-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e018-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e018-out.nq");
	println!("override default @language");
	println!("RDF version of expand-0018");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e019() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e019-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e019-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e019-out.nq");
	println!("remove @value = null");
	println!("RDF version of expand-0019");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e020() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e020-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e020-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e020-out.nq");
	println!("do not remove @graph if not at top-level");
	println!("Embedded @graph without @id creates BNode-labeled named graph (from expand-0020)");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e021() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e021-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e021-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e021-out.nq");
	println!("do not remove @graph at top-level if not only property");
	println!("RDF version of expand-0021");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e022() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e022-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e022-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e022-out.nq");
	println!("expand value with default language");
	println!("RDF version of expand-0022");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e023() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e023-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e023-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e023-out.nq");
	println!("Lists and sets of properties with list/set coercion");
	println!("RDF version of expand-0023");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e024() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e024-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e024-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e024-out.nq");
	println!("Multiple contexts");
	println!("RDF version of expand-0024");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e025() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e025-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e025-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e025-out.nq");
	println!("Problematic IRI expansion tests");
	println!("RDF version of expand-0025");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e027() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e027-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e027-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e027-out.nq");
	println!("Keep duplicate values in @list and @set");
	println!("RDF version of expand-0027");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e028() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e028-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e028-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e028-out.nq");
	println!("Use @vocab in properties and @type but not in @id");
	println!("RDF version of expand-0028");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e029() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e029-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e029-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e029-out.nq");
	println!("Relative IRIs");
	println!("RDF version of expand-0029");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e030() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e030-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e030-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e030-out.nq");
	println!("Language maps");
	println!("RDF version of expand-0030");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e031() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e031-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e031-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e031-out.nq");
	println!("type-coercion of native types");
	println!("RDF version of expand-0031");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e032() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e032-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e032-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e032-out.nq");
	println!("Mapping a term to null decouples it from @vocab");
	println!("RDF version of expand-0032");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e033() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e033-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e033-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e033-out.nq");
	println!("Using @vocab with with type-coercion");
	println!("RDF version of expand-0033");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e034() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e034-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e034-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e034-out.nq");
	println!("Multiple properties expanding to the same IRI");
	println!("RDF version of expand-0034");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e035() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e035-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e035-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e035-out.nq");
	println!("Language maps with @vocab, default language, and colliding property");
	println!("RDF version of expand-0035");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e036() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e036-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e036-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e036-out.nq");
	println!("Expanding @index");
	println!("RDF version of expand-0036");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e037() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e037-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e037-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e037-out.nq");
	println!("Expanding @reverse");
	println!("RDF version of expand-0037");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e039() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e039-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e039-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e039-out.nq");
	println!("Using terms in a reverse-maps");
	println!("RDF version of expand-0039");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e040() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e040-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e040-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e040-out.nq");
	println!("language and index expansion on non-objects");
	println!("RDF version of expand-0040");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e041() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e041-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e041-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e041-out.nq");
	println!("Reset the default language");
	println!("RDF version of expand-0041");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e042() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e042-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e042-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e042-out.nq");
	println!("Expanding reverse properties");
	println!("RDF version of expand-0042");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e043() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e043-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e043-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e043-out.nq");
	println!("Using reverse properties inside a @reverse-container");
	println!("RDF version of expand-0043");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e044() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e044-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e044-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e044-out.nq");
	println!("Ensure index maps use language mapping");
	println!("RDF version of expand-0044");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e045() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e045-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e045-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e045-out.nq");
	println!("Top-level value objects are removed");
	println!("RDF version of expand-0045");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e046() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e046-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e046-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e046-out.nq");
	println!("Free-floating nodes are removed");
	println!("RDF version of expand-0046");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e047() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e047-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e047-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e047-out.nq");
	println!("Remove free-floating set values and lists");
	println!("RDF version of expand-0047");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e048() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e048-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e048-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e048-out.nq");
	println!("Terms are ignored in @id");
	println!("RDF version of expand-0048");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e049() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e049-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e049-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e049-out.nq");
	println!("Using strings as value of a reverse property");
	println!("RDF version of expand-0049");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e050() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e050-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e050-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e050-out.nq");
	println!("Term definitions with prefix separate from prefix definitions");
	println!("RDF version of expand-0050");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e051() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e051-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e051-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e051-out.nq");
	println!("Expansion of keyword aliases in term definitions");
	println!("RDF version of expand-0051");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e052() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e052-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e052-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e052-out.nq");
	println!("@vocab-relative IRIs in term definitions");
	println!("RDF version of expand-0052");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e053() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e053-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e053-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e053-out.nq");
	println!("Expand absolute IRI with @type: @vocab");
	println!("RDF version of expand-0053");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e054() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e054-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e054-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e054-out.nq");
	println!("Expand term with @type: @vocab");
	println!("RDF version of expand-0054");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e055() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e055-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e055-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e055-out.nq");
	println!("Expand @vocab-relative term with @type: @vocab");
	println!("RDF version of expand-0055");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e056() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e056-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e056-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e056-out.nq");
	println!("Use terms with @type: @vocab but not with @type: @id");
	println!("RDF version of expand-0056");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e057() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e057-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e057-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e057-out.nq");
	println!("Expand relative IRI with @type: @vocab");
	println!("RDF version of expand-0057");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e058() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e058-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e058-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e058-out.nq");
	println!("Expand compact IRI with @type: @vocab");
	println!("RDF version of expand-0058");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e059() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e059-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e059-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e059-out.nq");
	println!("Reset @vocab by setting it to null");
	println!("RDF version of expand-0059");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e060() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e060-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e060-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e060-out.nq");
	println!("Overwrite document base with @base and reset it again");
	println!("RDF version of expand-0060");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e061() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e061-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e061-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e061-out.nq");
	println!("Coercing native types to arbitrary datatypes");
	println!("RDF version of expand-0061");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e062() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e062-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e062-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e062-out.nq");
	println!("Various relative IRIs with with @base");
	println!("RDF version of expand-0062");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e063() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e063-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e063-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e063-out.nq");
	println!("Expand a reverse property with an index-container");
	println!("RDF version of expand-0063");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e064() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e064-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e064-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e064-out.nq");
	println!("Expand reverse property whose values are unlabeled blank nodes");
	println!("RDF version of expand-0064");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e065() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e065-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e065-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e065-out.nq");
	println!("Keys that are not mapped to an IRI in a reverse-map are dropped");
	println!("RDF version of expand-0065");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e066() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e066-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e066-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e066-out.nq");
	println!("Use @vocab to expand keys in reverse-maps");
	println!("RDF version of expand-0066");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e067() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e067-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e067-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e067-out.nq");
	println!("prefix:://sufffix not a compact IRI");
	println!("RDF version of expand-0067");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e068() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e068-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e068-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e068-out.nq");
	println!("_::sufffix not a compact IRI");
	println!("RDF version of expand-0068");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e069() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e069-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e069-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e069-out.nq");
	println!("Compact IRI as term with type mapping");
	println!("RDF version of expand-0069");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e070() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e070-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e070-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e070-out.nq");
	println!("Redefine compact IRI with itself");
	println!("RDF version of expand-0070");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e072() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e072-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e072-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e072-out.nq");
	println!("Redefine term using @vocab, not itself");
	println!("RDF version of expand-0072");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e073() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e073-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e073-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e073-out.nq");
	println!("@context not first property");
	println!("Objects are unordered, so serialized node definition containing @context may have @context at the end of the node definition");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e074() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e074-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e074-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e074-out.nq");
	println!("@id not first property");
	println!("Objects are unordered, so serialized node definition containing @id may have @id at the end of the node definition");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e075() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e075-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e075-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e075-out.nq");
	println!("@vocab as blank node identifier");
	println!("Use @vocab to map all properties to blank node identifiers");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: true
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e076() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e076-in.jsonld");
	let base_url = iri!("http://example/base/");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e076-out.nq");
	println!("base option overrides document location");
	println!("Use of the base option overrides the document location");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e077() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e077-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e077-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e077-out.nq");
	println!("expandContext option");
	println!("Use of the expandContext option to expand the input document");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: Some(iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e077-context.jsonld")),
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e078() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e078-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e078-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e078-out.nq");
	println!("multiple reverse properties");
	println!("Use of multiple reverse properties");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e079() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e079-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e079-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e079-out.nq");
	println!("expand @graph container");
	println!("Use of @graph containers");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e080() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e080-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e080-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e080-out.nq");
	println!("expand [@graph, @set] container");
	println!("Use of [@graph, @set] containers");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e081() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e081-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e081-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e081-out.nq");
	println!("Creates an @graph container if value is a graph");
	println!("Don't double-expand an already expanded graph");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e082() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e082-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e082-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e082-out.nq");
	println!("expand [@graph, @index] container");
	println!("Use of @graph containers with @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e083() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e083-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e083-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e083-out.nq");
	println!("expand [@graph, @index, @set] container");
	println!("Use of @graph containers with @index and @set");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e084() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e084-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e084-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e084-out.nq");
	println!("Do not expand [@graph, @index] container if value is a graph");
	println!("Does not create a new graph object if indexed value is already a graph object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e085() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e085-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e085-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e085-out.nq");
	println!("expand [@graph, @id] container");
	println!("Use of @graph containers with @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e086() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e086-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e086-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e086-out.nq");
	println!("expand [@graph, @id, @set] container");
	println!("Use of @graph containers with @id and @set");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e087() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e087-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e087-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e087-out.nq");
	println!("Do not expand [@graph, @id] container if value is a graph");
	println!("Does not create a new graph object if indexed value is already a graph object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e088() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e088-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e088-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e088-out.nq");
	println!("Do not expand native values to IRIs");
	println!("Value Expansion does not expand native values, such as booleans, to a node object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e089() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e089-in.jsonld");
	let base_url = iri!("http://example/base/");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e089-out.nq");
	println!("empty @base applied to the base option");
	println!("Use of an empty @base is applied to the base option");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e090() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e090-in.jsonld");
	let base_url = iri!("http://example/base/");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e090-out.nq");
	println!("relative @base overrides base option and document location");
	println!("Use of a relative @base overrides base option and document location");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e091() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e091-in.jsonld");
	let base_url = iri!("http://example/base/");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e091-out.nq");
	println!("relative and absolute @base overrides base option and document location");
	println!("Use of a relative and absolute @base overrides base option and document location");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e092() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e092-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e092-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e092-out.nq");
	println!("Various relative IRIs as properties with with @vocab: ''");
	println!("Pathological relative property IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e093() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e093-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e093-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e093-out.nq");
	println!("expand @graph container (multiple objects)");
	println!("Use of @graph containers");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e094() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e094-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e094-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e094-out.nq");
	println!("expand [@graph, @set] container (multiple objects)");
	println!("Use of [@graph, @set] containers");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e095() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e095-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e095-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e095-out.nq");
	println!("Creates an @graph container if value is a graph (multiple objects)");
	println!("Don't double-expand an already expanded graph");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e096() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e096-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e096-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e096-out.nq");
	println!("expand [@graph, @index] container (multiple indexed objects)");
	println!("Use of @graph containers with @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e097() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e097-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e097-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e097-out.nq");
	println!("expand [@graph, @index, @set] container (multiple objects)");
	println!("Use of @graph containers with @index and @set");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e098() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e098-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e098-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e098-out.nq");
	println!("Do not expand [@graph, @index] container if value is a graph (multiple objects)");
	println!("Does not create a new graph object if indexed value is already a graph object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e099() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e099-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e099-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e099-out.nq");
	println!("expand [@graph, @id] container (multiple objects)");
	println!("Use of @graph containers with @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e100() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e100-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e100-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e100-out.nq");
	println!("expand [@graph, @id, @set] container (multiple objects)");
	println!("Use of @graph containers with @id and @set");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e101() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e101-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e101-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e101-out.nq");
	println!("Do not expand [@graph, @id] container if value is a graph (multiple objects)");
	println!("Does not create a new graph object if indexed value is already a graph object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e102() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e102-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e102-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e102-out.nq");
	println!("Expand @graph container if value is a graph (multiple objects)");
	println!("Does not create a new graph object if indexed value is already a graph object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e103() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e103-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e103-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e103-out.nq");
	println!("Expand @graph container if value is a graph (multiple graphs)");
	println!("Does not create a new graph object if indexed value is already a graph object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e104() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e104-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e104-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e104-out.nq");
	println!("Creates an @graph container if value is a graph (mixed graph and object)");
	println!("Don't double-expand an already expanded graph");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e105() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e105-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e105-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e105-out.nq");
	println!("Do not expand [@graph, @index] container if value is a graph (mixed graph and object)");
	println!("Does not create a new graph object if indexed value is already a graph object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e106() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e106-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e106-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e106-out.nq");
	println!("Do not expand [@graph, @id] container if value is a graph (mixed graph and object)");
	println!("Does not create a new graph object if indexed value is already a graph object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e107() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e107-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e107-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e107-out.nq");
	println!("expand [@graph, @index] container (indexes with multiple objects)");
	println!("Use of @graph containers with @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e108() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e108-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e108-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e108-out.nq");
	println!("expand [@graph, @id] container (multiple ids and objects)");
	println!("Use of @graph containers with @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e109() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e109-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e109-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e109-out.nq");
	println!("IRI expansion of fragments including ':'");
	println!("Do not treat as absolute IRIs values that look like compact IRIs if they're not absolute");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e110() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e110-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e110-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e110-out.nq");
	println!("Various relative IRIs as properties with with relative @vocab");
	println!("Pathological relative property IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e111() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e111-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e111-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e111-out.nq");
	println!("Various relative IRIs as properties with with relative @vocab itself relative to an existing vocabulary base");
	println!("Pathological relative property IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e112() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e112-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e112-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e112-out.nq");
	println!("Various relative IRIs as properties with with relative @vocab relative to another relative vocabulary base");
	println!("Pathological relative property IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e113() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e113-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e113-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e113-out.nq");
	println!("context with JavaScript Object property names");
	println!("Expand with context including JavaScript Object property names");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e114() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e114-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e114-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e114-out.nq");
	println!("Expansion allows multiple properties expanding to @type");
	println!("An exception for the colliding keywords error is made for @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e117() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e117-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e117-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e117-out.nq");
	println!("A term starting with a colon can expand to a different IRI");
	println!("Terms may begin with a colon and not be treated as IRIs.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e118() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e118-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e118-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e118-out.nq");
	println!("Expanding a value staring with a colon does not treat that value as an IRI");
	println!("Terms may begin with a colon and not be treated as IRIs.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e119() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e119-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e119-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e119-out.nq");
	println!("Ignore some terms with @, allow others.");
	println!("Processors SHOULD generate a warning and MUST ignore terms having the form of a keyword.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e120() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e120-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e120-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e120-out.nq");
	println!("Ignore some values of @id with @, allow others.");
	println!("Processors SHOULD generate a warning and MUST ignore values of @id having the form of a keyword.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e121() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e121-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e121-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e121-out.nq");
	println!("Ignore some values of @reverse with @, allow others.");
	println!("Processors SHOULD generate a warning and MUST ignore values of @reverse having the form of a keyword.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e123() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e123-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e123-in.jsonld");
	println!("Value objects including invalid literal datatype IRIs are rejected");
	println!("Processors MUST validate datatype IRIs.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTypedValue
	).await
}

#[async_std::test]
async fn to_rdf_e124() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e124-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e124-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e124-out.nq");
	println!("compact IRI as @vocab");
	println!("Verifies that @vocab defined as a compact IRI expands properly");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e125() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e125-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e125-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e125-out.nq");
	println!("term as @vocab");
	println!("Verifies that @vocab defined as a term expands properly");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e126() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e126-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e126-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e126-out.nq");
	println!("A scoped context may include itself recursively (direct)");
	println!("Verifies that no exception is raised on expansion when processing a scoped context referencing itself directly");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e127() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e127-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e127-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e127-out.nq");
	println!("A scoped context may include itself recursively (indirect)");
	println!("Verifies that no exception is raised on expansion when processing a scoped context referencing itself indirectly");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e128() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e128-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e128-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e128-out.nq");
	println!("Two scoped context may include a shared context");
	println!("Verifies that no exception is raised on expansion when processing two scoped contexts referencing a shared context");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e129() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e129-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e129-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e129-out.nq");
	println!("Base without trailing slash, without path");
	println!("Verify URI resolution relative to base (without trailing slash, without path) according to RFC 3986");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_e130() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e130-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e130-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/e130-out.nq");
	println!("Base without trailing slash, with path");
	println!("Verify URI resolution relative to base (without trailing slash, with path) according to RFC 3986");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_ec01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/ec01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/ec01-in.jsonld");
	println!("Invalid keyword in term definition");
	println!("Verifies that an exception is raised on expansion when a invalid term definition is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTermDefinition
	).await
}

#[async_std::test]
async fn to_rdf_ec02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/ec02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/ec02-in.jsonld");
	println!("Term definition on @type with empty map");
	println!("Verifies that an exception is raised if @type is defined as a term with an empty map");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::KeywordRedefinition
	).await
}

#[async_std::test]
async fn to_rdf_em01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/em01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/em01-in.jsonld");
	println!("Invalid container mapping");
	println!("Verifies that an exception is raised on expansion when a invalid container mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidContainerMapping
	).await
}

#[async_std::test]
async fn to_rdf_en01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/en01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/en01-in.jsonld");
	println!("@nest MUST NOT have a string value");
	println!("container: @nest");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidNestValue
	).await
}

#[async_std::test]
async fn to_rdf_en02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/en02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/en02-in.jsonld");
	println!("@nest MUST NOT have a boolen value");
	println!("Transparent Nesting");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidNestValue
	).await
}

#[async_std::test]
async fn to_rdf_en03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/en03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/en03-in.jsonld");
	println!("@nest MUST NOT have a numeric value");
	println!("Transparent Nesting");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidNestValue
	).await
}

#[async_std::test]
async fn to_rdf_en04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/en04-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/en04-in.jsonld");
	println!("@nest MUST NOT have a value object value");
	println!("Transparent Nesting");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidNestValue
	).await
}

#[async_std::test]
async fn to_rdf_en05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/en05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/en05-in.jsonld");
	println!("does not allow a keyword other than @nest for the value of @nest");
	println!("Transparent Nesting");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidNestValue
	).await
}

#[async_std::test]
async fn to_rdf_en06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/en06-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/en06-in.jsonld");
	println!("does not allow @nest with @reverse");
	println!("Transparent Nesting");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidReverseProperty
	).await
}

#[async_std::test]
async fn to_rdf_ep02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/ep02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/ep02-in.jsonld");
	println!("processingMode json-ld-1.0 conflicts with @version: 1.1");
	println!("If processingMode is explicitly json-ld-1.0, it will conflict with 1.1 features.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::ProcessingModeConflict
	).await
}

#[async_std::test]
async fn to_rdf_ep03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/ep03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/ep03-in.jsonld");
	println!("@version must be 1.1");
	println!("If @version is specified, it must be 1.1");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidVersionValue
	).await
}

#[async_std::test]
async fn to_rdf_er01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er01-in.jsonld");
	println!("Keywords cannot be aliased to other keywords");
	println!("Verifies that an exception is raised on expansion when processing an invalid context aliasing a keyword to another keyword");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::KeywordRedefinition
	).await
}

#[async_std::test]
async fn to_rdf_er04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er04-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er04-in.jsonld");
	println!("Error dereferencing a remote context");
	println!("Verifies that an exception is raised on expansion when a context dereference results in an error");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::LoadingRemoteContextFailed
	).await
}

#[async_std::test]
async fn to_rdf_er05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er05-in.jsonld");
	println!("Invalid remote context");
	println!("Verifies that an exception is raised on expansion when a remote context is not an object containing @context");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidRemoteContext
	).await
}

#[async_std::test]
async fn to_rdf_er06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er06-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er06-in.jsonld");
	println!("Invalid local context");
	println!("Verifies that an exception is raised on expansion when a context is not a string or object");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidLocalContext
	).await
}

#[async_std::test]
async fn to_rdf_er07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er07-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er07-in.jsonld");
	println!("Invalid base IRI");
	println!("Verifies that an exception is raised on expansion when a context contains an invalid @base");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidBaseIri
	).await
}

#[async_std::test]
async fn to_rdf_er08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er08-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er08-in.jsonld");
	println!("Invalid vocab mapping");
	println!("Verifies that an exception is raised on expansion when a context contains an invalid @vocab mapping");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidVocabMapping
	).await
}

#[async_std::test]
async fn to_rdf_er09() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er09-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er09-in.jsonld");
	println!("Invalid default language");
	println!("Verifies that an exception is raised on expansion when a context contains an invalid @language");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidDefaultLanguage
	).await
}

#[async_std::test]
async fn to_rdf_er10() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er10-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er10-in.jsonld");
	println!("Cyclic IRI mapping");
	println!("Verifies that an exception is raised on expansion when a cyclic IRI mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::CyclicIriMapping
	).await
}

#[async_std::test]
async fn to_rdf_er11() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er11-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er11-in.jsonld");
	println!("Invalid term definition");
	println!("Verifies that an exception is raised on expansion when a invalid term definition is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTermDefinition
	).await
}

#[async_std::test]
async fn to_rdf_er12() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er12-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er12-in.jsonld");
	println!("Invalid type mapping (not a string)");
	println!("Verifies that an exception is raised on expansion when a invalid type mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTypeMapping
	).await
}

#[async_std::test]
async fn to_rdf_er13() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er13-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er13-in.jsonld");
	println!("Invalid type mapping (not absolute IRI)");
	println!("Verifies that an exception is raised on expansion when a invalid type mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTypeMapping
	).await
}

#[async_std::test]
async fn to_rdf_er14() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er14-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er14-in.jsonld");
	println!("Invalid reverse property (contains @id)");
	println!("Verifies that an exception is raised on expansion when a invalid reverse property is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidReverseProperty
	).await
}

#[async_std::test]
async fn to_rdf_er15() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er15-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er15-in.jsonld");
	println!("Invalid IRI mapping (@reverse not a string)");
	println!("Verifies that an exception is raised on expansion when a invalid IRI mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidIriMapping
	).await
}

#[async_std::test]
async fn to_rdf_er17() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er17-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er17-in.jsonld");
	println!("Invalid reverse property (invalid @container)");
	println!("Verifies that an exception is raised on expansion when a invalid reverse property is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidReverseProperty
	).await
}

#[async_std::test]
async fn to_rdf_er18() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er18-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er18-in.jsonld");
	println!("Invalid IRI mapping (@id not a string)");
	println!("Verifies that an exception is raised on expansion when a invalid IRI mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidIriMapping
	).await
}

#[async_std::test]
async fn to_rdf_er19() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er19-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er19-in.jsonld");
	println!("Invalid keyword alias (@context)");
	println!("Verifies that an exception is raised on expansion when a invalid keyword alias is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidKeywordAlias
	).await
}

#[async_std::test]
async fn to_rdf_er20() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er20-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er20-in.jsonld");
	println!("Invalid IRI mapping (no vocab mapping)");
	println!("Verifies that an exception is raised on expansion when a invalid IRI mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidIriMapping
	).await
}

#[async_std::test]
async fn to_rdf_er21() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er21-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er21-in.jsonld");
	println!("Invalid container mapping");
	println!("Verifies that an exception is raised on expansion when a invalid container mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidContainerMapping
	).await
}

#[async_std::test]
async fn to_rdf_er22() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er22-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er22-in.jsonld");
	println!("Invalid language mapping");
	println!("Verifies that an exception is raised on expansion when a invalid language mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidLanguageMapping
	).await
}

#[async_std::test]
async fn to_rdf_er23() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er23-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er23-in.jsonld");
	println!("Invalid IRI mapping (relative IRI in @type)");
	println!("Verifies that an exception is raised on expansion when a invalid type mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTypeMapping
	).await
}

#[async_std::test]
async fn to_rdf_er25() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er25-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er25-in.jsonld");
	println!("Invalid reverse property map");
	println!("Verifies that an exception is raised in Expansion when a invalid reverse property map is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidReversePropertyMap
	).await
}

#[async_std::test]
async fn to_rdf_er26() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er26-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er26-in.jsonld");
	println!("Colliding keywords");
	println!("Verifies that an exception is raised in Expansion when colliding keywords are found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::CollidingKeywords
	).await
}

#[async_std::test]
async fn to_rdf_er27() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er27-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er27-in.jsonld");
	println!("Invalid @id value");
	println!("Verifies that an exception is raised in Expansion when an invalid @id value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidIdValue
	).await
}

#[async_std::test]
async fn to_rdf_er28() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er28-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er28-in.jsonld");
	println!("Invalid type value");
	println!("Verifies that an exception is raised in Expansion when an invalid type value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTypeValue
	).await
}

#[async_std::test]
async fn to_rdf_er29() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er29-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er29-in.jsonld");
	println!("Invalid value object value");
	println!("Verifies that an exception is raised in Expansion when an invalid value object value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidValueObjectValue
	).await
}

#[async_std::test]
async fn to_rdf_er30() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er30-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er30-in.jsonld");
	println!("Invalid language-tagged string");
	println!("Verifies that an exception is raised in Expansion when an invalid language-tagged string value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidLanguageTaggedString
	).await
}

#[async_std::test]
async fn to_rdf_er31() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er31-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er31-in.jsonld");
	println!("Invalid @index value");
	println!("Verifies that an exception is raised in Expansion when an invalid @index value value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidIndexValue
	).await
}

#[async_std::test]
async fn to_rdf_er33() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er33-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er33-in.jsonld");
	println!("Invalid @reverse value");
	println!("Verifies that an exception is raised in Expansion when an invalid @reverse value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidReverseValue
	).await
}

#[async_std::test]
async fn to_rdf_er34() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er34-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er34-in.jsonld");
	println!("Invalid reverse property value (in @reverse)");
	println!("Verifies that an exception is raised in Expansion when an invalid reverse property value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidReversePropertyValue
	).await
}

#[async_std::test]
async fn to_rdf_er35() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er35-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er35-in.jsonld");
	println!("Invalid language map value");
	println!("Verifies that an exception is raised in Expansion when an invalid language map value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidLanguageMapValue
	).await
}

#[async_std::test]
async fn to_rdf_er36() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er36-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er36-in.jsonld");
	println!("Invalid reverse property value (through coercion)");
	println!("Verifies that an exception is raised in Expansion when an invalid reverse property value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidReversePropertyValue
	).await
}

#[async_std::test]
async fn to_rdf_er37() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er37-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er37-in.jsonld");
	println!("Invalid value object (unexpected keyword)");
	println!("Verifies that an exception is raised in Expansion when an invalid value object is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidValueObject
	).await
}

#[async_std::test]
async fn to_rdf_er38() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er38-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er38-in.jsonld");
	println!("Invalid value object (@type and @language)");
	println!("Verifies that an exception is raised in Expansion when an invalid value object is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidValueObject
	).await
}

#[async_std::test]
async fn to_rdf_er39() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er39-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er39-in.jsonld");
	println!("Invalid language-tagged value");
	println!("Verifies that an exception is raised in Expansion when an invalid language-tagged value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidLanguageTaggedValue
	).await
}

#[async_std::test]
async fn to_rdf_er40() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er40-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er40-in.jsonld");
	println!("Invalid typed value");
	println!("Verifies that an exception is raised in Expansion when an invalid typed value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTypedValue
	).await
}

#[async_std::test]
async fn to_rdf_er41() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er41-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er41-in.jsonld");
	println!("Invalid set or list object");
	println!("Verifies that an exception is raised in Expansion when an invalid set or list object is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidSetOrListObject
	).await
}

#[async_std::test]
async fn to_rdf_er42() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er42-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er42-in.jsonld");
	println!("Keywords may not be redefined in 1.0");
	println!("Verifies that an exception is raised on expansion when processing an invalid context attempting to define @container on a keyword");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::KeywordRedefinition
	).await
}

#[async_std::test]
async fn to_rdf_er43() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er43-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er43-in.jsonld");
	println!("Term definition with @id: @type");
	println!("Expanding term mapping to @type uses @type syntax now illegal");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidIriMapping
	).await
}

#[async_std::test]
async fn to_rdf_er44() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er44-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er44-in.jsonld");
	println!("Redefine terms looking like compact IRIs");
	println!("Term definitions may look like compact IRIs, but must be consistent.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidIriMapping
	).await
}

#[async_std::test]
async fn to_rdf_er48() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er48-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er48-in.jsonld");
	println!("Invalid term as relative IRI");
	println!("Verifies that a relative IRI cannot be used as a term.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidIriMapping
	).await
}

#[async_std::test]
async fn to_rdf_er49() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er49-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er49-in.jsonld");
	println!("A relative IRI cannot be used as a prefix");
	println!("Verifies that a relative IRI cannot be used as a term.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTermDefinition
	).await
}

#[async_std::test]
async fn to_rdf_er50() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er50-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er50-in.jsonld");
	println!("Invalid reverse id");
	println!("Verifies that an exception is raised in Expansion when an invalid IRI is used for @reverse.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidIriMapping
	).await
}

#[async_std::test]
async fn to_rdf_er51() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er51-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er51-in.jsonld");
	println!("Invalid value object value using a value alias");
	println!("Verifies that an exception is raised in Expansion when an invalid value object value is found using a value alias");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidValueObjectValue
	).await
}

#[async_std::test]
async fn to_rdf_er52() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er52-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er52-in.jsonld");
	println!("Definition for the empty term");
	println!("Verifies that an exception is raised on expansion when a context contains a definition for the empty term");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTermDefinition
	).await
}

#[async_std::test]
async fn to_rdf_er53() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er53-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er53-in.jsonld");
	println!("Invalid prefix value");
	println!("Verifies that an exception is raised on expansion when a context contains an invalid @prefix value");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidPrefixValue
	).await
}

#[async_std::test]
async fn to_rdf_er54() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er54-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er54-in.jsonld");
	println!("Invalid value object, multiple values for @type.");
	println!("The value of @type in a value object MUST be a string or null.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTypedValue
	).await
}

#[async_std::test]
async fn to_rdf_er55() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er55-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/er55-in.jsonld");
	println!("Invalid term definition, multiple values for @type.");
	println!("The value of @type in an expanded term definition object MUST be a string or null.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTypeMapping
	).await
}

#[async_std::test]
async fn to_rdf_in01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in01-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in01-out.nq");
	println!("Basic Included array");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_in02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in02-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in02-out.nq");
	println!("Basic Included object");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_in03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in03-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in03-out.nq");
	println!("Multiple properties mapping to @included are folded together");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_in04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in04-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in04-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in04-out.nq");
	println!("Included containing @included");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_in05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in05-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in05-out.nq");
	println!("Property value with @included");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_in06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in06-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in06-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in06-out.nq");
	println!("json.api example");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_in07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in07-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in07-in.jsonld");
	println!("Error if @included value is a string");
	println!("Tests included blocks.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidIncludedValue
	).await
}

#[async_std::test]
async fn to_rdf_in08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in08-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in08-in.jsonld");
	println!("Error if @included value is a value object");
	println!("Tests included blocks.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidIncludedValue
	).await
}

#[async_std::test]
async fn to_rdf_in09() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in09-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/in09-in.jsonld");
	println!("Error if @included value is a list object");
	println!("Tests included blocks.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidIncludedValue
	).await
}

#[async_std::test]
async fn to_rdf_js01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js01-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js01-out.nq");
	println!("Transform JSON literal (boolean true)");
	println!("Tests transforming property with @type @json to a JSON literal (boolean true).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js02-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js02-out.nq");
	println!("Transform JSON literal (boolean false)");
	println!("Tests transforming property with @type @json to a JSON literal (boolean false).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js03-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js03-out.nq");
	println!("Transform JSON literal (double)");
	println!("Tests transforming property with @type @json to a JSON literal (double).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js04-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js04-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js04-out.nq");
	println!("Transform JSON literal (double-zero)");
	println!("Tests transforming property with @type @json to a JSON literal (double-zero).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js05-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js05-out.nq");
	println!("Transform JSON literal (integer)");
	println!("Tests transforming property with @type @json to a JSON literal (integer).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js06-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js06-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js06-out.nq");
	println!("Transform JSON literal (object)");
	println!("Tests transforming property with @type @json to a JSON literal (object).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js07-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js07-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js07-out.nq");
	println!("Transform JSON literal (array)");
	println!("Tests transforming property with @type @json to a JSON literal (array).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js08-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js08-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js08-out.nq");
	println!("Transform JSON literal with array canonicalization");
	println!("Tests transforming JSON literal with array canonicalization.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js09() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js09-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js09-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js09-out.nq");
	println!("Transform JSON literal with string canonicalization");
	println!("Tests transforming JSON literal with string canonicalization.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js10() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js10-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js10-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js10-out.nq");
	println!("Transform JSON literal with structural canonicalization");
	println!("Tests transforming JSON literal with structural canonicalization.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js11() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js11-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js11-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js11-out.nq");
	println!("Transform JSON literal with unicode canonicalization");
	println!("Tests transforming JSON literal with unicode canonicalization.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js12() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js12-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js12-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js12-out.nq");
	println!("Transform JSON literal with value canonicalization");
	println!("Tests transforming JSON literal with value canonicalization.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js13() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js13-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js13-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js13-out.nq");
	println!("Transform JSON literal with wierd canonicalization");
	println!("Tests transforming JSON literal with wierd canonicalization.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js14() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js14-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js14-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js14-out.nq");
	println!("Transform JSON literal without expanding contents");
	println!("Tests transforming JSON literal does not expand terms inside json.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js15() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js15-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js15-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js15-out.nq");
	println!("Transform JSON literal aleady in expanded form");
	println!("Tests transforming JSON literal in expanded form.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js16() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js16-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js16-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js16-out.nq");
	println!("Transform JSON literal aleady in expanded form with aliased keys");
	println!("Tests transforming JSON literal in expanded form with aliased keys in value object.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js17() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js17-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js17-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js17-out.nq");
	println!("Transform JSON literal (string)");
	println!("Tests transforming property with @type @json to a JSON literal (string).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js18() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js18-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js18-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js18-out.nq");
	println!("Transform JSON literal (null)");
	println!("Tests transforming property with @type @json to a JSON literal (null).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js19() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js19-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js19-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js19-out.nq");
	println!("Transform JSON literal with aliased @type");
	println!("Tests transforming JSON literal with aliased @type.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js20() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js20-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js20-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js20-out.nq");
	println!("Transform JSON literal with aliased @value");
	println!("Tests transforming JSON literal with aliased @value.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js21() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js21-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js21-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js21-out.nq");
	println!("Transform JSON literal with @context");
	println!("Tests transforming JSON literal with a @context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js22() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js22-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js22-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js22-out.nq");
	println!("Transform JSON literal (null) aleady in expanded form.");
	println!("Tests transforming property with @type @json to a JSON literal (null).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_js23() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js23-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js23-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/js23-out.nq");
	println!("Transform JSON literal (empty array).");
	println!("Tests transforming property with @type @json to a JSON literal (empty array).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_li01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li01-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li01-out.nq");
	println!("@list containing @list");
	println!("List of lists.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_li02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li02-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li02-out.nq");
	println!("@list containing empty @list");
	println!("List of lists.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_li03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li03-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li03-out.nq");
	println!("@list containing @list (with coercion)");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_li04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li04-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li04-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li04-out.nq");
	println!("@list containing empty @list (with coercion)");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_li05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li05-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li05-out.nq");
	println!("coerced @list containing an array");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_li06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li06-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li06-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li06-out.nq");
	println!("coerced @list containing an empty array");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_li07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li07-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li07-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li07-out.nq");
	println!("coerced @list containing deep arrays");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_li08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li08-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li08-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li08-out.nq");
	println!("coerced @list containing deep empty arrays");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_li09() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li09-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li09-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li09-out.nq");
	println!("coerced @list containing multiple lists");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_li10() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li10-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li10-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li10-out.nq");
	println!("coerced @list containing mixed list values");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_li11() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li11-in.jsonld");
	let base_url = iri!("http://example.com/");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li11-out.nq");
	println!("List with good @base.");
	println!("Tests list elements expanded to IRIs with a good @base.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_li13() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li13-in.jsonld");
	let base_url = iri!("http://example.com/");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li13-out.nq");
	println!("List with empty @base.");
	println!("Tests list elements expanded to IRIs with an empty @base.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_li14() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li14-in.jsonld");
	let base_url = iri!("http://example.com/");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/li14-out.nq");
	println!("List with null @base.");
	println!("Tests list elements expanded to IRIs with a null @base.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m001-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m001-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m001-out.nq");
	println!("Adds @id to object not having an @id");
	println!("Expansion using @container: @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m002-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m002-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m002-out.nq");
	println!("Retains @id in object already having an @id");
	println!("Expansion using @container: @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m003() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m003-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m003-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m003-out.nq");
	println!("Adds @type to object not having an @type");
	println!("Expansion using @container: @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m004() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m004-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m004-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m004-out.nq");
	println!("Prepends @type in object already having an @type");
	println!("Expansion using @container: @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m005() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m005-in.jsonld");
	let base_url = iri!("http://example.org/");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m005-out.nq");
	println!("Adds expanded @id to object");
	println!("Expansion using @container: @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m006() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m006-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m006-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m006-out.nq");
	println!("Adds vocabulary expanded @type to object");
	println!("Expansion using @container: @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m007() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m007-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m007-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m007-out.nq");
	println!("Adds document expanded @type to object");
	println!("Expansion using @container: @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m008() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m008-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m008-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m008-out.nq");
	println!("When type is in a type map");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m009() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m009-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m009-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m009-out.nq");
	println!("language map with @none");
	println!("index on @language");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m010() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m010-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m010-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m010-out.nq");
	println!("language map with alias of @none");
	println!("index on @language");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m011() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m011-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m011-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m011-out.nq");
	println!("id map with @none");
	println!("index on @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m012() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m012-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m012-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m012-out.nq");
	println!("type map with alias of @none");
	println!("index on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m013() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m013-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m013-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m013-out.nq");
	println!("graph index map with @none");
	println!("index on @graph and @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m014() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m014-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m014-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m014-out.nq");
	println!("graph index map with alias @none");
	println!("index on @graph and @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m015() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m015-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m015-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m015-out.nq");
	println!("graph id index map with aliased @none");
	println!("index on @graph and @id with @none");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m016() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m016-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m016-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m016-out.nq");
	println!("graph id index map with aliased @none");
	println!("index on @graph and @id with @none");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m017() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m017-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m017-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m017-out.nq");
	println!("string value of type map expands to node reference");
	println!("index on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m018() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m018-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m018-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m018-out.nq");
	println!("string value of type map expands to node reference with @type: @id");
	println!("index on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m019() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m019-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m019-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m019-out.nq");
	println!("string value of type map expands to node reference with @type: @vocab");
	println!("index on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_m020() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m020-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/m020-in.jsonld");
	println!("string value of type map must not be a literal");
	println!("index on @type");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTypeMapping
	).await
}

#[async_std::test]
async fn to_rdf_n001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n001-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n001-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n001-out.nq");
	println!("Expands input using @nest");
	println!("Expansion using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_n002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n002-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n002-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n002-out.nq");
	println!("Expands input using aliased @nest");
	println!("Expansion using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_n003() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n003-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n003-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n003-out.nq");
	println!("Appends nested values when property at base and nested");
	println!("Expansion using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_n004() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n004-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n004-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n004-out.nq");
	println!("Appends nested values from all @nest aliases in term order");
	println!("Expansion using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_n005() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n005-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n005-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n005-out.nq");
	println!("Nested nested containers");
	println!("Expansion using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_n006() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n006-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n006-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n006-out.nq");
	println!("Arrays of nested values");
	println!("Expansion using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_n007() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n007-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n007-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n007-out.nq");
	println!("A nest of arrays");
	println!("Expansion using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_n008() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n008-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n008-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/n008-out.nq");
	println!("Multiple keys may mapping to @type when nesting");
	println!("Expansion using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_nt01() {
	// TODO
}

#[async_std::test]
async fn to_rdf_nt02() {
	// TODO
}

#[async_std::test]
async fn to_rdf_nt03() {
	// TODO
}

#[async_std::test]
async fn to_rdf_nt04() {
	// TODO
}

#[async_std::test]
async fn to_rdf_nt05() {
	// TODO
}

#[async_std::test]
async fn to_rdf_nt06() {
	// TODO
}

#[async_std::test]
async fn to_rdf_nt07() {
	// TODO
}

#[async_std::test]
async fn to_rdf_nt08() {
	// TODO
}

#[async_std::test]
async fn to_rdf_nt09() {
	// TODO
}

#[async_std::test]
async fn to_rdf_nt10() {
	// TODO
}

#[async_std::test]
async fn to_rdf_nt11() {
	// TODO
}

#[async_std::test]
async fn to_rdf_nt12() {
	// TODO
}

#[async_std::test]
async fn to_rdf_nt13() {
	// TODO
}

#[async_std::test]
async fn to_rdf_nt14() {
	// TODO
}

#[async_std::test]
async fn to_rdf_nt15() {
	// TODO
}

#[async_std::test]
async fn to_rdf_nt16() {
	// TODO
}

#[async_std::test]
async fn to_rdf_p001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/p001-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/p001-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/p001-out.nq");
	println!("@version may be specified after first context");
	println!("If processing mode is not set through API, it is set by the first context containing @version.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_p002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/p002-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/p002-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/p002-out.nq");
	println!("@version setting [1.0, 1.1, 1.0]");
	println!("If processing mode is not set through API, it is set by the first context containing @version.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_p003() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/p003-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/p003-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/p003-out.nq");
	println!("@version setting [1.1, 1.0]");
	println!("If processing mode is not set through API, it is set by the first context containing @version.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_p004() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/p004-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/p004-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/p004-out.nq");
	println!("@version setting [1.1, 1.0, 1.1]");
	println!("If processing mode is not set through API, it is set by the first context containing @version.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pi01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi01-in.jsonld");
	println!("error if @version is json-ld-1.0 for property-valued index");
	println!("Expanding index maps where index is a property.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTermDefinition
	).await
}

#[async_std::test]
async fn to_rdf_pi02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi02-in.jsonld");
	println!("error if @container does not include @index for property-valued index");
	println!("Expanding index maps where index is a property.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTermDefinition
	).await
}

#[async_std::test]
async fn to_rdf_pi03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi03-in.jsonld");
	println!("error if @index is a keyword for property-valued index");
	println!("Expanding index maps where index is a property.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTermDefinition
	).await
}

#[async_std::test]
async fn to_rdf_pi04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi04-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi04-in.jsonld");
	println!("error if @index is not a string for property-valued index");
	println!("Expanding index maps where index is a property.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTermDefinition
	).await
}

#[async_std::test]
async fn to_rdf_pi05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi05-in.jsonld");
	println!("error if attempting to add property to value object for property-valued index");
	println!("Expanding index maps where index is a property.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidValueObject
	).await
}

#[async_std::test]
async fn to_rdf_pi06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi06-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi06-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi06-out.nq");
	println!("property-valued index expands to property value, instead of @index (value)");
	println!("Expanding index maps where index is a property.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pi07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi07-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi07-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi07-out.nq");
	println!("property-valued index appends to property value, instead of @index (value)");
	println!("Expanding index maps where index is a property.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pi08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi08-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi08-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi08-out.nq");
	println!("property-valued index expands to property value, instead of @index (node)");
	println!("Expanding index maps where index is a property.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pi09() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi09-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi09-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi09-out.nq");
	println!("property-valued index appends to property value, instead of @index (node)");
	println!("Expanding index maps where index is a property.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pi10() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi10-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi10-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi10-out.nq");
	println!("property-valued index does not output property for @none");
	println!("Expanding index maps where index is a property.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pi11() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi11-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi11-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pi11-out.nq");
	println!("property-valued index adds property to graph object");
	println!("Expanding index maps where index is a property.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr01-in.jsonld");
	println!("Protect a term");
	println!("Check error when overriding a protected term.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::ProtectedTermRedefinition
	).await
}

#[async_std::test]
async fn to_rdf_pr02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr02-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr02-out.nq");
	println!("Set a term to not be protected");
	println!("A term with @protected: false is not protected.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr03-in.jsonld");
	println!("Protect all terms in context");
	println!("A protected context protects all term definitions.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::ProtectedTermRedefinition
	).await
}

#[async_std::test]
async fn to_rdf_pr04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr04-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr04-in.jsonld");
	println!("Do not protect term with @protected: false");
	println!("A protected context does not protect terms with @protected: false.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::ProtectedTermRedefinition
	).await
}

#[async_std::test]
async fn to_rdf_pr05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr05-in.jsonld");
	println!("Clear active context with protected terms from an embedded context");
	println!("The Active context be set to null from an embedded context.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidContextNullification
	).await
}

#[async_std::test]
async fn to_rdf_pr06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr06-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr06-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr06-out.nq");
	println!("Clear active context of protected terms from a term.");
	println!("The Active context may be set to null from a scoped context of a term.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr08-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr08-in.jsonld");
	println!("Term with protected scoped context.");
	println!("A scoped context can protect terms.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::ProtectedTermRedefinition
	).await
}

#[async_std::test]
async fn to_rdf_pr09() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr09-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr09-in.jsonld");
	println!("Attempt to redefine term in other protected context.");
	println!("A protected term cannot redefine another protected term.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::ProtectedTermRedefinition
	).await
}

#[async_std::test]
async fn to_rdf_pr10() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr10-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr10-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr10-out.nq");
	println!("Simple protected and unprotected terms.");
	println!("Simple protected and unprotected terms.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr11() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr11-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr11-in.jsonld");
	println!("Fail to override protected term.");
	println!("Fail to override protected term.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::ProtectedTermRedefinition
	).await
}

#[async_std::test]
async fn to_rdf_pr12() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr12-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr12-in.jsonld");
	println!("Scoped context fail to override protected term.");
	println!("Scoped context fail to override protected term.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::ProtectedTermRedefinition
	).await
}

#[async_std::test]
async fn to_rdf_pr13() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr13-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr13-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr13-out.nq");
	println!("Override unprotected term.");
	println!("Override unprotected term.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr14() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr14-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr14-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr14-out.nq");
	println!("Clear protection with null context.");
	println!("Clear protection with null context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr15() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr15-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr15-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr15-out.nq");
	println!("Clear protection with array with null context");
	println!("Clear protection with array with null context");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr16() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr16-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr16-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr16-out.nq");
	println!("Override protected terms after null.");
	println!("Override protected terms after null.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr17() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr17-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr17-in.jsonld");
	println!("Fail to override protected terms with type.");
	println!("Fail to override protected terms with type.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidContextNullification
	).await
}

#[async_std::test]
async fn to_rdf_pr18() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr18-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr18-in.jsonld");
	println!("Fail to override protected terms with type+null+ctx.");
	println!("Fail to override protected terms with type+null+ctx.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidContextNullification
	).await
}

#[async_std::test]
async fn to_rdf_pr19() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr19-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr19-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr19-out.nq");
	println!("Mix of protected and unprotected terms.");
	println!("Mix of protected and unprotected terms.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr20() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr20-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr20-in.jsonld");
	println!("Fail with mix of protected and unprotected terms with type+null+ctx.");
	println!("Fail with mix of protected and unprotected terms with type+null+ctx.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidContextNullification
	).await
}

#[async_std::test]
async fn to_rdf_pr21() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr21-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr21-in.jsonld");
	println!("Fail with mix of protected and unprotected terms with type+null.");
	println!("Fail with mix of protected and unprotected terms with type+null.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidContextNullification
	).await
}

#[async_std::test]
async fn to_rdf_pr22() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr22-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr22-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr22-out.nq");
	println!("Check legal overriding of type-scoped protected term from nested node.");
	println!("Check legal overriding of type-scoped protected term from nested node.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr23() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr23-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr23-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr23-out.nq");
	println!("Allows redefinition of protected alias term with same definition.");
	println!("Allows redefinition of protected alias term with same definition.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr24() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr24-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr24-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr24-out.nq");
	println!("Allows redefinition of protected prefix term with same definition.");
	println!("Allows redefinition of protected prefix term with same definition.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr25() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr25-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr25-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr25-out.nq");
	println!("Allows redefinition of terms with scoped contexts using same definitions.");
	println!("Allows redefinition of terms with scoped contexts using same definitions.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr26() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr26-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr26-in.jsonld");
	println!("Fails on redefinition of terms with scoped contexts using different definitions.");
	println!("Fails on redefinition of terms with scoped contexts using different definitions.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::ProtectedTermRedefinition
	).await
}

#[async_std::test]
async fn to_rdf_pr27() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr27-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr27-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr27-out.nq");
	println!("Allows redefinition of protected alias term with same definition modulo protected flag.");
	println!("Allows redefinition of protected alias term with same definition modulo protected flag.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr28() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr28-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr28-in.jsonld");
	println!("Fails if trying to redefine a protected null term.");
	println!("A protected term with a null IRI mapping cannot be redefined.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::ProtectedTermRedefinition
	).await
}

#[async_std::test]
async fn to_rdf_pr29() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr29-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr29-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr29-out.nq");
	println!("Does not expand a Compact IRI using a non-prefix term.");
	println!("Expansion of Compact IRIs considers if the term can be used as a prefix.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr30() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr30-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr30-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr30-out.nq");
	println!("Keywords may be protected.");
	println!("Keywords may not be redefined other than to protect them.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr31() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr31-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr31-in.jsonld");
	println!("Protected keyword aliases cannot be overridden.");
	println!("Keywords may not be redefined other than to protect them.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::ProtectedTermRedefinition
	).await
}

#[async_std::test]
async fn to_rdf_pr32() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr32-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr32-in.jsonld");
	println!("Protected @type cannot be overridden.");
	println!("Keywords may not be redefined other than to protect them.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::ProtectedTermRedefinition
	).await
}

#[async_std::test]
async fn to_rdf_pr33() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr33-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr33-in.jsonld");
	println!("Fails if trying to declare a keyword alias as prefix.");
	println!("Keyword aliases can not be used as prefixes.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTermDefinition
	).await
}

#[async_std::test]
async fn to_rdf_pr34() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr34-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr34-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr34-out.nq");
	println!("Ignores a non-keyword term starting with '@'");
	println!("Terms in the form of a keyword, which are not keywords, are ignored.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr35() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr35-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr35-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr35-out.nq");
	println!("Ignores a non-keyword term starting with '@' (with @vocab)");
	println!("Terms in the form of a keyword, which are not keywords, are ignored.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr36() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr36-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr36-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr36-out.nq");
	println!("Ignores a term mapping to a value in the form of a keyword.");
	println!("Terms in the form of a keyword, which are not keywords, are ignored.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr37() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr37-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr37-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr37-out.nq");
	println!("Ignores a term mapping to a value in the form of a keyword (with @vocab).");
	println!("Terms in the form of a keyword, which are not keywords, are ignored.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr38() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr38-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr38-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr38-out.nq");
	println!("Ignores a term mapping to a value in the form of a keyword (@reverse).");
	println!("Terms in the form of a keyword, which are not keywords, are ignored.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr39() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr39-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr39-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr39-out.nq");
	println!("Ignores a term mapping to a value in the form of a keyword (@reverse with @vocab).");
	println!("Terms in the form of a keyword, which are not keywords, are ignored.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_pr40() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr40-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr40-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/pr40-out.nq");
	println!("Protected terms and property-scoped contexts");
	println!("Check overriding of protected term from property-scoped context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_rt01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/rt01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/rt01-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/rt01-out.nq");
	println!("Representing numbers >= 1e21");
	println!("numbers with no fractions but that are >= 1e21 are represented as xsd:double");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_so01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so01-in.jsonld");
	println!("@import is invalid in 1.0.");
	println!("@import is invalid in 1.0.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidContextEntry
	).await
}

#[async_std::test]
async fn to_rdf_so02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so02-in.jsonld");
	println!("@import must be a string");
	println!("@import must be a string.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidImportValue
	).await
}

#[async_std::test]
async fn to_rdf_so03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so03-in.jsonld");
	println!("@import overflow");
	println!("Processors must detect source contexts that include @import.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidContextEntry
	).await
}

#[async_std::test]
async fn to_rdf_so05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so05-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so05-out.nq");
	println!("@propagate: true on type-scoped context with @import");
	println!("type-scoped context with @propagate: true survive node-objects (with @import)");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_so06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so06-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so06-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so06-out.nq");
	println!("@propagate: false on property-scoped context with @import");
	println!("property-scoped context with @propagate: false do not survive node-objects (with @import)");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_so07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so07-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so07-in.jsonld");
	println!("Protect all terms in sourced context");
	println!("A protected context protects all term definitions.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::ProtectedTermRedefinition
	).await
}

#[async_std::test]
async fn to_rdf_so08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so08-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so08-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so08-out.nq");
	println!("Override term defined in sourced context");
	println!("The containing context is merged into the source context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_so09() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so09-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so09-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so09-out.nq");
	println!("Override @vocab defined in sourced context");
	println!("The containing context is merged into the source context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_so10() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so10-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so10-in.jsonld");
	println!("Protect terms in sourced context");
	println!("The containing context is merged into the source context.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::ProtectedTermRedefinition
	).await
}

#[async_std::test]
async fn to_rdf_so11() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so11-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so11-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so11-out.nq");
	println!("Override protected terms in sourced context");
	println!("The containing context is merged into the source context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_so12() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so12-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so12-in.jsonld");
	println!("@import may not be used in an imported context.");
	println!("@import only valid within a term definition.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidContextEntry
	).await
}

#[async_std::test]
async fn to_rdf_so13() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so13-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/so13-in.jsonld");
	println!("@import can only reference a single context");
	println!("@import can only reference a single context.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidRemoteContext
	).await
}

#[async_std::test]
async fn to_rdf_tn01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/tn01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/tn01-in.jsonld");
	println!("@type: @none is illegal in 1.0.");
	println!("@type: @none is illegal in json-ld-1.0.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		ErrorCode::InvalidTypeMapping
	).await
}

#[async_std::test]
async fn to_rdf_tn02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/tn02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/tn02-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/tn02-out.nq");
	println!("@type: @none expands strings as value objects");
	println!("@type: @none leaves inputs other than strings alone");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_wf01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf01-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf01-out.nq");
	println!("Triples including invalid subject IRIs are rejected");
	println!("ToRdf emits only well-formed statements.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_wf02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf02-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf02-out.nq");
	println!("Triples including invalid predicate IRIs are rejected");
	println!("ToRdf emits only well-formed statements.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_wf03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf03-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf03-out.nq");
	println!("Triples including invalid object IRIs are rejected");
	println!("ToRdf emits only well-formed statements.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_wf04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf04-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf04-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf04-out.nq");
	println!("Triples including invalid type IRIs are rejected");
	println!("ToRdf emits only well-formed statements.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_wf05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf05-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf05-out.nq");
	println!("Triples including invalid language tags are rejected");
	println!("ToRdf emits only well-formed statements.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

#[async_std::test]
async fn to_rdf_wf07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf07-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf07-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/toRdf/wf07-out.nq");
	println!("Triples including invalid graph name IRIs are rejected");
	println!("ToRdf emits only well-formed statements.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
			rdf_direction: None,
			produce_generalized_rdf: false
		},
		input_url,
		base_url,
		output_url
	).await
}

