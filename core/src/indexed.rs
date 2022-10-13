use crate::object::{InvalidExpandedJson, TryFromJson, TryFromJsonObject};
use json_ld_syntax::Entry;
use locspan::Meta;
use locspan_derive::*;
use rdf_types::VocabularyMut;
use std::convert::{TryFrom, TryInto};
use std::ops::{Deref, DerefMut};

/// Indexed objects.
///
/// Nodes and value objects may be indexed by a string in JSON-LD.
/// This type is a wrapper around any kind of indexable data.
///
/// It is a pointer type that `Deref` into the underlying value.
#[derive(
	Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, StrippedPartialEq, StrippedEq, StrippedHash,
)]
#[stripped_ignore(M)]
pub struct Indexed<T, M> {
	/// Index.
	index: Option<Entry<String, M>>,

	/// Value.
	value: T,
}

impl<T, M> Indexed<T, M> {
	/// Create a new (maybe) indexed value.
	#[inline(always)]
	pub fn new(value: T, index: Option<Entry<String, M>>) -> Indexed<T, M> {
		Indexed { value, index }
	}

	/// Get a reference to the inner value.
	#[inline(always)]
	pub fn inner(&self) -> &T {
		&self.value
	}

	pub fn inner_mut(&mut self) -> &mut T {
		&mut self.value
	}

	/// Drop the index and return the underlying value.
	#[inline(always)]
	pub fn into_inner(self) -> T {
		self.value
	}

	/// Get the index, if any.
	#[inline(always)]
	pub fn index(&self) -> Option<&str> {
		match &self.index {
			Some(index) => Some(index.as_str()),
			None => None,
		}
	}

	/// Get the index entry, if any.
	#[inline(always)]
	pub fn index_entry(&self) -> Option<&Entry<String, M>> {
		self.index.as_ref()
	}

	/// Set the value index.
	#[inline(always)]
	pub fn set_index(&mut self, index: Option<Entry<String, M>>) {
		self.index = index
	}

	/// Turn this indexed value into its components: inner value and index.
	#[inline(always)]
	pub fn into_parts(self) -> (T, Option<Entry<String, M>>) {
		(self.value, self.index)
	}

	/// Cast the inner value.
	#[inline(always)]
	pub fn map_inner<U, F>(self, f: F) -> Indexed<U, M>
	where
		F: FnOnce(T) -> U,
	{
		Indexed::new(f(self.value), self.index)
	}

	/// Cast the inner value.
	#[inline(always)]
	pub fn cast<U: From<T>>(self) -> Indexed<U, M> {
		Indexed::new(self.value.into(), self.index)
	}

	/// Try to cast the inner value.
	#[inline(always)]
	pub fn try_cast<U: TryFrom<T>>(self) -> Result<Indexed<U, M>, Indexed<U::Error, M>> {
		match self.value.try_into() {
			Ok(value) => Ok(Indexed::new(value, self.index)),
			Err(e) => Err(Indexed::new(e, self.index)),
		}
	}
}

impl<T, B, M, O: TryFromJsonObject<T, B, M>> TryFromJson<T, B, M> for Indexed<O, M> {
	fn try_from_json_in(
		vocabulary: &mut impl VocabularyMut<Iri=T, BlankId=B>,
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidExpandedJson<M>, M>> {
		match value {
			json_syntax::Value::Object(object) => {
				Self::try_from_json_object_in(vocabulary, Meta(object, meta))
			}
			_ => Err(Meta(InvalidExpandedJson::InvalidObject, meta)),
		}
	}
}

impl<T, B, M, O: TryFromJsonObject<T, B, M>> TryFromJsonObject<T, B, M> for Indexed<O, M> {
	fn try_from_json_object_in(
		vocabulary: &mut impl VocabularyMut<Iri=T, BlankId=B>,
		Meta(mut object, meta): Meta<json_syntax::Object<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidExpandedJson<M>, M>> {
		let index = match object
			.remove_unique("@index")
			.map_err(InvalidExpandedJson::duplicate_key)?
		{
			Some(index_entry) => match index_entry.value {
				Meta(json_syntax::Value::String(index), meta) => Some(Entry::new(
					index_entry.key.into_metadata(),
					Meta(index.to_string(), meta),
				)),
				Meta(_, meta) => return Err(Meta(InvalidExpandedJson::InvalidIndex, meta)),
			},
			None => None,
		};

		let Meta(value, meta) = O::try_from_json_object_in(vocabulary, Meta(object, meta))?;
		Ok(Meta(Self::new(value, index), meta))
	}
}

impl<T, M> From<T> for Indexed<T, M> {
	#[inline(always)]
	fn from(value: T) -> Indexed<T, M> {
		Indexed::new(value, None)
	}
}

impl<T, M> Deref for Indexed<T, M> {
	type Target = T;

	#[inline(always)]
	fn deref(&self) -> &T {
		&self.value
	}
}

impl<T, M> DerefMut for Indexed<T, M> {
	#[inline(always)]
	fn deref_mut(&mut self) -> &mut T {
		&mut self.value
	}
}

impl<T, M> AsRef<T> for Indexed<T, M> {
	#[inline(always)]
	fn as_ref(&self) -> &T {
		&self.value
	}
}

impl<T, M> AsMut<T> for Indexed<T, M> {
	#[inline(always)]
	fn as_mut(&mut self) -> &mut T {
		&mut self.value
	}
}

// impl<J: JsonClone, K: JsonFrom<J>, T: AsJson<J, K>> AsJson<J, K> for Indexed<T> {
// 	fn as_json_with(
// 		&self,
// 		meta: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
// 	) -> K {
// 		let mut json = self.value.as_json_with(meta.clone());

// 		if let Some(obj) = json.as_object_mut() {
// 			if let Some(index) = &self.index {
// 				obj.insert(
// 					K::new_key(Keyword::Index.into_str(), meta(None)),
// 					index.as_json_with(meta(None)),
// 				);
// 			}
// 		}

// 		json
// 	}
// }
