use crate::{Id, Indexed, Reference};
use rdf_types::BlankId;
use std::collections::HashSet;

pub trait MappedEq<T: ?Sized = Self> {
	/// Structural equality with mapped blank identifiers.
	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a BlankId) -> &'b BlankId>(
		&'a self,
		other: &T,
		f: F,
	) -> bool;
}

trait UnorderedMappedEq
where
	for<'a> &'a Self: IntoIterator<Item = &'a Self::Item>,
{
	type Item: MappedEq;

	fn len(&self) -> usize;

	fn unordered_mapped_eq<'a, 'b, F: Clone + Fn(&'a BlankId) -> &'b BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool {
		if self.len() == other.len() {
			let other_vec: Vec<_> = other.into_iter().collect();
			let mut selected = Vec::new();
			selected.resize(other_vec.len(), false);

			'self_items: for item in self {
				for (i, sel) in selected.iter_mut().enumerate() {
					if !*sel && item.mapped_eq(other_vec.get(i).unwrap(), f.clone()) {
						*sel = true;
						continue 'self_items;
					}
				}

				return false;
			}

			true
		} else {
			false
		}
	}
}

impl<'u, 't, U, T: MappedEq<U>> MappedEq<&'u U> for &'t T {
	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a BlankId) -> &'b BlankId>(
		&'a self,
		other: &&'u U,
		f: F,
	) -> bool {
		T::mapped_eq(*self, *other, f)
	}
}

impl<T: MappedEq> MappedEq for Option<T> {
	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a BlankId) -> &'b BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool {
		match (self, other) {
			(Some(a), Some(b)) => a.mapped_eq(b, f),
			(None, None) => true,
			_ => false,
		}
	}
}

impl<T: MappedEq> UnorderedMappedEq for Vec<T> {
	type Item = T;

	fn len(&self) -> usize {
		self.len()
	}
}

impl<T: MappedEq> MappedEq for Vec<T> {
	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a BlankId) -> &'b BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool {
		self.as_slice().mapped_eq(other.as_slice(), f)
	}
}

impl<T: MappedEq> UnorderedMappedEq for [T] {
	type Item = T;

	fn len(&self) -> usize {
		self.len()
	}
}

impl<T: MappedEq> MappedEq for [T] {
	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a BlankId) -> &'b BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool {
		self.len() == other.len()
			&& self
				.iter()
				.zip(other)
				.all(move |(a, b)| a.mapped_eq(b, f.clone()))
	}
}

impl<T: MappedEq> UnorderedMappedEq for HashSet<T> {
	type Item = T;

	fn len(&self) -> usize {
		self.len()
	}
}

impl<T: MappedEq> MappedEq for HashSet<T> {
	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a BlankId) -> &'b BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool {
		self.unordered_mapped_eq(other, f)
	}
}

impl<T: MappedEq> MappedEq for Indexed<T> {
	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a BlankId) -> &'b BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool {
		self.index() == other.index() && self.inner().mapped_eq(other.inner(), f)
	}
}

impl<T: PartialEq> MappedEq for Reference<T> {
	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a BlankId) -> &'b BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool {
		match (self, other) {
			(Self::Blank(a), Self::Blank(b)) => f(a) == b,
			(Self::Id(a), Self::Id(b)) => a == b,
			(Self::Invalid(a), Self::Invalid(b)) => a == b,
			_ => false,
		}
	}
}

impl<T: Id> MappedEq for super::Object<T> {
	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a BlankId) -> &'b BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool {
		match (self, other) {
			(Self::Value(a), Self::Value(b)) => a == b,
			(Self::Node(a), Self::Node(b)) => a.mapped_eq(b, f),
			(Self::List(a), Self::List(b)) => a.mapped_eq(b, f),
			_ => false,
		}
	}
}

fn opt_mapped_eq<'a, 'b, A: MappedEq, F: Clone + Fn(&'a BlankId) -> &'b BlankId>(
	a: Option<&'a A>,
	b: Option<&A>,
	f: F,
) -> bool {
	match (a, b) {
		(Some(a), Some(b)) => a.mapped_eq(b, f),
		(None, None) => true,
		_ => false,
	}
}

impl<T: Id> MappedEq for super::Node<T> {
	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a BlankId) -> &'b BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool {
		opt_mapped_eq(self.id(), other.id(), f.clone())
			&& opt_mapped_eq(self.included(), other.included(), f.clone())
			&& opt_mapped_eq(self.graph(), other.graph(), f.clone())
			&& self.properties().mapped_eq(other.properties(), f.clone())
			&& self
				.reverse_properties()
				.mapped_eq(other.reverse_properties(), f)
	}
}

impl<T: Id> MappedEq for super::node::Properties<T> {
	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a BlankId) -> &'b BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool {
		if self.len() == other.len() {
			let other_vec: Vec<_> = other.iter().collect();
			let mut selected = Vec::new();
			selected.resize(other_vec.len(), false);

			'self_items: for (prop, objects) in self {
				for (i, sel) in selected.iter_mut().enumerate() {
					let (other_prop, other_objects) = other_vec.get(i).unwrap();
					if !*sel
						&& prop.mapped_eq(other_prop, f.clone())
						&& objects.unordered_mapped_eq(other_objects, f.clone())
					{
						*sel = true;
						continue 'self_items;
					}
				}

				return false;
			}

			true
		} else {
			false
		}
	}
}

impl<T: Id> MappedEq for super::node::ReverseProperties<T> {
	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a BlankId) -> &'b BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool {
		if self.len() == other.len() {
			let other_vec: Vec<_> = other.iter().collect();
			let mut selected = Vec::new();
			selected.resize(other_vec.len(), false);

			'self_items: for (prop, nodes) in self {
				for (i, sel) in selected.iter_mut().enumerate() {
					let (other_prop, other_nodes) = other_vec.get(i).unwrap();
					if !*sel
						&& prop.mapped_eq(other_prop, f.clone())
						&& nodes.unordered_mapped_eq(other_nodes, f.clone())
					{
						*sel = true;
						continue 'self_items;
					}
				}

				return false;
			}

			true
		} else {
			false
		}
	}
}
