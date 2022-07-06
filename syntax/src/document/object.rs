pub use json_syntax::object::{Key, Equivalent};
use locspan::Meta;
use locspan_derive::*;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::fmt;
use super::Value;

mod index_map;
use index_map::IndexMap;

/// Object.
#[derive(
	Clone,
	StrippedPartialEq,
	StrippedEq,
	StrippedPartialOrd,
	StrippedOrd,
	StrippedHash,
	Debug
)]
#[stripped_ignore(M)]
pub struct Object<C, M> {
	context: Option<ContextEntry<C, M>>,
	entries: Entries<C, M>
}

impl<C, M> Object<C, M> {
	pub fn len(&self) -> usize {
		if self.context.is_some() {
			1 + self.entries.len()
		} else {
			self.entries.len()
		}
	}

	pub fn is_empty(&self) -> bool {
		self.context.is_none() && self.entries.is_empty()
	}

	pub fn context(&self) -> Option<&Meta<C, M>> {
		self.context.as_ref().map(|e| &e.value)
	}

	pub fn context_entry(&self) -> Option<&ContextEntry<C, M>> {
		self.context.as_ref()
	}

	pub fn entries(&self) -> &Entries<C, M> {
		&self.entries
	}

	/// Returns an iterator over the entries matching the given key.
	/// 
	/// Runs in `O(1)` (average).
	pub fn get<'a, Q: ?Sized>(&'a self, key: &Q) -> Option<&'a Entry<C, M>> where Q: Hash + Equivalent<Key> {
		self.entries.get(key)
	}
}

impl<C: PartialEq, M: PartialEq> PartialEq for Object<C, M> {
	fn eq(&self, other: &Self) -> bool {
		self.entries == other.entries
	}
}

impl<C: Eq, M: Eq> Eq for Object<C, M> {}

impl<C: PartialOrd, M: PartialOrd> PartialOrd for Object<C, M> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.entries.partial_cmp(&other.entries)
	}
}

impl<C: Ord, M: Ord> Ord for Object<C, M> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.entries.cmp(&other.entries)
	}
}

impl<C: Hash, M: Hash> Hash for Object<C, M> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.entries.hash(state)
	}
}

#[derive(
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	Debug,
	StrippedPartialEq,
	StrippedEq,
	StrippedPartialOrd,
	StrippedOrd,
	StrippedHash,
)]
#[stripped_ignore(M)]
pub struct Entry<C, M> {
	#[stripped_deref]
	pub key: Meta<Key, M>,
	pub value: Meta<Value<C, M>, M>
}

impl<C, M> Entry<C, M> {
	pub fn as_pair(&self) -> (&Meta<Key, M>, &Meta<Value<C, M>, M>) {
		(&self.key, &self.value)
	}
}

#[derive(
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	Debug,
	StrippedPartialEq,
	StrippedEq,
	StrippedPartialOrd,
	StrippedOrd,
	StrippedHash,
)]
#[stripped_ignore(M)]
pub struct ContextEntry<C, M> {
	#[stripped_ignore]
	pub key_metadata: M,
	pub value: Meta<C, M>
}

/// Object.
#[derive(
	Clone,
	StrippedPartialEq,
	StrippedEq,
	StrippedPartialOrd,
	StrippedOrd,
	StrippedHash,
)]
#[stripped_ignore(M)]
pub struct Entries<C, M> {
	/// The entries of the object, in order.
	entries: Vec<Entry<C, M>>,

	/// Maps each key to 
	#[stripped_ignore] indexes: IndexMap
}

impl<C, M> Entries<C, M> {
	pub fn len(&self) -> usize {
		self.entries.len()
	}

	pub fn is_empty(&self) -> bool {
		self.entries.is_empty()
	}

	/// Returns an iterator over the entries matching the given key.
	/// 
	/// Runs in `O(1)` (average).
	pub fn get<'a, Q: ?Sized>(&'a self, key: &Q) -> Option<&'a Entry<C, M>> where Q: Hash + Equivalent<Key> {
		//self.indexes.get::<C, M, Q>(&self.entries, key).map(|i| &self.entries[i])
		todo!()
	}

	pub fn as_slice(&self) -> &[Entry<C, M>] {
		&self.entries
	}

	pub fn iter(&self) -> core::slice::Iter<Entry<C, M>> {
		self.entries.iter()
	}
}

impl<C: PartialEq, M: PartialEq> PartialEq for Entries<C, M> {
	fn eq(&self, other: &Self) -> bool {
		self.entries == other.entries
	}
}

impl<C: Eq, M: Eq> Eq for Entries<C, M> {}

impl<C: PartialOrd, M: PartialOrd> PartialOrd for Entries<C, M> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.entries.partial_cmp(&other.entries)
	}
}

impl<C: Ord, M: Ord> Ord for Entries<C, M> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.entries.cmp(&other.entries)
	}
}

impl<C: Hash, M: Hash> Hash for Entries<C, M> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.entries.hash(state)
	}
}

impl<C: fmt::Debug, M: fmt::Debug> fmt::Debug for Entries<C, M> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_map().entries(self.entries.iter().map(Entry::as_pair)).finish()
	}
}

impl<'a, C, M> IntoIterator for &'a Entries<C, M> {
	type IntoIter = core::slice::Iter<'a, Entry<C, M>>;
	type Item = &'a Entry<C, M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<C, M> IntoIterator for Entries<C, M> {
	type IntoIter = std::vec::IntoIter<Entry<C, M>>;
	type Item = Entry<C, M>;

	fn into_iter(self) -> Self::IntoIter {
		self.entries.into_iter()
	}
}