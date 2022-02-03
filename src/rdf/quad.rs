use super::{Property, PropertyRef, RdfDirection, RdfSyntax, Triple, ValidReference, Value};
use crate::{id, ExpandedDocument, FlattenedDocument, Id};
use generic_json::JsonHash;
use std::borrow::Cow;
use std::convert::TryInto;

/// RDF Quad.
pub struct Quad<T: Id>(
	pub Option<ValidReference<T>>,
	pub ValidReference<T>,
	pub Property<T>,
	pub Value<T>,
);

pub struct QuadRef<'a, T: Id>(
	pub Option<&'a ValidReference<T>>,
	pub Cow<'a, ValidReference<T>>,
	pub PropertyRef<'a, T>,
	pub Value<T>,
);

struct Compound<'a, J: JsonHash, T: Id> {
	graph: Option<&'a ValidReference<T>>,
	triples: super::CompoundValueTriples<'a, J, T>,
}

/// Iterator over the RDF Quads of a JSON-LD document.
pub struct Quads<'a, 'g, J: JsonHash + ToString, T: Id, G: id::Generator<T>> {
	generator: &'g mut G,
	rdf_direction: RdfDirection,
	compound_value: Option<Compound<'a, J, T>>,
	quads: crate::quad::Quads<'a, J, T>,
}

impl<'a, 'g, J: JsonHash + ToString, T: Id, G: id::Generator<T>> Iterator
	for Quads<'a, 'g, J, T, G>
{
	type Item = QuadRef<'a, T>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			if let Some(compound_value) = &mut self.compound_value {
				match compound_value
					.triples
					.next(self.generator, self.rdf_direction)
				{
					Some(Triple(subject, property, object)) => {
						break Some(QuadRef(
							compound_value.graph,
							Cow::Owned(subject),
							property.try_into().expect("expected standard rdf property"),
							object,
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

						break Some(QuadRef(
							rdf_graph,
							Cow::Borrowed(rdf_subject),
							rdf_property,
							compound_value.value,
						));
					}
				}
				None => break None,
			}
		}
	}
}

impl<J: JsonHash + ToString, T: Id> ExpandedDocument<J, T> {
	pub fn rdf_quads<'g, G: id::Generator<T>>(
		&self,
		generator: &'g mut G,
		rdf_direction: RdfDirection,
	) -> Quads<'_, 'g, J, T, G> {
		Quads {
			generator,
			rdf_direction,
			compound_value: None,
			quads: self.quads(),
		}
	}
}

impl<J: JsonHash + ToString, T: Id> FlattenedDocument<J, T> {
	pub fn rdf_quads<'g, G: id::Generator<T>>(
		&self,
		generator: &'g mut G,
		rdf_direction: RdfDirection,
	) -> Quads<'_, 'g, J, T, G> {
		Quads {
			generator,
			rdf_direction,
			compound_value: None,
			quads: self.quads(),
		}
	}
}
