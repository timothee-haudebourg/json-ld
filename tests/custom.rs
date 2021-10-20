#![feature(proc_macro_hygiene)]

extern crate async_std;
extern crate iref;
#[macro_use]
extern crate static_iref;
extern crate json_ld;

use async_std::task;
use iref::{Iri, IriBuf};
use json_ld::{
	context::{self, Loader as ContextLoader, Local, ProcessingOptions},
	expansion,
	util::{json_ld_eq, AsJson},
	Document, FsLoader, Loader, ProcessingMode,
};
use ijson::IValue;

#[derive(Clone, Copy)]
struct Options<'a> {
	processing_mode: ProcessingMode,
	context: Option<Iri<'a>>,
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

fn positive_test(options: Options, input_url: Iri, base_url: Iri, output_url: Iri) {
	let mut loader = FsLoader::<IValue>::new(|s| serde_json::from_str(s));
	loader.mount(iri!("file://crate/tests"), "tests");

	let input = task::block_on(loader.load(input_url)).unwrap();
	let output = task::block_on(loader.load(output_url)).unwrap();
	let mut input_context: context::Json<IValue, IriBuf> = context::Json::new(Some(base_url));

	if let Some(context_url) = options.context {
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
	}

	let result = task::block_on(input.expand_with(
		Some(base_url),
		&input_context,
		&mut loader,
		options.into(),
	))
	.unwrap();

	let result_json: IValue = result.as_json();
	let success = json_ld_eq(&result_json, &*output);

	if success {
		println!("output=\n{}", serde_json::to_string_pretty(&result_json).unwrap());
	} else {
		println!("output=\n{}", serde_json::to_string_pretty(&result_json).unwrap());
		println!("\nexpected=\n{}", serde_json::to_string_pretty(&*output).unwrap());
	}

	assert!(success)
}

// See See w3c/json-ld-api#533
// #[test]
// fn custom_li12() {
// 	let input_url = iri!("file://crate/tests/custom/li12-in.jsonld");
// 	let base_url = iri!("file://crate/tests/custom/li12-in.jsonld");
// 	let output_url = iri!("file://crate/tests/custom/li12-out.jsonld");
// 	positive_test(
// 		Options {
// 			processing_mode: ProcessingMode::JsonLd1_1,
// 			context: None
// 		},
// 		input_url,
// 		base_url,
// 		output_url
// 	)
// }

#[test]
fn custom_e111() {
	let input_url = iri!("file://crate/tests/custom/e111-in.jsonld");
	let base_url = iri!("file://crate/tests/custom/e111-in.jsonld");
	let output_url = iri!("file://crate/tests/custom/e111-out.jsonld");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn custom_e112() {
	let input_url = iri!("file://crate/tests/custom/e112-in.jsonld");
	let base_url = iri!("file://crate/tests/custom/e112-in.jsonld");
	let output_url = iri!("file://crate/tests/custom/e112-out.jsonld");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

// See w3c/json-ld-api#480
// #[test]
// fn custom_e122() {
// 	let input_url = iri!("file://crate/tests/custom/e122-in.jsonld");
// 	let base_url = iri!("file://crate/tests/custom/e122-in.jsonld");
// 	let output_url = iri!("file://crate/tests/custom/e122-out.jsonld");
// 	positive_test(
// 		Options {
// 			processing_mode: ProcessingMode::JsonLd1_1,
// 			context: None
// 		},
// 		input_url,
// 		base_url,
// 		output_url
// 	)
// }

#[test]
fn custom_c037() {
	let input_url = iri!("file://crate/tests/custom/c037-in.jsonld");
	let base_url = iri!("file://crate/tests/custom/c037-in.jsonld");
	let output_url = iri!("file://crate/tests/custom/c037-out.jsonld");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn custom_c038() {
	let input_url = iri!("file://crate/tests/custom/c038-in.jsonld");
	let base_url = iri!("file://crate/tests/custom/c038-in.jsonld");
	let output_url = iri!("file://crate/tests/custom/c038-out.jsonld");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn custom_0120() {
	let input_url = iri!("file://crate/tests/custom/0120-in.jsonld");
	let base_url = iri!("file://crate/tests/custom/0120-in.jsonld");
	let output_url = iri!("file://crate/tests/custom/0120-out.jsonld");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn custom_0122() {
	let input_url = iri!("file://crate/tests/custom/0122-in.jsonld");
	let base_url = iri!("file://crate/tests/custom/0122-in.jsonld");
	let output_url = iri!("file://crate/tests/custom/0122-out.jsonld");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn custom_0123() {
	let input_url = iri!("file://crate/tests/custom/0123-in.jsonld");
	let base_url = iri!("file://crate/tests/custom/0123-in.jsonld");
	let output_url = iri!("file://crate/tests/custom/0123-out.jsonld");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn custom_0124() {
	let input_url = iri!("file://crate/tests/custom/0124-in.jsonld");
	let base_url = iri!("file://crate/tests/custom/0124-in.jsonld");
	let output_url = iri!("file://crate/tests/custom/0124-out.jsonld");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn custom_0125() {
	let input_url = iri!("file://crate/tests/custom/0125-in.jsonld");
	let base_url = iri!("file://crate/tests/custom/0125-in.jsonld");
	let output_url = iri!("file://crate/tests/custom/0125-out.jsonld");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}
