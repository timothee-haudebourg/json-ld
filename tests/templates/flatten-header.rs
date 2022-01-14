extern crate async_std;
extern crate iref;
#[macro_use]
extern crate static_iref;
extern crate json_ld;

use async_std::task;
use ijson::IValue;
use iref::{{Iri, IriBuf}};
use json_ld::{{
	compaction,
	context::{{self, Loader as ContextLoader, Local, ProcessingOptions}},
	expansion,
	util::{{json_ld_eq, AsJson}},
	Document, ErrorCode, FsLoader, Loader, ProcessingMode,
}};

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

fn positive_test(options: Options, input_url: Iri, base_url: Iri, output_url: Iri) {{
	let mut loader = FsLoader::<IValue>::new(|s| serde_json::from_str(s));
	loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");

	let input = task::block_on(loader.load(input_url)).unwrap();
	let expected_output = task::block_on(loader.load(output_url)).unwrap();
	let mut input_context: context::Json<IValue, IriBuf> = context::Json::new(Some(base_url));

	if let Some(context_url) = options.context {{
		let local_context = task::block_on(loader.load_context(context_url))
			.unwrap()
			.into_context();
		input_context = task::block_on(local_context.process_with(
			&input_context,
			&mut loader,
			Some(base_url),
			options.into(),
		))
		.unwrap()
		.into_inner();
	}}

	let mut id_generator = json_ld::id::generator::Blank::new_with_prefix("b".to_string());
	let output = task::block_on(input.flatten_with(
		&mut id_generator,
		Some(base_url),
		&input_context,
		&mut loader,
		options.into(),
	))
	.unwrap();
	let json_output: IValue = match options.context {{
		Some(_) => {{
			use compaction::Compact;
			task::block_on(output.compact_with(
				context::Inversible::new(&input_context),
				&mut loader,
				options.into(),
				no_metadata,
			))
			.unwrap()
		}}
		None => output.as_json(),
	}};

	let success = json_ld_eq(&json_output, &*expected_output);

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

fn negative_test(options: Options, input_url: Iri, base_url: Iri, error_code: ErrorCode) {{
	let mut loader = FsLoader::<IValue>::new(|s| serde_json::from_str(s));
	loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");

	let input = task::block_on(loader.load(input_url)).unwrap();
	let mut input_context: context::Json<IValue, IriBuf> = context::Json::new(Some(base_url));

	if let Some(context_url) = options.context {{
		let local_context = task::block_on(loader.load_context(context_url))
			.unwrap()
			.into_context();
		input_context = task::block_on(local_context.process_with(
			&input_context,
			&mut loader,
			Some(base_url),
			options.into(),
		))
		.unwrap()
		.into_inner();
	}}

	let mut id_generator = json_ld::id::generator::Blank::new();
	let result = task::block_on(input.flatten_with(
		&mut id_generator,
		Some(base_url),
		&input_context,
		&mut loader,
		options.into(),
	));

	match result {{
		Ok(output) => {{
			let output_json: IValue = output.as_json();
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