use std::collections::HashMap;

use btree_indexmap::{BTreeIndexMultiSet, BTreeIndexSet};
use json_syntax::ryu_js;
use rdf_syntax::{BlankIdBuf, Generator, Id};

use crate::{
	Lenient,
	object::{ObjectMut, ObjectRef},
};

pub trait VisitJsonLd {
	fn visit_with(&self, f: &mut impl FnMut(ObjectRef));

	fn visit(&self, mut f: impl FnMut(ObjectRef)) {
		self.visit_with(&mut f);
	}

	fn visit_mut_with(&mut self, f: &mut impl FnMut(ObjectMut));

	fn visit_mut(&mut self, mut f: impl FnMut(ObjectMut)) {
		self.visit_mut_with(&mut f);
	}

	fn relabel_with(&mut self, relabeling: &mut Relabeling<impl Generator>) {
		self.visit_mut(|object| {
			relabel_object(object, relabeling);
		})
	}

	fn relabel<G: Generator>(&mut self, generator: G) -> Relabeling<G> {
		let mut relabeling = Relabeling::new(generator);
		self.relabel_with(&mut relabeling);
		relabeling
	}

	fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer) {
		self.visit_mut(|object| {
			canonicalize_object(object, buffer);
		});
	}

	fn canonicalize(&mut self) {
		let mut buffer = ryu_js::Buffer::new();
		self.canonicalize_with(&mut buffer);
	}

	fn relabel_and_canonicalize_with(
		&mut self,
		relabeling: &mut Relabeling<impl Generator>,
		buffer: &mut ryu_js::Buffer,
	) {
		self.visit_mut(|mut object| {
			relabel_object(object.reborrow(), relabeling);
			canonicalize_object(object, buffer);
		});
	}

	fn relabel_and_canonicalize<G: Generator>(&mut self, generator: G) -> Relabeling<G> {
		let mut relabeling = Relabeling::new(generator);
		let mut buffer = ryu_js::Buffer::new();
		self.relabel_and_canonicalize_with(&mut relabeling, &mut buffer);
		relabeling
	}
}

fn relabel_object(object: ObjectMut, relabeling: &mut Relabeling<impl Generator>) {
	if let ObjectMut::Node(node) = object {
		node.id = Some(relabeling.relabel(node.id.take()));
		for ty in node.types_mut() {
			*ty = relabeling.relabel(Some(ty.clone()));
		}
	}
}

fn canonicalize_object(object: ObjectMut, buffer: &mut ryu_js::Buffer) {
	if let ObjectMut::Value(value) = object {
		value.canonicalize_with(buffer);
	}
}

impl<T> VisitJsonLd for [T]
where
	T: VisitJsonLd,
{
	fn visit_with(&self, f: &mut impl FnMut(ObjectRef)) {
		for t in self {
			t.visit_with(f);
		}
	}

	fn visit_mut_with(&mut self, f: &mut impl FnMut(ObjectMut)) {
		for t in self {
			t.visit_mut_with(f);
		}
	}
}

impl<T> VisitJsonLd for BTreeIndexSet<T>
where
	T: VisitJsonLd + Ord,
{
	fn visit_with(&self, f: &mut impl FnMut(ObjectRef)) {
		for t in self {
			t.visit_with(f);
		}
	}

	fn visit_mut_with(&mut self, f: &mut impl FnMut(ObjectMut)) {
		for mut t in std::mem::take(self) {
			t.visit_mut_with(f);
			self.insert(t);
		}
	}
}

impl<T> VisitJsonLd for BTreeIndexMultiSet<T>
where
	T: VisitJsonLd + Ord,
{
	fn visit_with(&self, f: &mut impl FnMut(ObjectRef)) {
		for t in self {
			t.visit_with(f);
		}
	}

	fn visit_mut_with(&mut self, f: &mut impl FnMut(ObjectMut)) {
		for mut t in std::mem::take(self) {
			t.visit_mut_with(f);
			self.insert(t);
		}
	}
}

impl<T> VisitJsonLd for Option<T>
where
	T: VisitJsonLd,
{
	fn visit_with(&self, f: &mut impl FnMut(ObjectRef)) {
		if let Some(t) = self {
			t.visit_with(f);
		}
	}

	fn visit_mut_with(&mut self, f: &mut impl FnMut(ObjectMut)) {
		if let Some(t) = self {
			t.visit_mut_with(f);
		}
	}
}

pub struct Relabeling<G> {
	generator: G,
	map: HashMap<BlankIdBuf, Id>,
}

impl<G> Relabeling<G> {
	pub fn new(generator: G) -> Self {
		Self {
			generator,
			map: HashMap::new(),
		}
	}
}

impl<G> Relabeling<G>
where
	G: Generator,
{
	pub fn relabel(&mut self, id: Option<Lenient<Id>>) -> Lenient<Id> {
		match id {
			Some(Lenient::Valid(Id::BlankId(b))) => Lenient::Valid(
				self.map
					.entry(b)
					.or_insert_with(|| self.generator.next_id())
					.clone(),
			),
			Some(id) => id,
			None => Lenient::Valid(self.generator.next_id()),
		}
	}
}
