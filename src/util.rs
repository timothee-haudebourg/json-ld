use std::hash::{Hash, Hasher};
use std::collections::{HashSet, HashMap, hash_map::DefaultHasher};
use json::{JsonValue, number::Number};

pub fn hash_json_number<H: Hasher>(number: &Number, hasher: &mut H) {
	let (positive, mantissa, exponent) = number.as_parts();
	positive.hash(hasher);
	mantissa.hash(hasher);
	exponent.hash(hasher);
}

pub fn hash_json<H: Hasher>(value: &JsonValue, hasher: &mut H) {
	match value {
		JsonValue::Null => (),
		JsonValue::Boolean(b) => b.hash(hasher),
		JsonValue::Number(n) => hash_json_number(n, hasher),
		JsonValue::Short(str) => str.hash(hasher),
		JsonValue::String(str) => str.hash(hasher),
		JsonValue::Array(ary) => {
			for item in ary {
				hash_json(item, hasher)
			}
		},
		JsonValue::Object(obj) => {
			// in JSON, the order of elements matters, so we don't need to be vigilant here.
			for (key, value) in obj.iter() {
				key.hash(hasher);
				hash_json(value, hasher);
			}
		}
	}
}

pub fn hash_set<T: Hash, H: Hasher>(set: &HashSet<T>, hasher: &mut H) {
	// Elements must be combined with a associative and commutative operation •.
	// (u64, •, 0) must form a commutative monoid.
	// This is satisfied by • = u64::wrapping_add.
	let mut hash = 0;
	for item in set {
		let mut h = DefaultHasher::new();
		item.hash(&mut h);
		hash = u64::wrapping_add(hash, h.finish());
	}

	hasher.write_u64(hash);
}

pub fn hash_set_opt<T: Hash, H: Hasher>(set_opt: &Option<HashSet<T>>, hasher: &mut H) {
	match set_opt.as_ref() {
		Some(set) => hash_set(set, hasher),
		None => ()
	}
}

pub fn hash_map<K: Hash, V: Hash, H: Hasher>(map: &HashMap<K, V>, hasher: &mut H) {
	// Elements must be combined with a associative and commutative operation •.
	// (u64, •, 0) must form a commutative monoid.
	// This is satisfied by • = u64::wrapping_add.
	let mut hash = 0;
	for entry in map {
		let mut h = DefaultHasher::new();
		entry.hash(&mut h);
		hash = u64::wrapping_add(hash, h.finish());
	}

	hasher.write_u64(hash);
}

pub fn hash_map_of_sets<K: Hash, V: Hash, H: Hasher>(map: &HashMap<K, HashSet<V>>, hasher: &mut H) {
	// Elements must be combined with a associative and commutative operation •.
	// (u64, •, 0) must form a commutative monoid.
	// This is satisfied by • = u64::wrapping_add.
	let mut hash = 0;
	for (key, value) in map {
		let mut h = DefaultHasher::new();
		key.hash(&mut h);
		hash_set(value, &mut h);
		hash = u64::wrapping_add(hash, h.finish());
	}

	hasher.write_u64(hash);
}
