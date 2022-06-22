use cc_traits::{Iter, MapIter};
use generic_json::{
	Json, JsonBuild, JsonClone, JsonIntoMut, JsonMutSendSync, Key, Value, ValueRef,
};
use langtag::{LanguageTag, LanguageTagBuf};

/// JSON value that can be converted from a `J` value.
pub trait JsonFrom<J: Json> = JsonMutSendSync + JsonBuild + JsonIntoMut
where <Self as Json>::Number: From<<J as Json>::Number>;

/// Type composed of `J` JSON values that can be converted
/// into a `K` JSON value.
pub trait AsJson<J: JsonClone, K: Json> {
	/// Converts this value into a `K` JSON value using the given
	/// `meta` function to convert `J::MetaData` into `K::MetaData`.
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K;

	/// Converts this value into a `K` JSON value.
	///
	/// The `K` value is annotated with the default value of `K::MetaData`.
	fn as_json(&self) -> K
	where
		K::MetaData: Default,
	{
		self.as_json_with(|_| K::MetaData::default())
	}
}

/// Type that can be converted into a `K` JSON value.
pub trait AsAnyJson<K: JsonBuild> {
	/// Converts this value into a `K` JSON value using the
	/// given `meta` value as metadata.
	fn as_json_with(&self, meta: K::MetaData) -> K;

	/// Converts this value into a `K` JSON value using the
	/// default metadata value.
	fn as_json(&self) -> K
	where
		K::MetaData: Default,
	{
		self.as_json_with(K::MetaData::default())
	}
}

/// Converts a JSON value into the same JSON value represented with another type.
fn json_to_json<J: JsonClone, K: JsonFrom<J>>(
	input: &J,
	m: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
) -> K {
	let meta: <K as Json>::MetaData = m(Some(input.metadata()));
	match input.as_value_ref() {
		ValueRef::Null => K::null(meta),
		ValueRef::Boolean(b) => K::boolean(b, meta),
		ValueRef::Number(n) => K::number(n.clone().into(), meta),
		ValueRef::String(s) => K::string((**s).into(), meta),
		ValueRef::Array(a) => K::array(
			a.iter()
				.map(|value| json_to_json(&*value, m.clone()))
				.collect(),
			meta,
		),
		ValueRef::Object(o) => K::object(
			o.iter()
				.map(|(key, value)| {
					(
						K::new_key(&**key, m(Some(key.metadata()))),
						json_to_json(&*value, m.clone()),
					)
				})
				.collect(),
			meta,
		),
	}
}

impl<J: JsonClone, K: JsonFrom<J>> AsJson<J, K> for J {
	fn as_json_with(
		&self,
		meta: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
	) -> K {
		json_to_json(self, meta)
	}
}

impl<K: JsonBuild> AsAnyJson<K> for bool {
	fn as_json_with(&self, meta: K::MetaData) -> K {
		Value::<K>::Boolean(*self).with(meta)
	}
}

impl<'a, K: JsonBuild> AsAnyJson<K> for &'a str {
	fn as_json_with(&self, meta: K::MetaData) -> K {
		Value::<K>::String((*self).into()).with(meta)
	}
}

impl<K: JsonBuild> AsAnyJson<K> for str {
	fn as_json_with(&self, meta: K::MetaData) -> K {
		<&str as AsAnyJson<K>>::as_json_with(&self, meta)
	}
}

impl<K: JsonBuild> AsAnyJson<K> for String {
	fn as_json_with(&self, meta: K::MetaData) -> K {
		AsAnyJson::<K>::as_json_with(self.as_str(), meta)
	}
}

impl<'a, K: JsonBuild, T: AsRef<[u8]> + ?Sized> AsAnyJson<K> for LanguageTag<'a, T> {
	fn as_json_with(&self, meta: K::MetaData) -> K {
		AsAnyJson::<K>::as_json_with(self.as_str(), meta)
	}
}

impl<K: JsonBuild, T: AsRef<[u8]>> AsAnyJson<K> for LanguageTagBuf<T> {
	fn as_json_with(&self, meta: K::MetaData) -> K {
		AsAnyJson::<K>::as_json_with(self.as_str(), meta)
	}
}

impl<J: JsonClone, K: JsonFrom<J>, T: AsJson<J, K>> AsJson<J, K> for [T] {
	fn as_json_with(
		&self,
		meta: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
	) -> K {
		let array = <K as Json>::Array::from_iter(
			self.iter().map(|value| value.as_json_with(meta.clone())),
		);
		Value::<K>::Array(array).with(meta(None))
	}
}

// impl<J: JsonClone, K: JsonFrom<J>, T: AsJson<J, K>> AsJson<J, K> for Vec<T> {
// 	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
// 		AsJson::<J, K>::as_json_with(self.as_slice(), meta)
// 	}
// }

// impl<J: JsonClone, K: JsonFrom<J>, T: AsJson<J, K>> AsJson<J, K> for HashSet<T> {
// 	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
// 		let array = self
// 			.iter()
// 			.map(|value| value.as_json_with(meta.clone()))
// 			.collect();
// 		Value::<K>::Array(array).with(meta(None))
// 	}
// }
