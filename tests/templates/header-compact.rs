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

	panic!("TODO positive compact test")
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

	panic!("TODO negative compact test")
}}
