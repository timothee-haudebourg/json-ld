#[test]
fn {}() {{
	let input_url = iri!("{}");
	let base_url = iri!("{}");
	println!("{}");{}
	negative_test(
		Options {{
			processing_mode: ProcessingMode::{:?},
			compact_arrays: {:?},
			context: {}
		}},
		input_url,
		base_url,
		ErrorCode::{:?}
	)
}}
