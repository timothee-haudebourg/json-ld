use generic_json::JsonHash;
use std::convert::TryInto;
use crate::{Id, ExpandedDocument, id};
use super::{ValidReference, Property, PropertyRef, Triple, Value, RdfDirection};
use std::borrow::Cow;

/// RDF Quad.
pub struct Quad<T: Id>(
	pub Option<ValidReference<T>>,
	pub ValidReference<T>,
	pub Property<T>,
	pub Value<T>
);

pub struct QuadRef<'a, T: Id>(
	pub Option<&'a ValidReference<T>>,
	pub Cow<'a, ValidReference<T>>,
	pub PropertyRef<'a, T>,
	pub Value<T>
);

pub struct Quads<'a, J: JsonHash + ToString, T: Id, G: id::Generator<T>> {
	generator: G,
	rdf_direction: RdfDirection,
	compound_value: Option<(Option<&'a ValidReference<T>>, super::CompoundValueTriples<'a, J, T>)>,
	quads: crate::quad::Quads<'a, J, T>
}

impl<'a, J: JsonHash + ToString, T: Id, G: id::Generator<T>> Iterator for Quads<'a, J, T, G> {
	type Item = QuadRef<'a, T>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			if let Some((graph, compound_value)) = &mut self.compound_value {
				match compound_value.next(&mut self.generator, self.rdf_direction) {
					Some(Triple(subject, property, object)) => break Some(QuadRef(
						graph.clone(),
						Cow::Owned(subject),
						property.try_into().expect("expected standard rdf property"),
						object
					)),
					None => self.compound_value = None
				}
			}
	
			match self.quads.next() {
				Some(crate::quad::QuadRef(graph, subject, property, object)) => {
					let rdf_graph: Option<&'a ValidReference<T>> = match graph.map(|r| r.try_into()) {
						Some(Ok(r)) => Some(r),
						None => None,
						_ => continue
					};

					let rdf_subject: &'a ValidReference<T> = match subject.try_into() {
						Ok(r) => r,
						Err(_) => continue
					};

					let rdf_property = match property {
						crate::quad::PropertyRef::Type => PropertyRef::Type,
						crate::quad::PropertyRef::Ref(r) => match r.try_into() {
							Ok(r) => PropertyRef::Other(r),
							Err(_) => continue
						}
					};

					if let Some((rdf_object, rdf_value_triples)) = object.rdf_value(&mut self.generator, self.rdf_direction) {
						if let Some(rdf_value_triples) = rdf_value_triples {
							self.compound_value = Some((rdf_graph, rdf_value_triples));
						}
	
						break Some(QuadRef(rdf_graph, Cow::Borrowed(rdf_subject), rdf_property, rdf_object))
					}
				},
				None => break None
			}
		}
	}
}

impl<J: JsonHash + ToString, T: Id> ExpandedDocument<J, T> {
	pub fn rdf_quads<G: id::Generator<T>>(&self, generator: G, rdf_direction: RdfDirection) -> Quads<J, T, G> {
		Quads {
			generator,
			rdf_direction,
			compound_value: None,
			quads: self.quads()
		}
	}
}