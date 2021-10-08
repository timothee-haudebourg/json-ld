use crate::{Id, Indexed, Object};
use generic_json::Json;
use iref::IriBuf;

pub enum Expanded<J: Json, T: Id = IriBuf> {
	Null,
	Object(Indexed<Object<J, T>>),
	Array(Vec<Indexed<Object<J, T>>>),
}

impl<J: Json, T: Id> Expanded<J, T> {
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

	pub fn iter(&self) -> Iter<J, T> {
		match self {
			Expanded::Null => Iter::Null,
			Expanded::Object(ref o) => Iter::Object(Some(o)),
			Expanded::Array(ary) => Iter::Array(ary.iter()),
		}
	}
}

impl<J: Json, T: Id> IntoIterator for Expanded<J, T> {
	type Item = Indexed<Object<J, T>>;
	type IntoIter = IntoIter<J, T>;

	fn into_iter(self) -> IntoIter<J, T> {
		match self {
			Expanded::Null => IntoIter::Null,
			Expanded::Object(o) => IntoIter::Object(Some(o)),
			Expanded::Array(ary) => IntoIter::Array(ary.into_iter()),
		}
	}
}

impl<'a, J: Json, T: Id> IntoIterator for &'a Expanded<J, T> {
	type Item = &'a Indexed<Object<J, T>>;
	type IntoIter = Iter<'a, J, T>;

	fn into_iter(self) -> Iter<'a, J, T> {
		self.iter()
	}
}

pub enum Iter<'a, J: Json, T: Id> {
	Null,
	Object(Option<&'a Indexed<Object<J, T>>>),
	Array(std::slice::Iter<'a, Indexed<Object<J, T>>>),
}

impl<'a, J: Json, T: Id> Iterator for Iter<'a, J, T> {
	type Item = &'a Indexed<Object<J, T>>;

	fn next(&mut self) -> Option<&'a Indexed<Object<J, T>>> {
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

pub enum IntoIter<J: Json, T: Id> {
	Null,
	Object(Option<Indexed<Object<J, T>>>),
	Array(std::vec::IntoIter<Indexed<Object<J, T>>>),
}

impl<J: Json, T: Id> Iterator for IntoIter<J, T> {
	type Item = Indexed<Object<J, T>>;

	fn next(&mut self) -> Option<Indexed<Object<J, T>>> {
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

impl<J: Json, T: Id> From<Indexed<Object<J, T>>> for Expanded<J, T> {
	fn from(obj: Indexed<Object<J, T>>) -> Expanded<J, T> {
		Expanded::Object(obj)
	}
}

impl<J: Json, T: Id> From<Vec<Indexed<Object<J, T>>>> for Expanded<J, T> {
	fn from(list: Vec<Indexed<Object<J, T>>>) -> Expanded<J, T> {
		Expanded::Array(list)
	}
}
