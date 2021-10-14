use generic_json::{Json, JsonHash, ValueRef};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

mod build;

pub use build::*;

pub enum AsArrayItem<'a, J: Json> {
	NotArray(&'a J),
	Array(<J::Array as cc_traits::CollectionRef>::ItemRef<'a>),
}

impl<'a, J: Json> std::ops::Deref for AsArrayItem<'a, J> {
	type Target = J;

	fn deref(&self) -> &J {
		match self {
			Self::NotArray(i) => i,
			Self::Array(i) => i.deref(),
		}
	}
}

pub enum AsArray<'a, J: Json> {
	NotArray(Option<&'a J>),
	Array(<J::Array as cc_traits::Iter>::Iter<'a>),
}

impl<'a, J: Json> Iterator for AsArray<'a, J> {
	type Item = AsArrayItem<'a, J>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::NotArray(item) => item.take().map(AsArrayItem::NotArray),
			Self::Array(ary) => ary.next().map(AsArrayItem::Array),
		}
	}
}

pub fn as_array<J: Json>(json: &J) -> (AsArray<J>, usize) {
	use cc_traits::{Iter, Len};
	match json.as_value_ref() {
		ValueRef::Array(ary) => (AsArray::Array(ary.iter()), ary.len()),
		_ => (AsArray::NotArray(Some(json)), 1),
	}
}

pub fn hash_json<J: JsonHash, H: Hasher>(json: &J, hasher: &mut H) {
	use cc_traits::{Iter, MapIter};
	match json.as_value_ref() {
		ValueRef::Null => (),
		ValueRef::Boolean(b) => b.hash(hasher),
		ValueRef::Number(n) => n.hash(hasher),
		ValueRef::String(s) => s.hash(hasher),
		ValueRef::Array(ary) => {
			for item in ary.iter() {
				hash_json(&*item, hasher)
			}
		}
		ValueRef::Object(obj) => {
			// Elements must be combined with a associative and commutative operation •.
			// (u64, •, 0) must form a commutative monoid.
			// This is satisfied by • = u64::wrapping_add.
			let mut hash = 0;
			for (key, value) in obj.iter() {
				let mut h = DefaultHasher::new();
				key.as_ref().hash(&mut h);
				hash_json(&*value, &mut h);
				hash = u64::wrapping_add(hash, h.finish());
			}
			hasher.write_u64(hash);
		}
	}
}
