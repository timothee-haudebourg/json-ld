use crate::context::AnyValueMut;

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
pub struct Object<M, C> {
	context: Option<ContextEntry<M, C>>,
	entries: Entries<M, C>,
}

impl<M, C> Default for Object<M, C> {
	fn default() -> Self {
		Self {
			context: None,
			entries: Entries::default(),
		}
	}
}

impl<M, C> Object<M, C> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self {
			context: None,
			entries: Entries::with_capacity(cap),
		}
	}

	pub fn into_parts(self) -> (Option<ContextEntry<M, C>>, Entries<M, C>) {
		(self.context, self.entries)
	}

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

	pub fn set_context(
		&mut self,
		key_metadata: M,
		context: Meta<C, M>,
	) -> Option<ContextEntry<M, C>> {
		self.set_context_entry(Some(ContextEntry::new(key_metadata, context)))
	}

	pub fn context_entry(&self) -> Option<&ContextEntry<M, C>> {
		self.context.as_ref()
	}

	pub fn set_context_entry(
		&mut self,
		mut entry: Option<ContextEntry<M, C>>,
	) -> Option<ContextEntry<M, C>> {
		core::mem::swap(&mut self.context, &mut entry);
		entry
	}

	pub fn remove_context(&mut self) -> Option<ContextEntry<M, C>> {
		self.context.take()
	}

	pub fn append_context(&mut self, context: C)
	where
		C: AnyValueMut,
		M: Default,
	{
		match self.context.as_mut() {
			None => {
				self.context = Some(ContextEntry::new(M::default(), Meta(context, M::default())))
			}
			Some(c) => c.value.append(context),
		}
	}

	pub fn append_context_with(&mut self, key_metadata: M, context: Meta<C, M>)
	where
		C: AnyValueMut,
	{
		match self.context.as_mut() {
			None => self.context = Some(ContextEntry::new(key_metadata, context)),
			Some(c) => c.value.append(context.into_value()),
		}
	}

	pub fn entries_with_context(&self) -> EntriesWithContext<M, C> {
		EntriesWithContext {
			context: self.context.as_ref(),
			entries: self.entries.iter(),
		}
	}

	pub fn entries(&self) -> &Entries<M, C> {
		&self.entries
	}

	pub fn iter(&self) -> core::slice::Iter<Entry<M, C>> {
		self.entries.iter()
	}

	/// Returns an iterator over the entries matching the given key.
	///
	/// Runs in `O(1)` (average).
	pub fn get<'a, Q: ?Sized>(&'a self, key: &Q) -> Option<&'a Entry<M, C>>
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
		value: Meta<Value<M, C>, M>,
	) -> Option<Entry<M, C>> {
		self.entries.insert(key, value)
	}

	pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<Entry<M, C>>
	where
		Q: Hash + Equivalent<Key>,
	{
		self.entries.remove(key)
	}
}

impl<C: PartialEq, M: PartialEq> PartialEq for Object<M, C> {
	fn eq(&self, other: &Self) -> bool {
		self.entries == other.entries
	}
}

impl<C: Eq, M: Eq> Eq for Object<M, C> {}

impl<C: PartialOrd, M: PartialOrd> PartialOrd for Object<M, C> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.entries.partial_cmp(&other.entries)
	}
}

impl<C: Ord, M: Ord> Ord for Object<M, C> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.entries.cmp(&other.entries)
	}
}

impl<C: Hash, M: Hash> Hash for Object<M, C> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.entries.hash(state)
	}
}

impl<'a, M, C> IntoIterator for &'a Object<M, C> {
	type IntoIter = core::slice::Iter<'a, Entry<M, C>>;
	type Item = &'a Entry<M, C>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<M, C> IntoIterator for Object<M, C> {
	type IntoIter = std::vec::IntoIter<Entry<M, C>>;
	type Item = Entry<M, C>;

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
pub struct Entry<M, C> {
	#[stripped_deref]
	pub key: Meta<Key, M>,
	pub value: Meta<Value<M, C>, M>,
}

impl<M, C> Entry<M, C> {
	pub fn new(key: Meta<Key, M>, value: Meta<Value<M, C>, M>) -> Self {
		Self { key, value }
	}

	#[allow(clippy::type_complexity)]
	pub fn as_pair(&self) -> (&Meta<Key, M>, &Meta<Value<M, C>, M>) {
		(&self.key, &self.value)
	}

	#[allow(clippy::type_complexity)]
	pub fn into_pair(self) -> (Meta<Key, M>, Meta<Value<M, C>, M>) {
		(self.key, self.value)
	}

	pub fn as_key(&self) -> &Meta<Key, M> {
		&self.key
	}

	pub fn into_key(self) -> Meta<Key, M> {
		self.key
	}

	pub fn as_value(&self) -> &Meta<Value<M, C>, M> {
		&self.value
	}

	pub fn into_value(self) -> Meta<Value<M, C>, M> {
		self.value
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
pub struct ContextEntry<M, C> {
	#[stripped_ignore]
	pub key_metadata: M,
	pub value: Meta<C, M>,
}

impl<M, C> ContextEntry<M, C> {
	pub fn new(key_metadata: M, value: Meta<C, M>) -> Self {
		Self {
			key_metadata,
			value,
		}
	}

	pub fn into_context(self) -> Meta<C, M> {
		self.value
	}
}

pub struct EntriesWithContext<'a, M, C> {
	context: Option<&'a ContextEntry<M, C>>,
	entries: core::slice::Iter<'a, Entry<M, C>>,
}

impl<'a, M, C> Iterator for EntriesWithContext<'a, M, C> {
	type Item = AnyEntryRef<'a, M, C>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = self.entries.len() + if self.context.is_some() { 1 } else { 0 };
		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		match self.context.take() {
			Some(e) => Some(AnyEntryRef::Context(e)),
			None => self.entries.next().map(AnyEntryRef::Entry),
		}
	}
}

impl<'a, M, C> ExactSizeIterator for EntriesWithContext<'a, M, C> {}

impl<'a, M, C> DoubleEndedIterator for EntriesWithContext<'a, M, C> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match self.entries.next_back() {
			Some(e) => Some(AnyEntryRef::Entry(e)),
			None => self.context.take().map(AnyEntryRef::Context),
		}
	}
}

pub enum AnyEntryRef<'a, M, C> {
	Context(&'a ContextEntry<M, C>),
	Entry(&'a Entry<M, C>),
}

impl<'a, M, C> AnyEntryRef<'a, M, C> {
	pub fn key(&self) -> Meta<AnyKeyRef<'a>, &'a M> {
		match self {
			Self::Context(e) => Meta(AnyKeyRef::Context, &e.key_metadata),
			Self::Entry(e) => Meta(AnyKeyRef::Key(e.key.value()), e.key.metadata()),
		}
	}

	pub fn value(&self) -> Meta<AnyValueRef<'a, M, C>, &'a M> {
		match self {
			Self::Context(e) => Meta(AnyValueRef::Context(&e.value), &e.key_metadata),
			Self::Entry(e) => Meta(AnyValueRef::Value(e.value.value()), e.value.metadata()),
		}
	}

	pub fn is_context(&self) -> bool {
		matches!(self, Self::Context(_))
	}
}

pub enum AnyKeyRef<'a> {
	Context,
	Key(&'a Key),
}

impl<'a> AnyKeyRef<'a> {
	pub fn is_context(&self) -> bool {
		matches!(self, Self::Context)
	}

	pub fn as_str(&self) -> &'a str {
		match self {
			Self::Context => "@context",
			Self::Key(k) => k.as_str(),
		}
	}
}

pub enum AnyValueRef<'a, M, C> {
	Context(&'a C),
	Value(&'a Value<M, C>),
}

/// Object.
#[derive(
	Clone, StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedOrd, StrippedHash, Derivative,
)]
#[derivative(Default(bound = ""))]
#[stripped_ignore(M)]
pub struct Entries<M, C> {
	/// The entries of the object, in order.
	entries: Vec<Entry<M, C>>,

	/// Maps each key to
	#[stripped_ignore]
	indexes: IndexMap,
}

impl<M, C> Entries<M, C> {
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
	pub fn get<Q: ?Sized + Hash + Equivalent<Key>>(&self, key: &Q) -> Option<&Entry<M, C>> {
		self.indexes
			.get::<M, C, Q>(&self.entries, key)
			.map(|i| &self.entries[i])
	}

	pub fn as_slice(&self) -> &[Entry<M, C>] {
		&self.entries
	}

	pub fn iter(&self) -> core::slice::Iter<Entry<M, C>> {
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
		value: Meta<Value<M, C>, M>,
	) -> Option<Entry<M, C>> {
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
	pub fn remove_at(&mut self, index: usize) -> Option<Entry<M, C>> {
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
	pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<Entry<M, C>>
	where
		Q: Hash + Equivalent<Key>,
	{
		match self.index_of(key) {
			Some(index) => self.remove_at(index),
			None => None,
		}
	}
}

impl<C: PartialEq, M: PartialEq> PartialEq for Entries<M, C> {
	fn eq(&self, other: &Self) -> bool {
		self.entries == other.entries
	}
}

impl<C: Eq, M: Eq> Eq for Entries<M, C> {}

impl<C: PartialOrd, M: PartialOrd> PartialOrd for Entries<M, C> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.entries.partial_cmp(&other.entries)
	}
}

impl<C: Ord, M: Ord> Ord for Entries<M, C> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.entries.cmp(&other.entries)
	}
}

impl<C: Hash, M: Hash> Hash for Entries<M, C> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.entries.hash(state)
	}
}

impl<C: fmt::Debug, M: fmt::Debug> fmt::Debug for Entries<M, C> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_map()
			.entries(self.entries.iter().map(Entry::as_pair))
			.finish()
	}
}

impl<'a, M, C> IntoIterator for &'a Entries<M, C> {
	type IntoIter = core::slice::Iter<'a, Entry<M, C>>;
	type Item = &'a Entry<M, C>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<M, C> IntoIterator for Entries<M, C> {
	type IntoIter = std::vec::IntoIter<Entry<M, C>>;
	type Item = Entry<M, C>;

	fn into_iter(self) -> Self::IntoIter {
		self.entries.into_iter()
	}
}
