use super::{RdfDirection, ValidReference, Value};
use crate::namespace::no_namespace_mut;
use crate::IriNamespaceMut;
use crate::{flattening::NodeMap, id, ExpandedDocument, FlattenedDocument};
use rdf_types::Triple;
use std::borrow::Cow;
use std::convert::TryInto;
use std::hash::Hash;

pub type Quad<T, B> =
	rdf_types::Quad<ValidReference<T, B>, ValidReference<T, B>, Value<T, B>, ValidReference<T, B>>;

pub type QuadRef<'a, T, B> = rdf_types::Quad<
	Cow<'a, ValidReference<T, B>>,
	Cow<'a, ValidReference<T, B>>,
	Value<T, B>,
	&'a ValidReference<T, B>,
>;

struct Compound<'a, T, B, M> {
	graph: Option<&'a ValidReference<T, B>>,
	triples: super::CompoundValueTriples<'a, T, B, M>,
}

/// Iterator over the RDF Quads of a JSON-LD document.
pub struct Quads<'a, 'n, 'g, T, B, N, M, G: id::Generator<T, B, M, N>> {
	namespace: &'n mut N,
	generator: &'g mut G,
	rdf_direction: Option<RdfDirection>,
	compound_value: Option<Compound<'a, T, B, M>>,
	quads: crate::quad::Quads<'a, T, B, M>,
}

impl<'a, 'n, 'g, T: Clone, B: Clone, N: IriNamespaceMut<T>, M, G: id::Generator<T, B, M, N>>
	Iterator for Quads<'a, 'n, 'g, T, B, N, M, G>
{
	type Item = QuadRef<'a, T, B>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			if let Some(compound_value) = &mut self.compound_value {
				match compound_value.triples.next(
					self.namespace,
					self.generator,
					self.rdf_direction,
				) {
					Some(Triple(subject, property, object)) => {
						break Some(rdf_types::Quad(
							Cow::Owned(subject),
							Cow::Owned(property),
							object,
							compound_value.graph,
						))
					}
					None => self.compound_value = None,
				}
			}

			match self.quads.next() {
				Some(crate::quad::QuadRef(graph, subject, property, object)) => {
					let rdf_graph: Option<&'a ValidReference<T, B>> =
						match graph.map(|r| r.try_into()) {
							Some(Ok(r)) => Some(r),
							None => None,
							_ => continue,
						};

					let rdf_subject: &'a ValidReference<T, B> = match subject.try_into() {
						Ok(r) => r,
						Err(_) => continue,
					};

					let rdf_property = match property {
						crate::quad::PropertyRef::Type => {
							Cow::Owned(ValidReference::Id(self.namespace.insert(super::RDF_TYPE)))
						}
						crate::quad::PropertyRef::Ref(r) => match r.try_into() {
							Ok(r) => Cow::Borrowed(r),
							Err(_) => continue,
						},
					};

					if let Some(compound_value) =
						object.rdf_value_in(self.namespace, self.generator, self.rdf_direction)
					{
						if let Some(rdf_value_triples) = compound_value.triples {
							self.compound_value = Some(Compound {
								graph: rdf_graph,
								triples: rdf_value_triples,
							});
						}

						break Some(rdf_types::Quad(
							Cow::Borrowed(rdf_subject),
							rdf_property,
							compound_value.value,
							rdf_graph,
						));
					}
				}
				None => break None,
			}
		}
	}
}

impl<T, B, M> ExpandedDocument<T, B, M> {
	pub fn rdf_quads_in<'n, 'g, N, G: id::Generator<T, B, M, N>>(
		&self,
		namespace: &'n mut N,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Quads<'_, 'n, 'g, T, B, N, M, G> {
		Quads {
			namespace,
			generator,
			rdf_direction,
			compound_value: None,
			quads: self.quads(),
		}
	}

	pub fn rdf_quads<'g, G: id::Generator<T, B, M, ()>>(
		&self,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Quads<'_, 'static, 'g, T, B, (), M, G> {
		Quads {
			namespace: no_namespace_mut(),
			generator,
			rdf_direction,
			compound_value: None,
			quads: self.quads(),
		}
	}
}

impl<T, B, M> FlattenedDocument<T, B, M> {
	pub fn rdf_quads_in<'n, 'g, N, G: id::Generator<T, B, M, N>>(
		&self,
		namespace: &'n mut N,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Quads<'_, 'n, 'g, T, B, N, M, G> {
		Quads {
			namespace,
			generator,
			rdf_direction,
			compound_value: None,
			quads: self.quads(),
		}
	}

	pub fn rdf_quads<'g, G: id::Generator<T, B, M, ()>>(
		&self,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Quads<'_, 'static, 'g, T, B, (), M, G> {
		Quads {
			namespace: no_namespace_mut(),
			generator,
			rdf_direction,
			compound_value: None,
			quads: self.quads(),
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> NodeMap<T, B, M> {
	pub fn rdf_quads_in<'n, 'g, N, G: id::Generator<T, B, M, N>>(
		&self,
		namespace: &'n mut N,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Quads<'_, 'n, 'g, T, B, N, M, G> {
		Quads {
			namespace,
			generator,
			rdf_direction,
			compound_value: None,
			quads: self.quads(),
		}
	}

	pub fn rdf_quads<'g, G: id::Generator<T, B, M, ()>>(
		&self,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Quads<'_, 'static, 'g, T, B, (), M, G> {
		Quads {
			namespace: no_namespace_mut(),
			generator,
			rdf_direction,
			compound_value: None,
			quads: self.quads(),
		}
	}
}
