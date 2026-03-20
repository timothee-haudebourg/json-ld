use json_syntax::JsonValue;

pub fn simple_json_ld_eq(a: &JsonValue, b: &JsonValue) -> bool {
	match (a, b) {
		(JsonValue::Array(a), JsonValue::Array(b)) if a.len() == b.len() => {
			let mut selected = Vec::with_capacity(a.len());
			selected.resize(a.len(), false);

			'a_items: for item in a.iter() {
				for (i, sel) in selected.iter_mut().enumerate() {
					if !*sel && simple_json_ld_eq(item, b.get(i).unwrap()) {
						*sel = true;
						continue 'a_items;
					}
				}

				return false;
			}

			true
		}
		(JsonValue::Object(a), JsonValue::Object(b)) if a.len() == b.len() => {
			for (key, value_a) in a.iter() {
				if let Some(value_b) = b.get(key).next() {
					if key == "@list" {
						match (value_a, value_b) {
							(JsonValue::Array(item_a), JsonValue::Array(item_b))
								if item_a.len() == item_b.len() =>
							{
								if !item_a
									.iter()
									.zip(item_b)
									.all(|(a, b)| simple_json_ld_eq(a, b))
								{
									return false;
								}
							}
							_ => {
								if !simple_json_ld_eq(value_a, value_b) {
									return false;
								}
							}
						}
					} else if !simple_json_ld_eq(value_a, value_b) {
						return false;
					}
				} else {
					return false;
				}
			}

			true
		}
		(JsonValue::Null, JsonValue::Null) => true,
		(JsonValue::Boolean(a), JsonValue::Boolean(b)) => a == b,
		(JsonValue::Number(a), JsonValue::Number(b)) => a == b,
		(JsonValue::String(a), JsonValue::String(b)) => (**a) == (**b),
		_ => false,
	}
}
