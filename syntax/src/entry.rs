use locspan::Meta;
use locspan_derive::{StrippedEq, StrippedHash, StrippedPartialEq};
use std::ops::Deref;

#[derive(
	Clone,
	Copy,
	PartialEq,
	StrippedPartialEq,
	StrippedEq,
	StrippedHash,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	Debug,
)]
#[stripped_ignore(M)]
pub struct Entry<T, M> {
	#[stripped_ignore]
	pub key_metadata: M,
	pub value: Meta<T, M>,
}

impl<T, M> Entry<T, M> {
	pub fn new(key_metadata: M, value: Meta<T, M>) -> Self {
		Self {
			key_metadata,
			value,
		}
	}

	pub fn as_value(&self) -> &Meta<T, M> {
		&self.value
	}

	pub fn into_value(self) -> Meta<T, M> {
		self.value
	}

	pub fn borrow_value(&self) -> Entry<&T, M>
	where
		M: Clone,
	{
		Entry::new(self.key_metadata.clone(), self.value.borrow_value())
	}

	pub fn map<U>(self, f: impl Fn(T) -> U) -> Entry<U, M> {
		Entry::new(self.key_metadata, self.value.map(f))
	}

	pub fn cast<U: From<T>>(self) -> Entry<U, M> {
		Entry::new(self.key_metadata, self.value.cast())
	}
}

impl<'a, T, M> Entry<&'a T, M> {
	pub fn into_deref(self) -> Entry<&'a T::Target, M>
	where
		T: Deref,
	{
		let Meta(value, meta) = self.value;
		Entry::new(self.key_metadata, Meta(value.deref(), meta))
	}
}

impl<T, M> std::ops::Deref for Entry<T, M> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.value.value()
	}
}

impl<T, M> std::ops::DerefMut for Entry<T, M> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.value.value_mut()
	}
}
