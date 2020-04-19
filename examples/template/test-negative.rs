#[test]
fn {}() {{
	let input_url = iri!("{}");
	println!("{}");{}
	negative_test(
		Options {{
			processing_mode: ProcessingMode::{:?},
			expand_context: {}
		}},
		input_url,
		"{}",
		ErrorCode::{:?}
	)
}}
