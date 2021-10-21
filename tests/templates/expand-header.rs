#![feature(proc_macro_hygiene)]

extern crate async_std;
extern crate iref;
#[macro_use]
extern crate static_iref;
extern crate json_ld;

use async_std::task;
use iref::{{Iri, IriBuf}};
use json_ld::{{
	ErrorCode,
	ProcessingMode,
	Document,
	context::{{
		self,
		ProcessingOptions,
		Local,
		Loader as ContextLoader
	}},
	expansion,
	util::{{
		AsJson,
		json_ld_eq
	}},
	Loader,
	FsLoader
}};
use ijson::IValue;

#[derive(Clone, Copy)]
struct Options<'a> {{
	processing_mode: ProcessingMode,
	context: Option<Iri<'a>>
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

fn positive_test(options: Options, input_url: Iri, base_url: Iri, output_url: Iri) {{
	let mut loader = FsLoader::<IValue>::new(|s| serde_json::from_str(s));
	loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");

	let input = task::block_on(loader.load(input_url)).unwrap();
	let expected_output = task::block_on(loader.load(output_url)).unwrap();
	let mut input_context: context::Json<IValue, IriBuf> = context::Json::new(Some(base_url));

	if let Some(context_url) = options.context {{
		let local_context = task::block_on(loader.load_context(context_url)).unwrap().into_context();
		input_context = task::block_on(local_context.process_with(&input_context, &mut loader, Some(base_url), options.into())).unwrap().into_inner();
	}}

	let output = task::block_on(input.expand_with(Some(base_url), &input_context, &mut loader, options.into())).unwrap();
	let output_json: IValue = output.as_json();

	let success = json_ld_eq(&output_json, &*expected_output);

	if !success {{
		println!(
			"output=\n{{}}",
			serde_json::to_string_pretty(&output_json).unwrap()
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
		let local_context = task::block_on(loader.load_context(context_url)).unwrap().into_context();
		input_context = task::block_on(local_context.process_with(&input_context, &mut loader, Some(base_url), options.into())).unwrap().into_inner();
	}}

	let result = task::block_on(input.expand_with(Some(base_url), &input_context, &mut loader, options.into()));

	match result {{
		Ok(output) => {{
			let output_json: IValue = output.as_json();
			println!("output=\n{{}}", serde_json::to_string_pretty(&output_json).unwrap());
			panic!("expansion succeeded where it should have failed with code: {{}}", error_code)
		}},
		Err(e) => {{
			assert_eq!(e.code(), error_code)
		}}
	}}
}}
