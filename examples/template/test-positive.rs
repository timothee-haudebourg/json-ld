#[test]
fn {}() {{
	let input_url = iri!("{}");
	println!("{}");{}
	positive_test(
		Options {{
			processing_mode: ProcessingMode::{:?},
			expand_context: {},
			ordered: {:?}
		}},
		input_url,
		"{}",
		"{}"
	)
}}
