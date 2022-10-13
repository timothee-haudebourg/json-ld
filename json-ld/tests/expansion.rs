use iref::IriBuf;
use json_ld::{syntax::Parse, Expand, RemoteDocument};
use locspan::{Meta, Span};
use rdf_types::BlankIdBuf;
use static_iref::iri;

#[async_std::test]
async fn expand() {
	let json = json_ld::syntax::Value::parse_str(
		r##"
		{
			"@graph": [
				{
				"http://example.org/vocab#a": {
					"@graph": [
					{
						"http://example.org/vocab#b": "Chapter One"
					}
					]
				}
				}
			]
		}
	"##,
		|span| span,
	)
	.unwrap();

	let document_url: IriBuf =
		iri!("https://w3c.github.io/json-ld-api/tests/0020-in.jsonld").into();
	let mut loader: json_ld::NoLoader<IriBuf, Span, json_ld::syntax::Value<Span>> =
		json_ld::NoLoader::new();
	let _: Meta<json_ld::ExpandedDocument<IriBuf, BlankIdBuf, _>, _> =
		RemoteDocument::new(Some(document_url), json)
			.expand(&mut loader)
			.await
			.unwrap();
}
