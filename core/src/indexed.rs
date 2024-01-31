use crate::object::{InvalidExpandedJson, TryFromJson, TryFromJsonObject};
use json_ld_syntax::{IntoJson, IntoJsonWithContext};
use rdf_types::VocabularyMut;
use std::convert::{TryFrom, TryInto};
use std::ops::{Deref, DerefMut};

/// Indexed objects.
///
/// Nodes and value objects may be indexed by a string in JSON-LD.
/// This type is a wrapper around any kind of indexable data.
///
/// It is a pointer type that `Deref` into the underlying value.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Indexed<T> {
	/// Index.
	index: Option<String>,

	/// Value.
	value: T,
}

impl<T> Indexed<T> {
	/// Creates a non-indexed value, without metadata.
	pub fn none(value: T) -> Self {
		Self::new(value, None)
	}

	/// Create a new (maybe) indexed value.
	#[inline(always)]
	pub fn new(value: T, index: Option<String>) -> Self {
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

	/// Set the value index.
	#[inline(always)]
	pub fn set_index(&mut self, index: Option<String>) {
		self.index = index
	}

	/// Turn this indexed value into its components: inner value and index.
	#[inline(always)]
	pub fn into_parts(self) -> (T, Option<String>) {
		(self.value, self.index)
	}

	/// Cast the inner value.
	#[inline(always)]
	pub fn map_inner<U, F>(self, f: F) -> Indexed<U>
	where
		F: FnOnce(T) -> U,
	{
		Indexed::new(f(self.value), self.index)
	}

	/// Cast the inner value.
	#[inline(always)]
	pub fn cast<U: From<T>>(self) -> Indexed<U> {
		Indexed::new(self.value.into(), self.index)
	}

	/// Try to cast the inner value.
	#[inline(always)]
	pub fn try_cast<U: TryFrom<T>>(self) -> Result<Indexed<U>, Indexed<U::Error>> {
		match self.value.try_into() {
			Ok(value) => Ok(Indexed::new(value, self.index)),
			Err(e) => Err(Indexed::new(e, self.index)),
		}
	}
}

impl<T, B, O: TryFromJsonObject<T, B>> TryFromJson<T, B> for Indexed<O> {
	fn try_from_json_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		value: json_syntax::Value,
	) -> Result<Self, InvalidExpandedJson> {
		match value {
			json_syntax::Value::Object(object) => Self::try_from_json_object_in(vocabulary, object),
			_ => Err(InvalidExpandedJson::InvalidObject),
		}
	}
}

impl<T, B, O: TryFromJsonObject<T, B>> TryFromJsonObject<T, B> for Indexed<O> {
	fn try_from_json_object_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		mut object: json_syntax::Object,
	) -> Result<Self, InvalidExpandedJson> {
		let index = match object
			.remove_unique("@index")
			.map_err(InvalidExpandedJson::duplicate_key)?
		{
			Some(index_entry) => match index_entry.value {
				json_syntax::Value::String(index) => Some(index.to_string()),
				_ => return Err(InvalidExpandedJson::InvalidIndex),
			},
			None => None,
		};

		let value = O::try_from_json_object_in(vocabulary, object)?;
		Ok(Self::new(value, index))
	}
}

impl<T> From<T> for Indexed<T> {
	#[inline(always)]
	fn from(value: T) -> Indexed<T> {
		Indexed::new(value, None)
	}
}

impl<T> Deref for Indexed<T> {
	type Target = T;

	#[inline(always)]
	fn deref(&self) -> &T {
		&self.value
	}
}

impl<T> DerefMut for Indexed<T> {
	#[inline(always)]
	fn deref_mut(&mut self) -> &mut T {
		&mut self.value
	}
}

impl<T> AsRef<T> for Indexed<T> {
	#[inline(always)]
	fn as_ref(&self) -> &T {
		&self.value
	}
}

impl<T> AsMut<T> for Indexed<T> {
	#[inline(always)]
	fn as_mut(&mut self) -> &mut T {
		&mut self.value
	}
}

impl<T: IntoJsonWithContext<N>, N> IntoJsonWithContext<N> for Indexed<T> {
	fn into_json_with(self, vocabulary: &N) -> json_syntax::Value {
		let mut result = self.value.into_json_with(vocabulary);

		if let Some(obj) = result.as_object_mut() {
			if let Some(index) = self.index {
				obj.insert("@index".into(), index.into_json());
			}
		}

		result
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
