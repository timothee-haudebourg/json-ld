//! This simple example shows how to compact a document using the `Document::compact` method.
use json::JsonValue;
use json_ld::{
	JsonContext,
	NoLoader,
	Document,
	context::{
		Local
	}
};

#[async_std::main]
async fn main() -> Result<(), json_ld::Error> {
	// Input JSON-LD document to compact.
	let input = json::parse(r#"
		[{
			"http://xmlns.com/foaf/0.1/name": ["Manu Sporny"],
			"http://xmlns.com/foaf/0.1/homepage": [{"@id": "https://manu.sporny.org/"}],
			"http://xmlns.com/foaf/0.1/avatar": [{"@id": "https://twitter.com/account/profile_image/manusporny"}]
		}]
	"#).unwrap();

	// Context
	let context = json::parse(r#"
		{
			"name": "http://xmlns.com/foaf/0.1/name",
			"homepage": {"@id": "http://xmlns.com/foaf/0.1/homepage", "@type": "@id"},
			"avatar": {"@id": "http://xmlns.com/foaf/0.1/avatar", "@type": "@id"}
		}
	"#).unwrap();
	let processed_context = context.process::<JsonContext, _>(&mut NoLoader, None).await?;
	
	// Compaction.
	let output = input.compact(&processed_context, &mut NoLoader).await.unwrap();
	println!("{}", output.pretty(2));

	Ok(())
}
