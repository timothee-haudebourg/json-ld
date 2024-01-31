use json_ld_core::IndexedObject;

pub enum Expanded<T, B> {
	Null,
	Object(IndexedObject<T, B>),
	Array(Vec<IndexedObject<T, B>>),
}

impl<T, B> Expanded<T, B> {
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

	pub fn iter(&self) -> Iter<T, B> {
		match self {
			Expanded::Null => Iter::Null,
			Expanded::Object(ref o) => Iter::Object(Some(o)),
			Expanded::Array(ary) => Iter::Array(ary.iter()),
		}
	}
}

impl<T, B> IntoIterator for Expanded<T, B> {
	type Item = IndexedObject<T, B>;
	type IntoIter = IntoIter<T, B>;

	fn into_iter(self) -> IntoIter<T, B> {
		match self {
			Expanded::Null => IntoIter::Null,
			Expanded::Object(o) => IntoIter::Object(Some(o)),
			Expanded::Array(ary) => IntoIter::Array(ary.into_iter()),
		}
	}
}

impl<'a, T, B> IntoIterator for &'a Expanded<T, B> {
	type Item = &'a IndexedObject<T, B>;
	type IntoIter = Iter<'a, T, B>;

	fn into_iter(self) -> Iter<'a, T, B> {
		self.iter()
	}
}

pub enum Iter<'a, T, B> {
	Null,
	Object(Option<&'a IndexedObject<T, B>>),
	Array(std::slice::Iter<'a, IndexedObject<T, B>>),
}

impl<'a, T, B> Iterator for Iter<'a, T, B> {
	type Item = &'a IndexedObject<T, B>;

	fn next(&mut self) -> Option<&'a IndexedObject<T, B>> {
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

pub enum IntoIter<T, B> {
	Null,
	Object(Option<IndexedObject<T, B>>),
	Array(std::vec::IntoIter<IndexedObject<T, B>>),
}

impl<T, B> Iterator for IntoIter<T, B> {
	type Item = IndexedObject<T, B>;

	fn next(&mut self) -> Option<IndexedObject<T, B>> {
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

impl<T, B> From<IndexedObject<T, B>> for Expanded<T, B> {
	fn from(obj: IndexedObject<T, B>) -> Expanded<T, B> {
		Expanded::Object(obj)
	}
}

impl<T, B> From<Vec<IndexedObject<T, B>>> for Expanded<T, B> {
	fn from(list: Vec<IndexedObject<T, B>>) -> Expanded<T, B> {
		Expanded::Array(list)
	}
}
