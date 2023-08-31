use contextual::WithContext;
use json_ld_serialization::serialize;
use json_syntax::Print;
use linked_data::LinkedData;

#[derive(LinkedData)]
#[ld(prefix("ex" = "http://example.org/"))]
struct Foo {
	#[ld("ex:name")]
	name: String,

	#[ld("ex:email")]
	email: String,
}

fn main() {
	let value = Foo {
		name: "John Smith".to_string(),
		email: "john.smith@example.org".to_string(),
	};

	let json = serialize(&value).expect("serialization failed");
	eprintln!("{}", json.with(&()).pretty_print());
}
