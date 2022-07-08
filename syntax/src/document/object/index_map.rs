use super::{Entry, Equivalent, Key};
use core::hash::{BuildHasher, Hash};
use hashbrown::hash_map::DefaultHashBuilder;
use hashbrown::raw::RawTable;

fn make_insert_hash<K, S>(hash_builder: &S, val: &K) -> u64
where
	K: ?Sized + Hash,
	S: BuildHasher,
{
	use core::hash::Hasher;
	let mut state = hash_builder.build_hasher();
	val.hash(&mut state);
	state.finish()
}

fn equivalent_key<'a, C, M, Q>(entries: &'a [Entry<C, M>], k: &'a Q) -> impl 'a + Fn(&usize) -> bool
where
	Q: ?Sized + Equivalent<Key>,
{
	move |i| k.equivalent(entries[*i].key.value())
}

fn make_hasher<'a, C, M, S>(
	entries: &'a [Entry<C, M>],
	hash_builder: &'a S,
) -> impl 'a + Fn(&usize) -> u64
where
	S: BuildHasher,
{
	move |i| make_hash::<S>(hash_builder, entries[*i].key.value())
}

fn make_hash<S>(hash_builder: &S, val: &Key) -> u64
where
	S: BuildHasher,
{
	use core::hash::Hasher;
	let mut state = hash_builder.build_hasher();
	val.hash(&mut state);
	state.finish()
}

#[derive(Clone)]
pub struct IndexMap<S = DefaultHashBuilder> {
	hash_builder: S,
	table: RawTable<usize>,
}

impl<S: Default> Default for IndexMap<S> {
	fn default() -> Self {
		Self {
			hash_builder: S::default(),
			table: RawTable::default(),
		}
	}
}

impl<S: BuildHasher> IndexMap<S> {
	pub fn get<C, M, Q: ?Sized>(&self, entries: &[Entry<C, M>], key: &Q) -> Option<usize>
	where
		Q: Hash + Equivalent<Key>,
	{
		let hash = make_insert_hash(&self.hash_builder, key);
		self.table
			.get(hash, equivalent_key::<C, M, Q>(entries, key))
			.cloned()
	}

	/// Associates the given `key` to `index`.
	///
	/// Returns `true` if the key was not associated to any index.
	pub fn insert<C, M>(&mut self, entries: &[Entry<C, M>], index: usize) -> bool {
		let key = entries[index].key.value();
		let hash = make_insert_hash(&self.hash_builder, key);
		match self
			.table
			.get_mut(hash, equivalent_key::<C, M, _>(entries, key))
		{
			Some(i) => {
				*i = index;
				false
			}
			None => {
				self.table.insert(
					hash,
					index,
					make_hasher::<C, M, S>(entries, &self.hash_builder),
				);
				true
			}
		}
	}

	/// Removes the association between the given key and index.
	pub fn remove<C, M>(&mut self, entries: &[Entry<C, M>], index: usize) {
		let key = entries[index].key.value();
		let hash = make_insert_hash(&self.hash_builder, key);
		self.table
			.remove_entry(hash, equivalent_key::<C, M, _>(entries, key));
	}

	/// Decreases all index greater than `index` by one everywhere in the table.
	pub fn shift(&mut self, index: usize) {
		unsafe {
			for bucket in self.table.iter() {
				let i = bucket.as_mut();
				if *i > index {
					*i -= 1
				}
			}
		}
	}
}
