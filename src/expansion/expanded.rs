use crate::{Id, Indexed, Object};
use iref::IriBuf;

pub enum Expanded<T: Id = IriBuf> {
	Null,
	Object(Indexed<Object<T>>),
	Array(Vec<Indexed<Object<T>>>),
}

impl<T: Id> Expanded<T> {
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

	pub fn iter(&self) -> Iter<T> {
		match self {
			Expanded::Null => Iter::Null,
			Expanded::Object(ref o) => Iter::Object(Some(o)),
			Expanded::Array(ary) => Iter::Array(ary.iter()),
		}
	}
}

impl<T: Id> IntoIterator for Expanded<T> {
	type Item = Indexed<Object<T>>;
	type IntoIter = IntoIter<T>;

	fn into_iter(self) -> IntoIter<T> {
		match self {
			Expanded::Null => IntoIter::Null,
			Expanded::Object(o) => IntoIter::Object(Some(o)),
			Expanded::Array(ary) => IntoIter::Array(ary.into_iter()),
		}
	}
}

impl<'a, T: Id> IntoIterator for &'a Expanded<T> {
	type Item = &'a Indexed<Object<T>>;
	type IntoIter = Iter<'a, T>;

	fn into_iter(self) -> Iter<'a, T> {
		self.iter()
	}
}

pub enum Iter<'a, T: Id> {
	Null,
	Object(Option<&'a Indexed<Object<T>>>),
	Array(std::slice::Iter<'a, Indexed<Object<T>>>),
}

impl<'a, T: Id> Iterator for Iter<'a, T> {
	type Item = &'a Indexed<Object<T>>;

	fn next(&mut self) -> Option<&'a Indexed<Object<T>>> {
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

pub enum IntoIter<T: Id> {
	Null,
	Object(Option<Indexed<Object<T>>>),
	Array(std::vec::IntoIter<Indexed<Object<T>>>),
}

impl<T: Id> Iterator for IntoIter<T> {
	type Item = Indexed<Object<T>>;

	fn next(&mut self) -> Option<Indexed<Object<T>>> {
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

impl<T: Id> From<Indexed<Object<T>>> for Expanded<T> {
	fn from(obj: Indexed<Object<T>>) -> Expanded<T> {
		Expanded::Object(obj)
	}
}

impl<T: Id> From<Vec<Indexed<Object<T>>>> for Expanded<T> {
	fn from(list: Vec<Indexed<Object<T>>>) -> Expanded<T> {
		Expanded::Array(list)
	}
}
