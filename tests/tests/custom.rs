use iref::IriBuf;
use json_ld::{syntax::Parse, JsonLdProcessor, RemoteDocument};
use locspan::Span;
use static_iref::iri;

async fn custom_01() {
	let mut loader: json_ld::FsLoader<IriBuf, Span> = json_ld::FsLoader::new(|_, _, content| {
		json_ld::syntax::Value::parse_str(content, |span| span)
	});

	loader.mount(
		iri!("https://www.w3.org/").to_owned(),
		"tests/custom/extern/www.w3.org/",
	);
	loader.mount(
		iri!("https://w3id.org/").to_owned(),
		"tests/custom/extern/w3id.org/",
	);

	let input = std::fs::read_to_string("tests/custom/t01-in.jsonld").unwrap();
	let json = json_ld::syntax::Value::parse_str(&input, |span| span).unwrap();
	let doc = RemoteDocument::new(None, None, json);

	let mut generator =
		rdf_types::generator::Blank::new_with_prefix("b".to_string()).with_default_metadata();

	eprintln!("available stack: {:?}", stacker::remaining_stack());
	doc.to_rdf(&mut generator, &mut loader).await.unwrap();
}

// This may fail depending on the default stack size.
// #[async_std::test]
// async fn custom_01_default_memory() {
// 	custom_01().await
// }

// This will fail because not enough stack memory.
// #[test]
// fn custom_01_low_memory() {
// 	let child = std::thread::Builder::new()
// 		.stack_size(512 * 1024)
// 		.spawn(|| async_std::task::block_on(custom_01()))
// 		.unwrap();

// 	child.join().unwrap()
// }

#[test]
fn custom_01_high_memory() {
	let child = std::thread::Builder::new()
		.stack_size(2 * 512 * 1024)
		.spawn(|| async_std::task::block_on(custom_01()))
		.unwrap();

	child.join().unwrap()
}
