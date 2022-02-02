use iref::{{Iri, IriBuf}};
use json_ld::{{
	compaction,
	context::{{self, Loader as ContextLoader, Local, ProcessedOwned, ProcessingOptions}},
	util::json_ld_eq,
	Document, ErrorCode, FsLoader, Loader, ProcessingMode,
}};
use serde_json::Value;
use static_iref::iri;

#[derive(Clone, Copy)]
struct Options<'a> {{
	processing_mode: ProcessingMode,
	compact_arrays: bool,
	context: Option<Iri<'a>>,
}}

impl<'a> From<Options<'a>> for compaction::Options {{
	fn from(options: Options<'a>) -> compaction::Options {{
		compaction::Options {{
			processing_mode: options.processing_mode,
			compact_arrays: options.compact_arrays,
			ordered: false,
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

fn base_json_context(base_url: Iri) -> Value {{
	let mut object = serde_json::Map::new();
	object.insert("@base".to_string(), Value::from(base_url.as_str()));
	object.into()
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
	let compact_context: context::ProcessedOwned<Value, context::Json<Value, IriBuf>> =
		match options.context {{
			Some(context_url) => {{
				let local_context = loader
					.load_context(context_url)
					.await
					.unwrap()
					.into_context();
				local_context
					.process_with(
						&context::Json::new(Some(base_url)),
						&mut loader,
						Some(base_url),
						options.into(),
					)
					.await
					.unwrap()
					.owned()
			}}
			None => {{
				let base_json_context = base_json_context(base_url);
				ProcessedOwned::new(base_json_context, context::Json::new(Some(base_url)))
			}}
		}};

	let output: Value = input
		.compact_with(
			Some(base_url),
			&expand_context,
			&compact_context.inversible(),
			&mut loader,
			options.into(),
			no_metadata,
		)
		.await
		.unwrap();
	let success = json_ld_eq(&output, &*expected_output).await.unwrap();

	if !success {{
		println!(
			"output=\n{{}}",
			serde_json::to_string_pretty(&output).unwrap()
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

	let expand_context: context::Json<Value, IriBuf> = context::Json::new(Some(base_url));
	let compact_context: context::ProcessedOwned<Value, context::Json<Value, IriBuf>> =
		match options.context {{
			Some(context_url) => {{
				let local_context = loader
					.load_context(context_url)
					.await
					.unwrap()
					.into_context();
				let context = local_context
					.process_with(
						&context::Json::new(Some(base_url)),
						&mut loader,
						Some(base_url),
						options.into(),
					)
					.await;
				match context {{
					Ok(context) => context.owned(),
					Err(e) => {{
						assert_eq!(e.code(), error_code);
						return;
					}}
				}}
			}}
			None => {{
				let base_json_context = base_json_context(base_url);
				ProcessedOwned::new(base_json_context, context::Json::new(Some(base_url)))
			}}
		}};

	let result: Result<Value, _> = input
		.compact_with(
			Some(base_url),
			&expand_context,
			&compact_context.inversible(),
			&mut loader,
			options.into(),
			no_metadata,
		)
		.await;

	match result {{
		Ok(output) => {{
			println!(
				"output=\n{{}}",
				serde_json::to_string_pretty(&output).unwrap()
			);
			panic!(
				"compaction succeeded where it should have failed with code: {{}}",
				error_code
			)
		}}
		Err(e) => {{
			assert_eq!(e.code(), error_code)
		}}
	}}
}}
