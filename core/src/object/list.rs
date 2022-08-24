use super::{InvalidExpandedJson, MappedEq, Object};
use crate::{Indexed, NamespaceMut, TryFromJson};
use derivative::Derivative;
use json_ld_syntax::{Entry, IntoJson};
use locspan::{Meta, StrippedEq, StrippedPartialEq};
use locspan_derive::StrippedHash;
use std::hash::Hash;

#[derive(Derivative, Clone, Hash, StrippedHash)]
#[derivative(
	PartialEq(bound = "T: Eq + Hash, B: Eq + Hash, M: PartialEq"),
	Eq(bound = "T: Eq + Hash, B: Eq + Hash, M: Eq")
)]
#[stripped_ignore(M)]
#[stripped(T, B)]
/// List object.
pub struct List<T, B, M> {
	entry: Entry<Vec<Meta<Indexed<Object<T, B, M>>, M>>, M>,
}

impl<T, B, M> List<T, B, M> {
	pub fn new(key_metadata: M, value: Meta<Vec<Meta<Indexed<Object<T, B, M>>, M>>, M>) -> Self {
		Self {
			entry: Entry::new(key_metadata, value),
		}
	}

	pub fn len(&self) -> usize {
		self.entry.value.len()
	}

	pub fn is_empty(&self) -> bool {
		self.entry.value.is_empty()
	}

	/// Returns a reference to the "@list" entry of the list object.
	pub fn entry(&self) -> &Entry<Vec<Meta<Indexed<Object<T, B, M>>, M>>, M> {
		&self.entry
	}

	pub fn entry_mut(&mut self) -> &mut Entry<Vec<Meta<Indexed<Object<T, B, M>>, M>>, M> {
		&mut self.entry
	}

	pub fn into_entry(self) -> Entry<Vec<Meta<Indexed<Object<T, B, M>>, M>>, M> {
		self.entry
	}

	pub fn push(&mut self, object: Meta<Indexed<Object<T, B, M>>, M>) {
		self.entry.push(object)
	}

	pub fn pop(&mut self) -> Option<Meta<Indexed<Object<T, B, M>>, M>> {
		self.entry.pop()
	}

	pub fn iter(&self) -> core::slice::Iter<Meta<Indexed<Object<T, B, M>>, M>> {
		self.entry.iter()
	}

	pub fn iter_mut(&mut self) -> core::slice::IterMut<Meta<Indexed<Object<T, B, M>>, M>> {
		self.entry.iter_mut()
	}

	pub fn as_slice(&self) -> &[Meta<Indexed<Object<T, B, M>>, M>] {
		self.entry.as_slice()
	}

	pub fn as_mut_slice(&mut self) -> &mut [Meta<Indexed<Object<T, B, M>>, M>] {
		self.entry.as_mut_slice()
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> List<T, B, M> {
	pub(crate) fn try_from_json_object_in<C: IntoJson<M>>(
		namespace: &mut impl NamespaceMut<T, B>,
		object: json_ld_syntax::Object<C, M>,
		list_entry: json_ld_syntax::object::Entry<C, M>,
	) -> Result<Self, Meta<InvalidExpandedJson, M>> {
		let list = Vec::try_from_json_in(namespace, list_entry.value)?;

		match object.into_iter().next() {
			Some(unexpected_entry) => Err(Meta(
				InvalidExpandedJson::UnexpectedEntry,
				unexpected_entry.key.into_metadata(),
			)),
			None => Ok(Self::new(list_entry.key.into_metadata(), list)),
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> StrippedPartialEq for List<T, B, M> {
	fn stripped_eq(&self, other: &Self) -> bool {
		self.entry.stripped_eq(&other.entry)
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> StrippedEq for List<T, B, M> {}

impl<T: Eq + Hash, B: Eq + Hash, M> MappedEq for List<T, B, M> {
	type BlankId = B;

	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a B) -> &'b B>(&'a self, other: &Self, f: F) -> bool
	where
		B: 'a + 'b,
	{
		self.entry.mapped_eq(&other.entry, f)
	}
}

impl<'a, T, B, M> IntoIterator for &'a List<T, B, M> {
	type Item = &'a Meta<Indexed<Object<T, B, M>>, M>;
	type IntoIter = core::slice::Iter<'a, Meta<Indexed<Object<T, B, M>>, M>>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, T, B, M> IntoIterator for &'a mut List<T, B, M> {
	type Item = &'a mut Meta<Indexed<Object<T, B, M>>, M>;
	type IntoIter = core::slice::IterMut<'a, Meta<Indexed<Object<T, B, M>>, M>>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

impl<T, B, M> IntoIterator for List<T, B, M> {
	type Item = Meta<Indexed<Object<T, B, M>>, M>;
	type IntoIter = std::vec::IntoIter<Meta<Indexed<Object<T, B, M>>, M>>;

	fn into_iter(self) -> Self::IntoIter {
		self.entry.value.into_value().into_iter()
	}
}

pub type EntryRef<'a, T, B, M> = &'a Entry<Vec<Meta<Indexed<Object<T, B, M>>, M>>, M>;

pub type EntryValueRef<'a, T, B, M> = &'a [Meta<Indexed<Object<T, B, M>>, M>];

/// List object fragment.
pub enum FragmentRef<'a, T, B, M> {
	/// "@list" entry.
	Entry(EntryRef<'a, T, B, M>),

	/// "@list" entry key.
	Key(&'a M),

	/// "@list" value.
	Value(EntryValueRef<'a, T, B, M>),
}
