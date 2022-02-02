use iref::{{Iri, IriBuf}};
use json_ld::{{
	compaction,
	context::{{self, Loader as ContextLoader, Local, ProcessingOptions}},
	expansion,
	util::{{json_ld_eq, AsJson}},
	Document, ErrorCode, FsLoader, Loader, ProcessingMode,
}};
use serde_json::Value;
use static_iref::iri;

#[derive(Clone, Copy)]
struct Options<'a> {{
	processing_mode: ProcessingMode,
	ordered: bool,
	compact_arrays: bool,
	context: Option<Iri<'a>>,
}}

impl<'a> From<Options<'a>> for expansion::Options {{
	fn from(options: Options<'a>) -> expansion::Options {{
		expansion::Options {{
			processing_mode: options.processing_mode,
			ordered: options.ordered,
			..expansion::Options::default()
		}}
	}}
}}

impl<'a> From<Options<'a>> for compaction::Options {{
	fn from(options: Options<'a>) -> compaction::Options {{
		compaction::Options {{
			processing_mode: options.processing_mode,
			compact_arrays: options.compact_arrays,
			..compaction::Options::default()
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

fn no_metadata<M>(_: Option<&M>) -> () {{
	()
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
	let expected_output = loader.load(output_url).await.unwrap();

	let expand_context: context::Json<Value, IriBuf> = context::Json::new(Some(base_url));
	let compact_context: Option<context::ProcessedOwned<Value, context::Json<Value, IriBuf>>> =
		match options.context {{
			Some(context_url) => {{
				let local_context = loader
					.load_context(context_url)
					.await
					.unwrap()
					.into_context();
				Some(
					local_context
						.process_with(
							&context::Json::new(Some(base_url)),
							&mut loader,
							Some(base_url),
							options.into(),
						)
						.await
						.unwrap()
						.owned(),
				)
			}}
			None => None,
		}};

	let mut id_generator = json_ld::id::generator::Blank::new_with_prefix("b".to_string());
	let output = input
		.flatten_with(
			&mut id_generator,
			Some(base_url),
			&expand_context,
			&mut loader,
			options.into(),
		)
		.await
		.unwrap();

	let json_output: Value = match compact_context {{
		Some(compact_context) => output
			.compact(
				&compact_context.inversible(),
				&mut loader,
				options.into(),
				no_metadata,
			)
			.await
			.unwrap(),
		None => output.as_json(),
	}};

	let success = json_ld_eq(&json_output, &*expected_output).await.unwrap();

	if !success {{
		println!(
			"output=\n{{}}",
			serde_json::to_string_pretty(&json_output).unwrap()
		);
		println!(
			"\nexpected=\n{{}}",
			serde_json::to_string_pretty(&*expected_output).unwrap()
		);
	}}

	assert!(success)
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

	let mut id_generator = json_ld::id::generator::Blank::new();
	let result = input
		.flatten_with(
			&mut id_generator,
			Some(base_url),
			&input_context,
			&mut loader,
			options.into(),
		)
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
