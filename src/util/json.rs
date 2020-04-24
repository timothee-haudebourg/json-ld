use std::collections::HashSet;
use json::JsonValue;

pub trait AsJson {
	fn as_json(&self) -> JsonValue;
}

impl AsJson for JsonValue {
	fn as_json(&self) -> JsonValue {
		self.clone()
	}
}

impl AsJson for bool {
	fn as_json(&self) -> JsonValue {
		JsonValue::Boolean(*self)
	}
}

impl AsJson for str {
	fn as_json(&self) -> JsonValue {
		self.into()
	}
}

impl AsJson for String {
	fn as_json(&self) -> JsonValue {
		self.as_str().as_json()
	}
}

impl<T: AsJson> AsJson for [T] {
	fn as_json(&self) -> JsonValue {
		let mut ary = Vec::with_capacity(self.len());
		for item in self {
			ary.push(item.as_json())
		}

		JsonValue::Array(ary)
	}
}

impl<T: AsJson> AsJson for Vec<T> {
	fn as_json(&self) -> JsonValue {
		self.as_slice().as_json()
	}
}

impl<T: AsJson> AsJson for HashSet<T> {
	fn as_json(&self) -> JsonValue {
		let mut ary = Vec::with_capacity(self.len());
		for item in self {
			ary.push(item.as_json())
		}

		JsonValue::Array(ary)
	}
}

pub fn json_ld_eq(a: &JsonValue, b: &JsonValue) -> bool {
	match (a, b) {
		(JsonValue::Array(a), JsonValue::Array(b)) if a.len() == b.len() => {
			let mut selected = Vec::with_capacity(a.len());
			selected.resize(a.len(), false);

			'a_items: for item in a {
				for i in 0..b.len() {
					if !selected[i] && json_ld_eq(&b[i], item) {
						selected[i] = true;
						continue 'a_items
					}
				}

				return false
			}
		},
		(JsonValue::Object(a), JsonValue::Object(b)) if a.len() == b.len() => {
			for (key, value_a) in a.iter() {
				if let Some(value_b) = b.get(key) {
					if key == "@list" {
						match (value_a, value_b) {
							(JsonValue::Array(item_a), JsonValue::Array(item_b)) if item_a.len() == item_b.len() => {
								for i in 0..item_a.len() {
									if !json_ld_eq(&item_a[i], &item_b[i]) {
										return false
									}
								}
							},
							_ => {
								if !json_ld_eq(value_a, value_b) {
									return false
								}
							}
						}
					} else {
						if !json_ld_eq(value_a, value_b) {
							return false
						}
					}
				} else {
					return false
				}
			}
		},
		_ => return a == b
	}

	true
}
