use crate::{Id, Indexed, ValidId};
use indexmap::IndexSet;
use std::hash::Hash;

pub trait MappedEq<T: ?Sized = Self> {
	type BlankId;
	/// Structural equality with mapped blank identifiers.
	///
	/// Does not care for metadata.
	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a Self::BlankId) -> &'b Self::BlankId>(
		&'a self,
		other: &T,
		f: F,
	) -> bool
	where
		Self::BlankId: 'a + 'b;
}

trait UnorderedMappedEq
where
	for<'a> &'a Self: IntoIterator<Item = &'a Self::Item>,
{
	type Item: MappedEq;

	fn len(&self) -> usize;

	fn unordered_mapped_eq<
		'a,
		'b,
		F: Clone + Fn(&'a <Self::Item as MappedEq>::BlankId) -> &'b <Self::Item as MappedEq>::BlankId,
	>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool
	where
		<Self::Item as MappedEq>::BlankId: 'a + 'b,
	{
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

impl<'u, U, T: MappedEq<U>> MappedEq<&'u U> for &T {
	type BlankId = T::BlankId;

	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a Self::BlankId) -> &'b Self::BlankId>(
		&'a self,
		other: &&'u U,
		f: F,
	) -> bool
	where
		Self::BlankId: 'a + 'b,
	{
		T::mapped_eq(*self, *other, f)
	}
}

impl<T: MappedEq> MappedEq for Option<T> {
	type BlankId = T::BlankId;

	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a Self::BlankId) -> &'b Self::BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool
	where
		Self::BlankId: 'a + 'b,
	{
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
	type BlankId = T::BlankId;

	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a Self::BlankId) -> &'b Self::BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool
	where
		Self::BlankId: 'a + 'b,
	{
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
	type BlankId = T::BlankId;

	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a Self::BlankId) -> &'b Self::BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool
	where
		Self::BlankId: 'a + 'b,
	{
		self.len() == other.len()
			&& self
				.iter()
				.zip(other)
				.all(move |(a, b)| a.mapped_eq(b, f.clone()))
	}
}

impl<T: MappedEq> UnorderedMappedEq for IndexSet<T> {
	type Item = T;

	fn len(&self) -> usize {
		self.len()
	}
}

impl<T: MappedEq> MappedEq for IndexSet<T> {
	type BlankId = T::BlankId;

	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a Self::BlankId) -> &'b Self::BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool
	where
		Self::BlankId: 'a + 'b,
	{
		self.unordered_mapped_eq(other, f)
	}
}

impl<T: MappedEq> MappedEq for Indexed<T> {
	type BlankId = T::BlankId;

	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a Self::BlankId) -> &'b Self::BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool
	where
		Self::BlankId: 'a + 'b,
	{
		self.index() == other.index() && self.inner().mapped_eq(other.inner(), f)
	}
}

impl<T: PartialEq, B: PartialEq> MappedEq for Id<T, B> {
	type BlankId = B;

	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a Self::BlankId) -> &'b Self::BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool
	where
		Self::BlankId: 'a + 'b,
	{
		match (self, other) {
			(Self::Valid(a), Self::Valid(b)) => a.mapped_eq(b, f),
			(Self::Invalid(a), Self::Invalid(b)) => a == b,
			_ => false,
		}
	}
}

impl<T: PartialEq, B: PartialEq> MappedEq for ValidId<T, B> {
	type BlankId = B;

	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a Self::BlankId) -> &'b Self::BlankId>(
		&'a self,
		other: &Self,
		f: F,
	) -> bool
	where
		Self::BlankId: 'a + 'b,
	{
		match (self, other) {
			(Self::Blank(a), Self::Blank(b)) => f(a) == b,
			(Self::Iri(a), Self::Iri(b)) => a == b,
			_ => false,
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash> MappedEq for super::Object<T, B> {
	type BlankId = B;

	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a B) -> &'b B>(&'a self, other: &Self, f: F) -> bool
	where
		B: 'a + 'b,
	{
		match (self, other) {
			(Self::Value(a), Self::Value(b)) => a == b,
			(Self::Node(a), Self::Node(b)) => a.mapped_eq(b, f),
			(Self::List(a), Self::List(b)) => a.mapped_eq(b, f),
			_ => false,
		}
	}
}

fn opt_mapped_eq<'a, 'b, A: MappedEq, F: Clone + Fn(&'a A::BlankId) -> &'b A::BlankId>(
	a: Option<&'a A>,
	b: Option<&A>,
	f: F,
) -> bool
where
	A::BlankId: 'a + 'b,
{
	match (a, b) {
		(Some(a), Some(b)) => a.mapped_eq(b, f),
		(None, None) => true,
		_ => false,
	}
}

impl<T: Eq + Hash, B: Eq + Hash> MappedEq for super::Node<T, B> {
	type BlankId = B;

	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a B) -> &'b B>(&'a self, other: &Self, f: F) -> bool
	where
		B: 'a + 'b,
	{
		opt_mapped_eq(self.id.as_ref(), other.id.as_ref(), f.clone())
			&& opt_mapped_eq(self.included_entry(), other.included_entry(), f.clone())
			&& opt_mapped_eq(self.graph_entry(), other.graph_entry(), f.clone())
			&& self.properties().mapped_eq(other.properties(), f.clone())
			&& opt_mapped_eq(
				self.reverse_properties_entry(),
				other.reverse_properties_entry(),
				f,
			)
	}
}

impl<T: Eq + Hash, B: Eq + Hash> MappedEq for super::node::Properties<T, B> {
	type BlankId = B;

	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a B) -> &'b B>(&'a self, other: &Self, f: F) -> bool
	where
		B: 'a + 'b,
	{
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

impl<T: Eq + Hash, B: Eq + Hash> MappedEq for super::node::ReverseProperties<T, B> {
	type BlankId = B;

	fn mapped_eq<'a, 'b, F: Clone + Fn(&'a B) -> &'b B>(&'a self, other: &Self, f: F) -> bool
	where
		B: 'a + 'b,
	{
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
