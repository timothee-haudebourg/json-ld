use iref::IriBuf;
use json_ld::{syntax::Parse, Expand, RemoteDocument};
use static_iref::iri;

#[async_std::test]
async fn expand() {
	let (json, _) = json_ld::syntax::Value::parse_str(
		r#"
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
	"#,
	)
	.unwrap();

	let document_url: IriBuf =
		iri!("https://w3c.github.io/json-ld-api/tests/0020-in.jsonld").to_owned();
	RemoteDocument::new(Some(document_url), None, json)
		.expand(&mut json_ld::NoLoader)
		.await
		.unwrap();
}
