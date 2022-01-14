#![feature(proc_macro_hygiene)]

extern crate async_std;
extern crate iref;
#[macro_use]
extern crate static_iref;
extern crate json_ld;

use async_std::task;
use iref::{{Iri, IriBuf}};
use json_ld::{{
	compaction,
	context::{{self, Loader as ContextLoader, Local, ProcessedOwned, ProcessingOptions}},
	util::json_ld_eq,
	Document, ErrorCode, FsLoader, Loader, ProcessingMode,
}};
use serde_json::Value;

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

fn positive_test(options: Options, input_url: Iri, base_url: Iri, output_url: Iri) {{
	let mut loader = FsLoader::<Value>::new(|s| serde_json::from_str(s));
	loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");

	let input = task::block_on(loader.load(input_url)).unwrap();
	let expected_output = task::block_on(loader.load(output_url)).unwrap();
	let base_json_context = base_json_context(base_url);
	let mut input_context: ProcessedOwned<Value, context::Json<Value, IriBuf>> =
		ProcessedOwned::new(base_json_context, context::Json::new(Some(base_url)));

	if let Some(context_url) = options.context {{
		let local_context = task::block_on(loader.load_context(context_url))
			.unwrap()
			.into_context();
		input_context = task::block_on(local_context.process_with(
			input_context.as_ref(),
			&mut loader,
			Some(base_url),
			options.into(),
		))
		.unwrap()
		.owned();
	}}

	let output: Value = task::block_on(input.compact_with(
		Some(base_url),
		&input_context,
		&mut loader,
		options.into(),
		no_metadata,
		no_metadata,
	))
	.unwrap();
	let success = json_ld_eq(&output, &*expected_output);

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

fn negative_test(options: Options, input_url: Iri, base_url: Iri, error_code: ErrorCode) {{
	let mut loader = FsLoader::<Value>::new(|s| serde_json::from_str(s));
	loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");

	let input = task::block_on(loader.load(input_url)).unwrap();
	let base_json_context = base_json_context(base_url);
	let mut input_context: ProcessedOwned<Value, context::Json<Value, IriBuf>> =
		ProcessedOwned::new(base_json_context, context::Json::new(Some(base_url)));

	if let Some(context_url) = options.context {{
		let local_context = task::block_on(loader.load_context(context_url))
			.unwrap()
			.into_context();
		input_context = match task::block_on(local_context.process_with(
			input_context.as_ref(),
			&mut loader,
			Some(base_url),
			options.into(),
		)) {{
			Ok(context) => context.owned(),
			Err(e) => {{
				assert_eq!(e.code(), error_code);
				return;
			}}
		}};
	}}

	let result: Result<Value, _> = task::block_on(input.compact_with(
		Some(base_url),
		&input_context,
		&mut loader,
		options.into(),
		no_metadata,
		no_metadata,
	));

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