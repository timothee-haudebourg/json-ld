//! Utility functions.
use std::collections::{hash_map::DefaultHasher, HashMap, HashSet};
use std::hash::{Hash, Hasher};

mod json;
mod pairings;

pub use self::json::*;
pub use pairings::Pairings;

/// Hash a [`HashSet`].
///
/// The standard library does not provide (yet) a `Hash` implementation
/// for the [`HashSet`] type. This can be used instead.
///
/// Note that this function not particularly strong and does
/// not protect against DoS attacks.
pub fn hash_set<T: Hash, H: Hasher>(set: &HashSet<T>, hasher: &mut H) {
	// See: https://github.com/rust-lang/rust/pull/48366
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

/// Hash an optional [`HashSet`].
pub fn hash_set_opt<T: Hash, H: Hasher>(set_opt: &Option<HashSet<T>>, hasher: &mut H) {
	if let Some(set) = set_opt.as_ref() {
		hash_set(set, hasher)
	}
}

/// Hash a [`HashMap`].
///
/// The standard library does not provide (yet) a `Hash` implementation
/// for the [`HashMap`] type. This can be used instead.
///
/// Note that this function not particularly strong and does
/// not protect against DoS attacks.
pub fn hash_map<K: Hash, V: Hash, H: Hasher>(map: &HashMap<K, V>, hasher: &mut H) {
	// See: https://github.com/rust-lang/rust/pull/48366
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
