use iref::{{Iri, IriBuf}};
use json_ld::{{
	context::{{self, ContextLoader, Local, ProcessingOptions}},
	expansion, rdf,
	rdf::Display,
	util::AsJson,
	Document, ErrorCode, FsLoader, Loader, ProcessingMode,
}};
use locspan::Loc;
use serde_json::Value;
use static_iref::iri;
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
struct Options<'a> {{
	processing_mode: ProcessingMode,
	context: Option<Iri<'a>>,
	rdf_direction: Option<rdf::RdfDirection>,
	produce_generalized_rdf: bool,
}}

impl<'a> From<Options<'a>> for expansion::Options {{
	fn from(options: Options<'a>) -> expansion::Options {{
		expansion::Options {{
			processing_mode: options.processing_mode,
			ordered: false,
			..expansion::Options::default()
		}}
	}}
}}

impl<'a> From<Options<'a>> for ProcessingOptions {{
	fn from(options: Options<'a>) -> ProcessingOptions {{
		ProcessingOptions {{
			processing_mode: options.processing_mode,
			..ProcessingOptions::default()
		}}
	}}
}}

fn infallible<T>(t: T) -> Result<T, std::convert::Infallible> {{
	Ok(t)
}}

async fn positive_test(
	options: Options<'_>,
	input_url: Iri<'_>,
	base_url: Iri<'_>,
	output_url: Iri<'_>,
) {{
	let mut loader = FsLoader::<Value>::new(|s| serde_json::from_str(s));
	loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");

	let input = loader.load(input_url).await.unwrap();
	let mut input_context: context::Json<Value, IriBuf> = context::Json::new(Some(base_url));

	if let Some(context_url) = options.context {{
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
	}}

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
	{{
		if options.produce_generalized_rdf || property.as_iri().is_some() {{
			match graph {{
				Some(graph) => lines.insert(format!(
					"{{}} {{}} {{}} {{}} .\n",
					subject.rdf_display(),
					property,
					object,
					graph.rdf_display()
				)),
				None => lines.insert(format!(
					"{{}} {{}} {{}} .\n",
					subject.rdf_display(),
					property,
					object
				)),
			}};
		}}
	}}

	let output = itertools::join(lines, "");

	let mut success = output == expected_output;

	if !success {{
		let parsed_output = parse_nquads(&output);
		let parsed_expected_output = parse_nquads(&expected_output);
		success = parsed_output.is_isomorphic_to(&parsed_expected_output);
		if !success {{
			let json: Value = doc.as_json();
			eprintln!(
				"expanded:\n{{}}",
				serde_json::to_string_pretty(&json).unwrap()
			);
			eprintln!("expected:\n{{}}", expected_output);
			eprintln!("found:\n{{}}", output);
		}}
	}}

	assert!(success)
}}

fn parse_nquads(buffer: &str) -> grdf::BTreeDataset {{
	eprintln!("parse:\n{{}}", buffer);

	use locspan::Strip;
	use nquads_syntax::Parse;
	let mut lexer = nquads_syntax::Lexer::new(
		(),
		nquads_syntax::lexing::Utf8Decoded::new(buffer.chars().map(infallible)).peekable(),
	);

	match nquads_syntax::GrdfDocument::parse(&mut lexer) {{
		Ok(Loc(nquads, _)) => nquads.into_iter().map(Strip::strip).collect(),
		Err(Loc(e, _)) => {{
			panic!("parse error: {{:?}}", e)
		}}
	}}
}}

async fn negative_test(
	options: Options<'_>,
	input_url: Iri<'_>,
	base_url: Iri<'_>,
	error_code: ErrorCode,
) {{
	let mut loader = FsLoader::<Value>::new(|s| serde_json::from_str(s));
	loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");

	let input = loader.load(input_url).await.unwrap();
	let mut input_context: context::Json<Value, IriBuf> = context::Json::new(Some(base_url));

	if let Some(context_url) = options.context {{
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
	}}

	let result = input
		.expand_with(Some(base_url), &input_context, &mut loader, options.into())
		.await;

	match result {{
		Ok(output) => {{
			let output_json: Value = output.as_json();
			println!(
				"output=\n{{}}",
				serde_json::to_string_pretty(&output_json).unwrap()
			);
			panic!(
				"expansion succeeded where it should have failed with code: {{}}",
				error_code
			)
		}}
		Err(e) => {{
			assert_eq!(e.code(), error_code)
		}}
	}}
}}
