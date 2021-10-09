use cc_traits::{Get, Iter, Len, MapIter};
use generic_json::{Json, JsonClone, JsonBuild, JsonMut, Value, ValueRef};
use langtag::{LanguageTag, LanguageTagBuf};
use std::collections::HashSet;

pub trait JsonFrom<J: JsonClone> = JsonBuild + JsonMut + From<J>
where
	<Self as Json>::Number: From<<J as Json>::Number>;

pub trait AsJson<J: JsonClone, K: JsonFrom<J>> {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K;
	
	fn as_json(&self) -> K where K::MetaData: Default {
		self.as_json_with(|_| K::MetaData::default())
	}
}

pub fn json_to_json<J: JsonClone, K: JsonFrom<J>>(input: &J, m: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
	let meta: K::MetaData = m(Some(input.metadata()));
	match input.as_value_ref() {
		ValueRef::Null => K::null(meta),
		ValueRef::Boolean(b) => K::boolean(b, meta),
		ValueRef::Number(n) => K::number(n.clone().into(), meta),
		ValueRef::String(s) => K::string(s.as_ref().into(), meta),
		ValueRef::Array(a) => K::array(a.iter().map(|value| json_to_json(&*value, m.clone())).collect(), meta),
		ValueRef::Object(o) => K::object(o.iter().map(|(key, value)| (key.as_ref().into(), json_to_json(&*value, m.clone()))).collect(), meta)
	}
}

impl<J: JsonClone, K: JsonFrom<J>> AsJson<J, K> for bool {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		Value::<K>::Boolean(*self).with(meta(None))
	}
}

impl<'a, J: JsonClone, K: JsonFrom<J>> AsJson<J, K> for &'a str {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		Value::<K>::String((*self).into()).with(meta(None))
	}
}

impl<J: JsonClone, K: JsonFrom<J>> AsJson<J, K> for str {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		<&str as AsJson<J, K>>::as_json_with(&self, meta)
	}
}

impl<J: JsonClone, K: JsonFrom<J>> AsJson<J, K> for String {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		AsJson::<J, K>::as_json_with(self.as_str(), meta)
	}
}

impl<'a, J: JsonClone, K: JsonFrom<J>, T: AsRef<[u8]> + ?Sized> AsJson<J, K> for LanguageTag<'a, T> {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		AsJson::<J, K>::as_json_with(self.as_str(), meta)
	}
}

impl<J: JsonClone, K: JsonFrom<J>, T: AsRef<[u8]>> AsJson<J, K> for LanguageTagBuf<T> {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		AsJson::<J, K>::as_json_with(self.as_str(), meta)
	}
}

impl<J: JsonClone, K: JsonFrom<J>, T: AsJson<J, K>> AsJson<J, K> for [T] {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		let array = K::Array::from_iter(self.iter().map(|value| value.as_json_with(meta.clone())));
		Value::<K>::Array(array).with(meta(None))
	}
}

impl<J: JsonClone, K: JsonFrom<J>, T: AsJson<J, K>> AsJson<J, K> for Vec<T> {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		AsJson::<J, K>::as_json_with(self.as_slice(), meta)
	}
}

impl<J: JsonClone, K: JsonFrom<J>, T: AsJson<J, K>> AsJson<J, K> for HashSet<T> {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		let array = self.iter().map(|value| value.as_json_with(meta.clone())).collect();
		Value::<K>::Array(array).with(meta(None))
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
