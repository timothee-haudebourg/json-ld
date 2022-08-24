use std::hash::{BuildHasher, Hash, Hasher};

#[derive(Default, Clone, Copy)]
pub struct DeterministicHasherBuilder;

impl BuildHasher for DeterministicHasherBuilder {
	type Hasher = std::collections::hash_map::DefaultHasher;

	fn build_hasher(&self) -> Self::Hasher {
		Self::Hasher::new()
	}
}

use locspan::{StrippedEq, StrippedHash, StrippedPartialEq};

/// Multiset of values.
#[derive(Clone)]
pub struct Multiset<T, S = DeterministicHasherBuilder> {
	data: Vec<T>,
	hasher: S,
}

impl<T, S: Default> Default for Multiset<T, S> {
	fn default() -> Self {
		Self {
			data: Vec::new(),
			hasher: S::default(),
		}
	}
}

impl<T, S> Multiset<T, S> {
	pub fn new() -> Self
	where
		S: Default,
	{
		Self::default()
	}

	pub fn with_capacity(cap: usize) -> Self
	where
		S: Default,
	{
		Self {
			data: Vec::with_capacity(cap),
			hasher: S::default(),
		}
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}

	pub fn contains(&self, value: &T) -> bool
	where
		T: PartialEq,
	{
		self.data.contains(value)
	}

	pub fn iter(&self) -> core::slice::Iter<T> {
		self.data.iter()
	}

	pub fn iter_mut(&mut self) -> core::slice::IterMut<T> {
		self.data.iter_mut()
	}

	pub fn as_slice(&self) -> &[T] {
		&self.data
	}

	// pub fn into_stripped(self) -> Multiset<locspan::Stripped<T>, S> {
	// 	Multiset { data: unsafe { core::mem::transmute(self.data) }, hasher: self.hasher }
	// }
}

impl<T: Hash, S: BuildHasher> Multiset<T, S> {
	pub fn singleton(value: T) -> Self
	where
		S: Default,
	{
		let mut result = Self::new();
		result.insert(value);
		result
	}

	pub fn insert(&mut self, value: T) {
		self.data.push(value);
	}

	pub fn insert_unique(&mut self, value: T) -> bool
	where
		T: PartialEq,
	{
		if self.contains(&value) {
			false
		} else {
			self.insert(value);
			true
		}
	}
}

// impl<T, S> Multiset<locspan::Stripped<T>, S> {
// 	pub fn into_unstripped(self) -> Multiset<T, S> {
// 		Multiset { data: unsafe { core::mem::transmute(self.data) }, hasher: self.hasher }
// 	}
// }

// impl<T, S> From<Multiset<locspan::Stripped<T>, S>> for Multiset<T, S> {
// 	fn from(m: Multiset<locspan::Stripped<T>, S>) -> Self {
// 		m.into_unstripped()
// 	}
// }

// impl<T, S> From<Multiset<T, S>> for Multiset<locspan::Stripped<T>, S> {
// 	fn from(m: Multiset<T, S>) -> Self {
// 		m.into_stripped()
// 	}
// }

impl<'a, T, S> IntoIterator for &'a Multiset<T, S> {
	type Item = &'a T;
	type IntoIter = core::slice::Iter<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, T, S> IntoIterator for &'a mut Multiset<T, S> {
	type Item = &'a mut T;
	type IntoIter = core::slice::IterMut<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

impl<T, S> IntoIterator for Multiset<T, S> {
	type Item = T;
	type IntoIter = std::vec::IntoIter<T>;

	fn into_iter(self) -> Self::IntoIter {
		self.data.into_iter()
	}
}

impl<T: Hash, S: Default + BuildHasher> FromIterator<T> for Multiset<T, S> {
	fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
		let mut result = Self::new();

		for item in iter {
			result.insert(item)
		}

		result
	}
}

impl<T: Hash, S: BuildHasher> Extend<T> for Multiset<T, S> {
	fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
		for item in iter {
			self.insert(item)
		}
	}
}

impl<T: PartialEq<U>, U, S, P> PartialEq<Multiset<U, P>> for Multiset<T, S> {
	fn eq(&self, other: &Multiset<U, P>) -> bool {
		compare_unordered(&self.data, &other.data)
	}
}

impl<T: StrippedPartialEq<U>, U, S, P> StrippedPartialEq<Multiset<U, P>> for Multiset<T, S> {
	fn stripped_eq(&self, other: &Multiset<U, P>) -> bool {
		compare_stripped_unordered(&self.data, &other.data)
	}
}

pub(crate) fn compare_unordered<T: PartialEq<U>, U>(a: &[T], b: &[U]) -> bool {
	if a.len() == b.len() {
		let mut free_indexes = Vec::new();
		free_indexes.resize(a.len(), true);

		for item in a {
			match free_indexes
				.iter_mut()
				.enumerate()
				.find(|(i, free)| **free && item == &b[*i])
			{
				Some((_, free)) => *free = false,
				None => return false,
			}
		}

		true
	} else {
		false
	}
}

pub(crate) fn compare_unordered_opt<T: PartialEq<U>, U>(a: Option<&[T]>, b: Option<&[U]>) -> bool {
	match (a, b) {
		(Some(a), Some(b)) => compare_unordered(a, b),
		(None, None) => true,
		_ => false,
	}
}

pub(crate) fn compare_stripped_unordered<T: StrippedPartialEq<U>, U>(a: &[T], b: &[U]) -> bool {
	if a.len() == b.len() {
		let mut free_indexes = Vec::new();
		free_indexes.resize(a.len(), true);

		for item in a {
			match free_indexes
				.iter_mut()
				.enumerate()
				.find(|(i, free)| **free && item.stripped_eq(&b[*i]))
			{
				Some((_, free)) => *free = false,
				None => return false,
			}
		}

		true
	} else {
		false
	}
}

pub(crate) fn compare_stripped_unordered_opt<T: StrippedPartialEq<U>, U>(
	a: Option<&[T]>,
	b: Option<&[U]>,
) -> bool {
	match (a, b) {
		(Some(a), Some(b)) => compare_stripped_unordered(a, b),
		(None, None) => true,
		_ => false,
	}
}

impl<T: Eq, S> Eq for Multiset<T, S> {}

impl<T: StrippedEq, S> StrippedEq for Multiset<T, S> {}

impl<T: Hash, S: BuildHasher> Hash for Multiset<T, S> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		let mut hash = 0u64;

		for item in self {
			let mut hasher = self.hasher.build_hasher();
			item.hash(&mut hasher);
			hash = hash.wrapping_add(hasher.finish());
		}

		state.write_u64(hash)
	}
}

impl<T: StrippedHash, S: BuildHasher> StrippedHash for Multiset<T, S> {
	fn stripped_hash<H: std::hash::Hasher>(&self, state: &mut H) {
		let mut hash = 0u64;

		for item in self {
			let mut hasher = self.hasher.build_hasher();
			item.stripped_hash(&mut hasher);
			hash = hash.wrapping_add(hasher.finish());
		}

		state.write_u64(hash)
	}
}
