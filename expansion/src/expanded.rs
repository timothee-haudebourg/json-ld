use json_ld_core::{Indexed, Object};
use locspan::Meta;

pub enum Expanded<T, B, M> {
	Null,
	Object(Meta<Indexed<Object<T, B, M>>, M>),
	Array(Vec<Meta<Indexed<Object<T, B, M>>, M>>),
}

impl<T, B, M> Expanded<T, B, M> {
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

	pub fn iter(&self) -> Iter<T, B, M> {
		match self {
			Expanded::Null => Iter::Null,
			Expanded::Object(ref o) => Iter::Object(Some(o)),
			Expanded::Array(ary) => Iter::Array(ary.iter()),
		}
	}
}

impl<T, B, M> IntoIterator for Expanded<T, B, M> {
	type Item = Meta<Indexed<Object<T, B, M>>, M>;
	type IntoIter = IntoIter<T, B, M>;

	fn into_iter(self) -> IntoIter<T, B, M> {
		match self {
			Expanded::Null => IntoIter::Null,
			Expanded::Object(o) => IntoIter::Object(Some(o)),
			Expanded::Array(ary) => IntoIter::Array(ary.into_iter()),
		}
	}
}

impl<'a, T, B, M> IntoIterator for &'a Expanded<T, B, M> {
	type Item = &'a Meta<Indexed<Object<T, B, M>>, M>;
	type IntoIter = Iter<'a, T, B, M>;

	fn into_iter(self) -> Iter<'a, T, B, M> {
		self.iter()
	}
}

pub enum Iter<'a, T, B, M> {
	Null,
	Object(Option<&'a Meta<Indexed<Object<T, B, M>>, M>>),
	Array(std::slice::Iter<'a, Meta<Indexed<Object<T, B, M>>, M>>),
}

impl<'a, T, B, M> Iterator for Iter<'a, T, B, M> {
	type Item = &'a Meta<Indexed<Object<T, B, M>>, M>;

	fn next(&mut self) -> Option<&'a Meta<Indexed<Object<T, B, M>>, M>> {
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

pub enum IntoIter<T, B, M> {
	Null,
	Object(Option<Meta<Indexed<Object<T, B, M>>, M>>),
	Array(std::vec::IntoIter<Meta<Indexed<Object<T, B, M>>, M>>),
}

impl<T, B, M> Iterator for IntoIter<T, B, M> {
	type Item = Meta<Indexed<Object<T, B, M>>, M>;

	fn next(&mut self) -> Option<Meta<Indexed<Object<T, B, M>>, M>> {
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

impl<T, B, M> From<Meta<Indexed<Object<T, B, M>>, M>> for Expanded<T, B, M> {
	fn from(obj: Meta<Indexed<Object<T, B, M>>, M>) -> Expanded<T, B, M> {
		Expanded::Object(obj)
	}
}

impl<T, B, M> From<Vec<Meta<Indexed<Object<T, B, M>>, M>>> for Expanded<T, B, M> {
	fn from(list: Vec<Meta<Indexed<Object<T, B, M>>, M>>) -> Expanded<T, B, M> {
		Expanded::Array(list)
	}
}
