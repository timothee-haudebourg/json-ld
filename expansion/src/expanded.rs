use json_ld_core::{Id, Indexed, Object};

pub enum Expanded<T: Id, M> {
	Null,
	Object(Indexed<Object<T, M>>),
	Array(Vec<Indexed<Object<T, M>>>),
}

impl<T: Id, M> Expanded<T, M> {
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

	pub fn iter(&self) -> Iter<T, M> {
		match self {
			Expanded::Null => Iter::Null,
			Expanded::Object(ref o) => Iter::Object(Some(o)),
			Expanded::Array(ary) => Iter::Array(ary.iter()),
		}
	}
}

impl<T: Id, M> IntoIterator for Expanded<T, M> {
	type Item = Indexed<Object<T, M>>;
	type IntoIter = IntoIter<T, M>;

	fn into_iter(self) -> IntoIter<T, M> {
		match self {
			Expanded::Null => IntoIter::Null,
			Expanded::Object(o) => IntoIter::Object(Some(o)),
			Expanded::Array(ary) => IntoIter::Array(ary.into_iter()),
		}
	}
}

impl<'a, T: Id, M> IntoIterator for &'a Expanded<T, M> {
	type Item = &'a Indexed<Object<T, M>>;
	type IntoIter = Iter<'a, T, M>;

	fn into_iter(self) -> Iter<'a, T, M> {
		self.iter()
	}
}

pub enum Iter<'a, T: Id, M> {
	Null,
	Object(Option<&'a Indexed<Object<T, M>>>),
	Array(std::slice::Iter<'a, Indexed<Object<T, M>>>),
}

impl<'a, T: Id, M> Iterator for Iter<'a, T, M> {
	type Item = &'a Indexed<Object<T, M>>;

	fn next(&mut self) -> Option<&'a Indexed<Object<T, M>>> {
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

pub enum IntoIter<T: Id, M> {
	Null,
	Object(Option<Indexed<Object<T, M>>>),
	Array(std::vec::IntoIter<Indexed<Object<T, M>>>),
}

impl<T: Id, M> Iterator for IntoIter<T, M> {
	type Item = Indexed<Object<T, M>>;

	fn next(&mut self) -> Option<Indexed<Object<T, M>>> {
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

impl<T: Id, M> From<Indexed<Object<T, M>>> for Expanded<T, M> {
	fn from(obj: Indexed<Object<T, M>>) -> Expanded<T, M> {
		Expanded::Object(obj)
	}
}

impl<T: Id, M> From<Vec<Indexed<Object<T, M>>>> for Expanded<T, M> {
	fn from(list: Vec<Indexed<Object<T, M>>>) -> Expanded<T, M> {
		Expanded::Array(list)
	}
}
