use json_syntax::Value;
use locspan::Meta;

pub fn simple_json_ld_eq<M, N>(a: &Value<M>, b: &Value<N>) -> bool {
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
				let key = entry.key.value();
				let value_a = entry.value.value();
				if let Some(Meta(value_b, _)) = b.get(key).next() {
					if key == "@list" {
						match (value_a, value_b) {
							(Value::Array(item_a), Value::Array(item_b))
								if item_a.len() == item_b.len() =>
							{
								if !item_a
									.iter()
									.zip(item_b)
									.all(|(a, b)| simple_json_ld_eq(a.value(), b.value()))
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
