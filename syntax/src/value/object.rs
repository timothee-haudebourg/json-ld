use super::Value;
use derivative::Derivative;
pub use json_syntax::object::{Equivalent, Key};
use locspan::Meta;
use locspan_derive::*;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

mod index_map;
use index_map::IndexMap;

/// Object.
///
/// In contrast to JSON, in JSON-LD the keys in objects MUST be unique.
#[derive(
	Clone, StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedOrd, StrippedHash, Debug,
)]
#[stripped_ignore(M)]
pub struct Object<M> {
	entries: Entries<M>,
}

impl<M> Default for Object<M> {
	fn default() -> Self {
		Self {
			entries: Entries::default(),
		}
	}
}

impl<M> Object<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self {
			entries: Entries::with_capacity(cap),
		}
	}

	pub fn into_entries(self) -> Entries<M> {
		self.entries
	}

	pub fn len(&self) -> usize {
		self.entries.len()
	}

	pub fn is_empty(&self) -> bool {
		self.entries.is_empty()
	}

	pub fn context(&self) -> Option<&Entry<M>> {
		self.get("@context")
	}

	pub fn remove_context(&mut self) -> Option<Entry<M>> {
		self.remove("@context")
	}

	// pub fn set_context(
	// 	&mut self,
	// 	key_metadata: M,
	// 	context: Meta<Value<M>, M>,
	// ) -> Option<ContextEntry<M>> {
	// 	self.set_context_entry(Some(ContextEntry::new(key_metadata, context)))
	// }

	// pub fn set_context_entry(
	// 	&mut self,
	// 	mut entry: Option<ContextEntry<M>>,
	// ) -> Option<ContextEntry<M>> {
	// 	core::mem::swap(&mut self.context, &mut entry);
	// 	entry
	// }

	// pub fn append_context(&mut self, context: C)
	// where
	// 	C: AnyValueMut,
	// 	M: Default,
	// {
	// 	match self.context.as_mut() {
	// 		None => {
	// 			self.context = Some(ContextEntry::new(M::default(), Meta(context, M::default())))
	// 		}
	// 		Some(c) => c.value.append(context),
	// 	}
	// }

	// pub fn append_context_with(&mut self, key_metadata: M, context: Meta<Value<M>, M>)
	// where
	// 	C: AnyValueMut,
	// {
	// 	match self.context.as_mut() {
	// 		None => self.context = Some(ContextEntry::new(key_metadata, context)),
	// 		Some(c) => c.value.append(context.into_value()),
	// 	}
	// }

	pub fn entries(&self) -> &Entries<M> {
		&self.entries
	}

	pub fn iter(&self) -> core::slice::Iter<Entry<M>> {
		self.entries.iter()
	}

	/// Returns an iterator over the entries matching the given key.
	///
	/// Runs in `O(1)` (average).
	pub fn get<'a, Q: ?Sized>(&'a self, key: &Q) -> Option<&'a Entry<M>>
	where
		Q: Hash + Equivalent<Key>,
	{
		self.entries.get(key)
	}

	pub fn index_of<Q: ?Sized>(&self, key: &Q) -> Option<usize>
	where
		Q: Hash + Equivalent<Key>,
	{
		self.entries.index_of(key)
	}

	/// Inserts the given key-value pair.
	///
	/// If one or more entries are already matching the given key,
	/// all of them are removed and returned in the resulting iterator.
	/// Otherwise, `None` is returned.
	pub fn insert(
		&mut self,
		key: Meta<Key, M>,
		value: Meta<Value<M>, M>,
	) -> Option<Entry<M>> {
		self.entries.insert(key, value)
	}

	pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<Entry<M>>
	where
		Q: Hash + Equivalent<Key>,
	{
		self.entries.remove(key)
	}
}

impl<M: PartialEq> PartialEq for Object<M> {
	fn eq(&self, other: &Self) -> bool {
		self.entries == other.entries
	}
}

impl<M: Eq> Eq for Object<M> {}

impl<M: PartialOrd> PartialOrd for Object<M> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.entries.partial_cmp(&other.entries)
	}
}

impl<M: Ord> Ord for Object<M> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.entries.cmp(&other.entries)
	}
}

impl<M: Hash> Hash for Object<M> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.entries.hash(state)
	}
}

impl<'a, M> IntoIterator for &'a Object<M> {
	type IntoIter = core::slice::Iter<'a, Entry<M>>;
	type Item = &'a Entry<M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<M> IntoIterator for Object<M> {
	type IntoIter = std::vec::IntoIter<Entry<M>>;
	type Item = Entry<M>;

	fn into_iter(self) -> Self::IntoIter {
		self.entries.into_iter()
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
pub struct Entry<M> {
	#[stripped_deref]
	pub key: Meta<Key, M>,
	pub value: Meta<Value<M>, M>,
}

impl<M> Entry<M> {
	pub fn new(key: Meta<Key, M>, value: Meta<Value<M>, M>) -> Self {
		Self { key, value }
	}

	#[allow(clippy::type_complexity)]
	pub fn as_pair(&self) -> (&Meta<Key, M>, &Meta<Value<M>, M>) {
		(&self.key, &self.value)
	}

	#[allow(clippy::type_complexity)]
	pub fn into_pair(self) -> (Meta<Key, M>, Meta<Value<M>, M>) {
		(self.key, self.value)
	}

	pub fn as_key(&self) -> &Meta<Key, M> {
		&self.key
	}

	pub fn into_key(self) -> Meta<Key, M> {
		self.key
	}

	pub fn as_value(&self) -> &Meta<Value<M>, M> {
		&self.value
	}

	pub fn into_value(self) -> Meta<Value<M>, M> {
		self.value
	}
}

/// Object.
#[derive(
	Clone, StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedOrd, StrippedHash, Derivative,
)]
#[derivative(Default(bound = ""))]
#[stripped_ignore(M)]
pub struct Entries<M> {
	/// The entries of the object, in order.
	entries: Vec<Entry<M>>,

	/// Maps each key to
	#[stripped_ignore]
	indexes: IndexMap,
}

impl<M> Entries<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self {
			entries: Vec::with_capacity(cap),
			indexes: IndexMap::with_capacity(cap),
		}
	}

	pub fn len(&self) -> usize {
		self.entries.len()
	}

	pub fn is_empty(&self) -> bool {
		self.entries.is_empty()
	}

	/// Returns an iterator over the entries matching the given key.
	///
	/// Runs in `O(1)` (average).
	pub fn get<Q: ?Sized + Hash + Equivalent<Key>>(&self, key: &Q) -> Option<&Entry<M>> {
		self.indexes
			.get::<M, Q>(&self.entries, key)
			.map(|i| &self.entries[i])
	}

	pub fn as_slice(&self) -> &[Entry<M>] {
		&self.entries
	}

	pub fn iter(&self) -> core::slice::Iter<Entry<M>> {
		self.entries.iter()
	}

	pub fn index_of<Q: ?Sized>(&self, key: &Q) -> Option<usize>
	where
		Q: Hash + Equivalent<Key>,
	{
		self.indexes.get(&self.entries, key)
	}

	/// Inserts the given key-value pair.
	///
	/// If one or more entries are already matching the given key,
	/// all of them are removed and returned in the resulting iterator.
	/// Otherwise, `None` is returned.
	pub fn insert(
		&mut self,
		key: Meta<Key, M>,
		value: Meta<Value<M>, M>,
	) -> Option<Entry<M>> {
		match self.index_of(key.value()) {
			Some(index) => {
				let mut entry = Entry::new(key, value);
				core::mem::swap(&mut entry, &mut self.entries[index]);
				Some(entry)
			}
			None => {
				let index = self.entries.len();
				self.entries.push(Entry::new(key, value));
				self.indexes.insert(&self.entries, index);
				None
			}
		}
	}

	/// Removes the entry at the given index.
	pub fn remove_at(&mut self, index: usize) -> Option<Entry<M>> {
		if index < self.entries.len() {
			self.indexes.remove(&self.entries, index);
			self.indexes.shift(index);
			Some(self.entries.remove(index))
		} else {
			None
		}
	}

	/// Remove the entry associated to the given key.
	///
	/// Runs in `O(n)` time (average).
	pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<Entry<M>>
	where
		Q: Hash + Equivalent<Key>,
	{
		match self.index_of(key) {
			Some(index) => self.remove_at(index),
			None => None,
		}
	}
}

impl<M: PartialEq> PartialEq for Entries<M> {
	fn eq(&self, other: &Self) -> bool {
		self.entries == other.entries
	}
}

impl<M: Eq> Eq for Entries<M> {}

impl<M: PartialOrd> PartialOrd for Entries<M> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.entries.partial_cmp(&other.entries)
	}
}

impl<M: Ord> Ord for Entries<M> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.entries.cmp(&other.entries)
	}
}

impl<M: Hash> Hash for Entries<M> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.entries.hash(state)
	}
}

impl<M: fmt::Debug> fmt::Debug for Entries<M> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_map()
			.entries(self.entries.iter().map(Entry::as_pair))
			.finish()
	}
}

impl<'a, M> IntoIterator for &'a Entries<M> {
	type IntoIter = core::slice::Iter<'a, Entry<M>>;
	type Item = &'a Entry<M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<M> IntoIterator for Entries<M> {
	type IntoIter = std::vec::IntoIter<Entry<M>>;
	type Item = Entry<M>;

	fn into_iter(self) -> Self::IntoIter {
		self.entries.into_iter()
	}
}
