use cc_traits::{Get, Iter, Len, MapIter};
use generic_json::{Json, Value, ValueRef};
use langtag::{LanguageTag, LanguageTagBuf};
use std::{collections::HashSet, iter::FromIterator};

pub trait AsJson<J: Json> {
	fn as_json_with<M>(&self, meta: M) -> J
	where
		M: Clone + Fn() -> J::MetaData;

	fn as_json(&self) -> J
	where
		J::MetaData: Default,
	{
		self.as_json_with(|| J::MetaData::default())
	}
}

// impl<J: Json> AsJson<J> for J where J: Clone {
// 	fn as_json_with<M>(&self, _meta: M) -> J where M: Clone + Fn() -> J::MetaData {
// 		self.clone()
// 	}
// }

impl<J: Json> AsJson<J> for bool {
	fn as_json_with<M>(&self, meta: M) -> J
	where
		M: Clone + Fn() -> J::MetaData,
	{
		Value::<J>::Boolean(*self).with(meta())
	}
}

impl<'a, J: Json> AsJson<J> for &'a str
where
	J::String: From<&'a str>,
{
	fn as_json_with<M>(&self, meta: M) -> J
	where
		M: Clone + Fn() -> J::MetaData,
	{
		Value::<J>::String((*self).into()).with(meta())
	}
}

impl<J: Json> AsJson<J> for str
where
	J::String: for<'a> From<&'a str>,
{
	fn as_json_with<M>(&self, meta: M) -> J
	where
		M: Clone + Fn() -> J::MetaData,
	{
		<&str as AsJson<J>>::as_json_with(&self, meta)
	}
}

impl<J: Json> AsJson<J> for String
where
	J::String: for<'a> From<&'a str>,
{
	fn as_json_with<M>(&self, meta: M) -> J
	where
		M: Clone + Fn() -> J::MetaData,
	{
		AsJson::<J>::as_json_with(self.as_str(), meta)
	}
}

impl<'a, T: AsRef<[u8]> + ?Sized, J: Json> AsJson<J> for LanguageTag<'a, T>
where
	J::String: for<'s> From<&'s str>,
{
	fn as_json_with<M>(&self, meta: M) -> J
	where
		M: Clone + Fn() -> J::MetaData,
	{
		AsJson::<J>::as_json_with(self.as_str(), meta)
	}
}

impl<T: AsRef<[u8]>, J: Json> AsJson<J> for LanguageTagBuf<T>
where
	J::String: for<'a> From<&'a str>,
{
	fn as_json_with<M>(&self, meta: M) -> J
	where
		M: Clone + Fn() -> J::MetaData,
	{
		AsJson::<J>::as_json_with(self.as_str(), meta)
	}
}

impl<J: Json, T: AsJson<J>> AsJson<J> for [T]
where
	J::Array: FromIterator<J>,
{
	fn as_json_with<M>(&self, meta: M) -> J
	where
		M: Clone + Fn() -> J::MetaData,
	{
		let array = J::Array::from_iter(self.iter().map(|value| value.as_json_with(meta.clone())));
		Value::<J>::Array(array).with(meta())
	}
}

impl<J: Json, T: AsJson<J>> AsJson<J> for Vec<T>
where
	J::Array: FromIterator<J>,
{
	fn as_json_with<M>(&self, meta: M) -> J
	where
		M: Clone + Fn() -> J::MetaData,
	{
		AsJson::<J>::as_json_with(self.as_slice(), meta)
	}
}

impl<J: Json, T: AsJson<J>> AsJson<J> for HashSet<T>
where
	J::Array: FromIterator<J>,
{
	fn as_json_with<M>(&self, meta: M) -> J
	where
		M: Clone + Fn() -> J::MetaData,
	{
		let array = J::Array::from_iter(self.iter().map(|value| value.as_json_with(meta.clone())));
		Value::<J>::Array(array).with(meta())
	}
}

pub fn json_ld_eq<J: Json, K: Json>(a: &J, b: &K) -> bool
where
	J::Number: PartialEq<K::Number>,
{
	match (a.as_value_ref(), b.as_value_ref()) {
		(ValueRef::Array(a), ValueRef::Array(b)) if a.len() == b.len() => {
			let mut selected = Vec::with_capacity(a.len());
			selected.resize(a.len(), false);

			'a_items: for item in a.iter() {
				for i in 0..b.len() {
					if !selected[i] && json_ld_eq(&*item, &*b.get(i).unwrap()) {
						selected[i] = true;
						continue 'a_items;
					}
				}

				return false;
			}

			true
		}
		(ValueRef::Object(a), ValueRef::Object(b)) if a.len() == b.len() => {
			for (key, value_a) in a.iter() {
				let key = key.as_ref();
				if let Some(value_b) = b.get(key) {
					if key == "@list" {
						match (value_a.as_value_ref(), value_b.as_value_ref()) {
							(ValueRef::Array(item_a), ValueRef::Array(item_b))
								if item_a.len() == item_b.len() =>
							{
								for i in 0..item_a.len() {
									if !json_ld_eq(
										&*item_a.get(i).unwrap(),
										&*item_b.get(i).unwrap(),
									) {
										return false;
									}
								}
							}
							_ => {
								if !json_ld_eq(&*value_a, &*value_b) {
									return false;
								}
							}
						}
					} else if !json_ld_eq(&*value_a, &*value_b) {
						return false;
					}
				} else {
					return false;
				}
			}

			true
		}
		(ValueRef::Null, ValueRef::Null) => true,
		(ValueRef::Number(a), ValueRef::Number(b)) => a == b,
		(ValueRef::String(a), ValueRef::String(b)) => a.as_ref() == b.as_ref(),
		_ => false,
	}
}
