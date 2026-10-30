use crate::IndexedJsonLdObject;

pub enum Expanded {
	Null,
	Object(IndexedJsonLdObject),
	Array(Vec<IndexedJsonLdObject>),
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
	type Item = IndexedJsonLdObject;
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
	type Item = &'a IndexedJsonLdObject;
	type IntoIter = Iter<'a>;

	fn into_iter(self) -> Iter<'a> {
		self.iter()
	}
}

pub enum Iter<'a> {
	Null,
	Object(Option<&'a IndexedJsonLdObject>),
	Array(std::slice::Iter<'a, IndexedJsonLdObject>),
}

impl<'a> Iterator for Iter<'a> {
	type Item = &'a IndexedJsonLdObject;

	fn next(&mut self) -> Option<&'a IndexedJsonLdObject> {
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
	Object(Option<IndexedJsonLdObject>),
	Array(std::vec::IntoIter<IndexedJsonLdObject>),
}

impl Iterator for IntoIter {
	type Item = IndexedJsonLdObject;

	fn next(&mut self) -> Option<IndexedJsonLdObject> {
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

impl From<IndexedJsonLdObject> for Expanded {
	fn from(obj: IndexedJsonLdObject) -> Expanded {
		Expanded::Object(obj)
	}
}

impl From<Vec<IndexedJsonLdObject>> for Expanded {
	fn from(list: Vec<IndexedJsonLdObject>) -> Expanded {
		Expanded::Array(list)
	}
}
