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
		JsonContext,
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

struct Options<'a> {{
	processing_mode: ProcessingMode,
	context: Option<Iri<'a>>
}}

impl<'a> From<Options<'a>> for expansion::Options {{
	fn from(options: Options<'a>) -> expansion::Options {{
		expansion::Options {{
			processing_mode: options.processing_mode,
			ordered: false
		}}
	}}
}}

fn positive_test(options: Options, input_url: Iri, base_url: Iri, output_url: Iri) {{
	let mut loader = FsLoader::new();
	loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");

	let input = task::block_on(loader.load(input_url)).unwrap();
	let output = task::block_on(loader.load(output_url)).unwrap();
	let mut input_context: JsonContext<IriBuf> = JsonContext::new(Some(base_url));

	if let Some(context_url) = options.context {{
		let local_context = task::block_on(loader.load_context(context_url)).unwrap().into_context();
		input_context = task::block_on(local_context.process(&input_context, &mut loader, Some(base_url))).unwrap();
	}}

	let result = task::block_on(input.expand_with(Some(base_url), &input_context, &mut loader, options.into())).unwrap();

	let result_json = result.as_json();
	let success = json_ld_eq(&result_json, &output);

	if !success {{
		println!("output=\n{{}}", result_json.pretty(2));
		println!("\nexpected=\n{{}}", output.pretty(2));
	}}

	assert!(success)
}}

fn negative_test(options: Options, input_url: Iri, base_url: Iri, error_code: ErrorCode) {{
	let mut loader = FsLoader::new();
	loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");

	let input = task::block_on(loader.load(input_url)).unwrap();
	let mut input_context: JsonContext<IriBuf> = JsonContext::new(Some(base_url));

	if let Some(context_url) = options.context {{
		let local_context = task::block_on(loader.load_context(context_url)).unwrap().into_context();
		input_context = task::block_on(local_context.process(&input_context, &mut loader, Some(base_url))).unwrap();
	}}

	match task::block_on(input.expand_with(Some(base_url), &input_context, &mut loader, options.into())) {{
		Ok(result) => {{
			println!("output=\n{{}}", result.as_json().pretty(2));
			panic!("expansion succeeded where it should have failed with code: {{}}", error_code)
		}},
		Err(e) => {{
			assert_eq!(e.code(), error_code)
		}}
	}}
}}
