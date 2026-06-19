use json_syntax::JsonValue;

/// JSON-LD comparison.
pub trait JsonLdCompare {
	fn compare_json_ld(&self, other: &Self) -> bool;
}

impl JsonLdCompare for JsonValue {
	fn compare_json_ld(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Null, Self::Null) => true,
			(Self::Boolean(a), Self::Boolean(b)) => a == b,
			(Self::Number(a), Self::Number(b)) => a == b,
			(Self::String(a), Self::String(b)) => a == b,
			(Self::Array(a), Self::Array(b)) if a.len() == b.len() => {
				let mut selected = Vec::new();
				selected.resize(b.len(), false);

				'next_item: for item in a {
					for (other, selected) in b.iter().zip(selected.iter_mut()) {
						if !*selected && item.compare_json_ld(other) {
							*selected = true;
							continue 'next_item;
						}
					}

					return false;
				}

				true
			}
			(Self::Object(a), Self::Object(b)) if a.len() == b.len() => {
				for entry in a {
					match b.get_unique(entry.0).expect("invalid JSON-LD") {
						Some(value) => {
							if !entry.1.compare_json_ld(value) {
								return false;
							}
						}
						None => return false,
					}
				}

				true
			}
			_ => false,
		}
	}
}
