use crate::{Id, Object};

pub enum Expanded<T: Id> {
	Null,
	Object(Object<T>),
	Array(Vec<Object<T>>)
}

impl<T: Id> Expanded<T> {
	pub fn len(&self) -> usize {
		match self {
			Expanded::Null => 0,
			Expanded::Object(_) => 1,
			Expanded::Array(ary) => ary.len()
		}
	}

	pub fn is_null(&self) -> bool {
		match self {
			Expanded::Null => true,
			_ => false
		}
	}

	pub fn is_list(&self) -> bool {
		match self {
			Expanded::Object(o) => o.is_list(),
			_ => false
		}
	}

	pub fn iter(&self) -> Iter<T> {
		match self {
			Expanded::Null => Iter::Null,
			Expanded::Object(ref o) => Iter::Object(Some(o)),
			Expanded::Array(ary) => Iter::Array(ary.iter())
		}
	}
}

impl<T: Id> IntoIterator for Expanded<T> {
	type Item = Object<T>;
	type IntoIter = IntoIter<T>;

	fn into_iter(self) -> IntoIter<T> {
		match self {
			Expanded::Null => IntoIter::Null,
			Expanded::Object(o) => IntoIter::Object(Some(o)),
			Expanded::Array(ary) => IntoIter::Array(ary.into_iter())
		}
	}
}

impl<'a, T: Id> IntoIterator for &'a Expanded<T> {
	type Item = &'a Object<T>;
	type IntoIter = Iter<'a, T>;

	fn into_iter(self) -> Iter<'a, T> {
		self.iter()
	}
}

pub enum Iter<'a, T: Id> {
	Null,
	Object(Option<&'a Object<T>>),
	Array(std::slice::Iter<'a, Object<T>>)
}

impl<'a, T: Id> Iterator for Iter<'a, T> {
	type Item = &'a Object<T>;

	fn next(&mut self) -> Option<&'a Object<T>> {
		match self {
			Iter::Null => None,
			Iter::Object(ref mut o) => {
				let mut result = None;
				std::mem::swap(o, &mut result);
				result
			},
			Iter::Array(ref mut it) => {
				it.next()
			}
		}
	}
}

pub enum IntoIter<T: Id> {
	Null,
	Object(Option<Object<T>>),
	Array(std::vec::IntoIter<Object<T>>)
}

impl<T: Id> Iterator for IntoIter<T> {
	type Item = Object<T>;

	fn next(&mut self) -> Option<Object<T>> {
		match self {
			IntoIter::Null => None,
			IntoIter::Object(ref mut o) => {
				let mut result = None;
				std::mem::swap(o, &mut result);
				result
			},
			IntoIter::Array(ref mut it) => {
				it.next()
			},
		}
	}
}
