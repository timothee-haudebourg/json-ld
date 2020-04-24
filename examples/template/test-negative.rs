#[test]
fn {}() {{
	let input_url = iri!("{}");
	let base_url = iri!("{}");
	println!("{}");{}
	negative_test(
		Options {{
			processing_mode: ProcessingMode::{:?},
			expand_context: {}
		}},
		input_url,
		base_url,
		ErrorCode::{:?}
	)
}}
