#[async_std::test]
async fn {}() {{
	let input_url = iri!("{}");
	let base_url = iri!("{}");
	println!("{}");{}
	negative_test(
		Options {{
			processing_mode: ProcessingMode::{:?},
			context: {},
			rdf_direction: {},
			produce_generalized_rdf: {:?}
		}},
		input_url,
		base_url,
		ErrorCode::{:?}
	).await
}}
