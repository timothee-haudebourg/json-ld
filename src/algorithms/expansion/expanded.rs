use crate::JsonLdIndexedObject;

pub enum Expanded {
	Null,
	Object(JsonLdIndexedObject),
	Array(Vec<JsonLdIndexedObject>),
}

impl Expanded {
	pub fn len(&self) -> usize {
		match self {
			Expanded::Null => 0,
			Expanded::Object(_) => 1,
			Expanded::Array(ary) => ary.len(),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn is_null(&self) -> bool {
		matches!(self, Expanded::Null)
	}

	pub fn is_list(&self) -> bool {
		match self {
			Expanded::Object(o) => o.is_list(),
			_ => false,
		}
	}

	pub fn iter(&self) -> Iter<'_> {
		match self {
			Expanded::Null => Iter::Null,
			Expanded::Object(o) => Iter::Object(Some(o)),
			Expanded::Array(ary) => Iter::Array(ary.iter()),
		}
	}
}

impl IntoIterator for Expanded {
	type Item = JsonLdIndexedObject;
	type IntoIter = IntoIter;

	fn into_iter(self) -> IntoIter {
		match self {
			Expanded::Null => IntoIter::Null,
			Expanded::Object(o) => IntoIter::Object(Some(o)),
			Expanded::Array(ary) => IntoIter::Array(ary.into_iter()),
		}
	}
}

impl<'a> IntoIterator for &'a Expanded {
	type Item = &'a JsonLdIndexedObject;
	type IntoIter = Iter<'a>;

	fn into_iter(self) -> Iter<'a> {
		self.iter()
	}
}

pub enum Iter<'a> {
	Null,
	Object(Option<&'a JsonLdIndexedObject>),
	Array(std::slice::Iter<'a, JsonLdIndexedObject>),
}

impl<'a> Iterator for Iter<'a> {
	type Item = &'a JsonLdIndexedObject;

	fn next(&mut self) -> Option<&'a JsonLdIndexedObject> {
		match self {
			Iter::Null => None,
			Iter::Object(o) => {
				let mut result = None;
				std::mem::swap(o, &mut result);
				result
			}
			Iter::Array(it) => it.next(),
		}
	}
}

pub enum IntoIter {
	Null,
	Object(Option<JsonLdIndexedObject>),
	Array(std::vec::IntoIter<JsonLdIndexedObject>),
}

impl Iterator for IntoIter {
	type Item = JsonLdIndexedObject;

	fn next(&mut self) -> Option<JsonLdIndexedObject> {
		match self {
			IntoIter::Null => None,
			IntoIter::Object(o) => {
				let mut result = None;
				std::mem::swap(o, &mut result);
				result
			}
			IntoIter::Array(it) => it.next(),
		}
	}
}

impl From<JsonLdIndexedObject> for Expanded {
	fn from(obj: JsonLdIndexedObject) -> Expanded {
		Expanded::Object(obj)
	}
}

impl From<Vec<JsonLdIndexedObject>> for Expanded {
	fn from(list: Vec<JsonLdIndexedObject>) -> Expanded {
		Expanded::Array(list)
	}
}
