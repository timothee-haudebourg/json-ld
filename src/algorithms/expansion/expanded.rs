use crate::IndexedObject;

pub enum Expanded {
	Null,
	Object(IndexedObject),
	Array(Vec<IndexedObject>),
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

	pub fn iter(&self) -> Iter {
		match self {
			Expanded::Null => Iter::Null,
			Expanded::Object(ref o) => Iter::Object(Some(o)),
			Expanded::Array(ary) => Iter::Array(ary.iter()),
		}
	}
}

impl IntoIterator for Expanded {
	type Item = IndexedObject;
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
	type Item = &'a IndexedObject;
	type IntoIter = Iter<'a>;

	fn into_iter(self) -> Iter<'a> {
		self.iter()
	}
}

pub enum Iter<'a> {
	Null,
	Object(Option<&'a IndexedObject>),
	Array(std::slice::Iter<'a, IndexedObject>),
}

impl<'a> Iterator for Iter<'a> {
	type Item = &'a IndexedObject;

	fn next(&mut self) -> Option<&'a IndexedObject> {
		match self {
			Iter::Null => None,
			Iter::Object(ref mut o) => {
				let mut result = None;
				std::mem::swap(o, &mut result);
				result
			}
			Iter::Array(ref mut it) => it.next(),
		}
	}
}

pub enum IntoIter {
	Null,
	Object(Option<IndexedObject>),
	Array(std::vec::IntoIter<IndexedObject>),
}

impl Iterator for IntoIter {
	type Item = IndexedObject;

	fn next(&mut self) -> Option<IndexedObject> {
		match self {
			IntoIter::Null => None,
			IntoIter::Object(ref mut o) => {
				let mut result = None;
				std::mem::swap(o, &mut result);
				result
			}
			IntoIter::Array(ref mut it) => it.next(),
		}
	}
}

impl From<IndexedObject> for Expanded {
	fn from(obj: IndexedObject) -> Expanded {
		Expanded::Object(obj)
	}
}

impl From<Vec<IndexedObject>> for Expanded {
	fn from(list: Vec<IndexedObject>) -> Expanded {
		Expanded::Array(list)
	}
}
