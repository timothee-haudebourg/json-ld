use json_syntax::Value;

pub fn simple_json_ld_eq(a: &Value, b: &Value) -> bool {
	match (a, b) {
		(Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
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
		(Value::Object(a), Value::Object(b)) if a.len() == b.len() => {
			for entry in a.iter() {
				let key = &entry.key;
				let value_a = &entry.value;
				if let Some(value_b) = b.get(key).next() {
					if key == "@list" {
						match (value_a, value_b) {
							(Value::Array(item_a), Value::Array(item_b))
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
		(Value::Null, Value::Null) => true,
		(Value::Boolean(a), Value::Boolean(b)) => a == b,
		(Value::Number(a), Value::Number(b)) => a == b,
		(Value::String(a), Value::String(b)) => (**a) == (**b),
		_ => false,
	}
}
