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
#[locspan(ignore(M))]
pub struct Entry<T, M = ()> {
	#[locspan(ignore)]
	pub key_metadata: M,
	pub value: Meta<T, M>,
}

impl<T> Entry<T> {
	pub fn new(value: T) -> Self {
		Self::new_with((), Meta::none(value))
	}
}

impl<T, M> Entry<T, M> {
	pub fn new_with(key_metadata: M, value: Meta<T, M>) -> Self {
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
		Entry::new_with(self.key_metadata.clone(), self.value.borrow_value())
	}

	pub fn map<U>(self, f: impl Fn(T) -> U) -> Entry<U, M> {
		Entry::new_with(self.key_metadata, self.value.map(f))
	}

	pub fn cast<U: From<T>>(self) -> Entry<U, M> {
		Entry::new_with(self.key_metadata, self.value.cast())
	}
}

impl<'a, T, M> Entry<&'a T, M> {
	pub fn into_deref(self) -> Entry<&'a T::Target, M>
	where
		T: Deref,
	{
		let Meta(value, meta) = self.value;
		Entry::new_with(self.key_metadata, Meta(value.deref(), meta))
	}
}

impl<T, M> std::ops::Deref for Entry<T, M> {
	type Target = Meta<T, M>;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<T, M> std::ops::DerefMut for Entry<T, M> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.value
	}
}

impl<T: serde::Serialize, M> serde::Serialize for Entry<T, M> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.value.serialize(serializer)
	}
}

impl<'de, T: serde::Deserialize<'de>, M: Default> serde::Deserialize<'de> for Entry<T, M> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		Ok(Self::new_with(
			M::default(),
			Meta(T::deserialize(deserializer)?, M::default()),
		))
	}
}
