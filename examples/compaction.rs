//! This simple example shows how to compact a document using the `Document::compact` method.
use ijson::IValue;
use json_ld::{
	context::{self, Local},
	Document, Error, Loc, NoLoader,
};

#[async_std::main]
async fn main() -> Result<(), Loc<Error, ()>> {
	// Input JSON-LD document to compact.
	let input: IValue = serde_json::from_str(r#"
		[{
			"http://xmlns.com/foaf/0.1/name": ["Manu Sporny"],
			"http://xmlns.com/foaf/0.1/homepage": [{"@id": "https://manu.sporny.org/"}],
			"http://xmlns.com/foaf/0.1/avatar": [{"@id": "https://twitter.com/account/profile_image/manusporny"}]
		}]
	"#).unwrap();

	// Context
	let context: IValue = serde_json::from_str(
		r#"
		{
			"name": "http://xmlns.com/foaf/0.1/name",
			"homepage": {"@id": "http://xmlns.com/foaf/0.1/homepage", "@type": "@id"},
			"avatar": {"@id": "http://xmlns.com/foaf/0.1/avatar", "@type": "@id"}
		}
	"#,
	)
	.unwrap();

	// JSON-LD document loader.
	//
	// We won't be loading any external document here,
	// so we use the `NoLoader` type.
	let mut loader = NoLoader::<IValue>::new();

	let processed_context = context
		.process::<context::Json<IValue>, _>(&mut loader, None)
		.await?;

	// Compaction.
	let output: IValue = input
		.compact(&processed_context.owned().inversible(), &mut loader)
		.await
		.unwrap();

	println!("{}", serde_json::to_string_pretty(&output).unwrap());

	Ok(())
}
