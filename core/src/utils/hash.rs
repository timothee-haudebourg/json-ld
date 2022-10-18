use locspan::{Stripped, StrippedHash};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};

/// Hash a [`HashSet`].
///
/// The standard library does not provide (yet) a `Hash` implementation
/// for the [`HashSet`] type. This can be used instead.
///
/// Note that this function not particularly strong and does
/// not protect against DoS attacks.
pub fn hash_set<S: IntoIterator, H: Hasher>(set: S, hasher: &mut H)
where
	S::Item: Hash,
{
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

pub fn hash_set_stripped<S: IntoIterator, H: Hasher>(set: S, hasher: &mut H)
where
	S::Item: StrippedHash,
{
	// See: https://github.com/rust-lang/rust/pull/48366
	// Elements must be combined with a associative and commutative operation •.
	// (u64, •, 0) must form a commutative monoid.
	// This is satisfied by • = u64::wrapping_add.
	let mut hash = 0;
	for item in set {
		let mut h = DefaultHasher::new();
		item.stripped_hash(&mut h);
		hash = u64::wrapping_add(hash, h.finish());
	}

	hasher.write_u64(hash);
}

/// Hash an optional [`HashSet`].
pub fn hash_set_opt<S: IntoIterator, H: Hasher>(set_opt: Option<S>, hasher: &mut H)
where
	S::Item: Hash,
{
	if let Some(set) = set_opt {
		hash_set(set, hasher)
	}
}

pub fn hash_set_stripped_opt<S: IntoIterator, H: Hasher>(set_opt: Option<S>, hasher: &mut H)
where
	S::Item: StrippedHash,
{
	if let Some(set) = set_opt {
		hash_set_stripped(set, hasher)
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

pub fn hash_map_stripped<K: Hash, V: StrippedHash, H: Hasher>(map: &HashMap<K, V>, hasher: &mut H) {
	// See: https://github.com/rust-lang/rust/pull/48366
	// Elements must be combined with a associative and commutative operation •.
	// (u64, •, 0) must form a commutative monoid.
	// This is satisfied by • = u64::wrapping_add.
	let mut hash = 0;
	for (k, v) in map {
		let mut h = DefaultHasher::new();
		(k, Stripped(v)).hash(&mut h);
		hash = u64::wrapping_add(hash, h.finish());
	}

	hasher.write_u64(hash);
}
