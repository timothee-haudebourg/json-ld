#![cfg(feature="test")]
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
		ProcessingOptions,
		JsonContext,
		Processed,
		Local,
		Loader as ContextLoader
	}},
	compaction,
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
	compact_arrays: bool,
	context: Option<Iri<'a>>
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

fn positive_test(options: Options, input_url: Iri, base_url: Iri, output_url: Iri) {{
	let mut loader = FsLoader::<IValue>::new(|s| serde_json::from_str(s));
	loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");

	let input = task::block_on(loader.load(input_url)).unwrap();
	let output = task::block_on(loader.load(output_url)).unwrap();
	let base_json_context = json::object! {{
		"@base": json::JsonValue::from(base_url.as_str())
	}};
	let mut input_context: Processed<json::JsonValue, JsonContext<IriBuf>> = Processed::new(base_json_context, JsonContext::new(Some(base_url)));

	if let Some(context_url) = options.context {{
		let local_context = task::block_on(loader.load_context(context_url)).unwrap().into_context();
		input_context = task::block_on(local_context.process_with(input_context.as_ref(), &mut loader, Some(base_url), options.into())).unwrap().owned();
	}}

	let result = task::block_on(input.compact_with(Some(base_url), &input_context, &mut loader, options.into())).unwrap();
	let success = json_ld_eq(&result, &output);

	if !success {{
		println!("output=\n{{}}", result.pretty(2));
		println!("\nexpected=\n{{}}", output.pretty(2));
	}}

	assert!(success)
}}

fn negative_test(options: Options, input_url: Iri, base_url: Iri, error_code: ErrorCode) {{
	let mut loader = FsLoader::new();
	loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");

	let input = task::block_on(loader.load(input_url)).unwrap();
	let base_json_context = json::object! {{
		"@base": json::JsonValue::from(base_url.as_str())
	}};
	let mut input_context: Processed<json::JsonValue, JsonContext<IriBuf>> = Processed::new(base_json_context, JsonContext::new(Some(base_url)));

	if let Some(context_url) = options.context {{
		let local_context = task::block_on(loader.load_context(context_url)).unwrap().into_context();
		input_context = match task::block_on(local_context.process_with(input_context.as_ref(), &mut loader, Some(base_url), options.into())) {{
			Ok(context) => context.owned(),
			Err(e) => {{
				assert_eq!(e.code(), error_code);
				return
			}}
		}};
	}}

	match task::block_on(input.compact_with(Some(base_url), &input_context, &mut loader, options.into())) {{
		Ok(result) => {{
			println!("output=\n{{}}", result.as_json().pretty(2));
			panic!("compaction succeeded where it should have failed with code: {{}}", error_code)
		}},
		Err(e) => {{
			assert_eq!(e.code(), error_code)
		}}
	}}
}}
