use super::{Property, PropertyRef, RdfDirection, RdfSyntax, ValidReference, Value};
use crate::{flattening::NodeMap, id, ExpandedDocument, FlattenedDocument, Id};
use rdf_types::Triple;
use std::borrow::Cow;
use std::convert::TryInto;

pub type Quad<T> = rdf_types::Quad<ValidReference<T>, Property<T>, Value<T>, ValidReference<T>>;

pub type QuadRef<'a, T> = rdf_types::Quad<
	Cow<'a, ValidReference<T>>,
	PropertyRef<'a, T>,
	Value<T>,
	&'a ValidReference<T>,
>;

struct Compound<'a, T: Id, M> {
	graph: Option<&'a ValidReference<T>>,
	triples: super::CompoundValueTriples<'a, T, M>,
}

/// Iterator over the RDF Quads of a JSON-LD document.
pub struct Quads<'a, 'g, T: Id, M, G: id::Generator<T>> {
	generator: &'g mut G,
	rdf_direction: Option<RdfDirection>,
	compound_value: Option<Compound<'a, T, M>>,
	quads: crate::quad::Quads<'a, T, M>,
}

impl<'a, 'g, T: Id, M, G: id::Generator<T>> Iterator for Quads<'a, 'g, T, M, G> {
	type Item = QuadRef<'a, T>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			if let Some(compound_value) = &mut self.compound_value {
				match compound_value
					.triples
					.next(self.generator, self.rdf_direction)
				{
					Some(Triple(subject, property, object)) => {
						break Some(rdf_types::Quad(
							Cow::Owned(subject),
							property
								.try_into()
								.ok()
								.expect("expected standard rdf property"),
							object,
							compound_value.graph,
						))
					}
					None => self.compound_value = None,
				}
			}

			match self.quads.next() {
				Some(crate::quad::QuadRef(graph, subject, property, object)) => {
					let rdf_graph: Option<&'a ValidReference<T>> = match graph.map(|r| r.try_into())
					{
						Some(Ok(r)) => Some(r),
						None => None,
						_ => continue,
					};

					let rdf_subject: &'a ValidReference<T> = match subject.try_into() {
						Ok(r) => r,
						Err(_) => continue,
					};

					let rdf_property = match property {
						crate::quad::PropertyRef::Type => PropertyRef::Rdf(RdfSyntax::Type),
						crate::quad::PropertyRef::Ref(r) => match r.try_into() {
							Ok(r) => PropertyRef::Other(r),
							Err(_) => continue,
						},
					};

					if let Some(compound_value) =
						object.rdf_value(self.generator, self.rdf_direction)
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

impl<T: Id, M> ExpandedDocument<T, M> {
	pub fn rdf_quads<'g, G: id::Generator<T>>(
		&self,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Quads<'_, 'g, T, M, G> {
		Quads {
			generator,
			rdf_direction,
			compound_value: None,
			quads: self.quads(),
		}
	}
}

impl<T: Id, M> FlattenedDocument<T, M> {
	pub fn rdf_quads<'g, G: id::Generator<T>>(
		&self,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Quads<'_, 'g, T, M, G> {
		Quads {
			generator,
			rdf_direction,
			compound_value: None,
			quads: self.quads(),
		}
	}
}

impl<T: Id, M> NodeMap<T, M> {
	pub fn rdf_quads<'g, G: id::Generator<T>>(
		&self,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Quads<'_, 'g, T, M, G> {
		Quads {
			generator,
			rdf_direction,
			compound_value: None,
			quads: self.quads(),
		}
	}
}
