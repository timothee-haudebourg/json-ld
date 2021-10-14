//! Utility functions.
use std::collections::{hash_map::DefaultHasher, HashMap, HashSet};
use std::hash::{Hash, Hasher};

mod json;
pub use self::json::*;

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
	if let Some(set) = set_opt.as_ref() {
		hash_set(set, hasher)
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

// pub fn hash_map_of_sets<K: Hash, V: Hash, H: Hasher>(map: &HashMap<K, HashSet<V>>, hasher: &mut H) {
// 	// Elements must be combined with a associative and commutative operation •.
// 	// (u64, •, 0) must form a commutative monoid.
// 	// This is satisfied by • = u64::wrapping_add.
// 	let mut hash = 0;
// 	for (key, value) in map {
// 		let mut h = DefaultHasher::new();
// 		key.hash(&mut h);
// 		hash_set(value, &mut h);
// 		hash = u64::wrapping_add(hash, h.finish());
// 	}
//
// 	hasher.write_u64(hash);
// }
