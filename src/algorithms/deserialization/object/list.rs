use linked_data::{ser::SerializeLinkedDataWith, LinkedDataSerializer, SerializeLinkedData};
use rdf_types::{Term, RDF_FIRST, RDF_NIL, RDF_REST};

use crate::{object::ListObject, IndexedObject};

use super::super::RdfSerializationOptions;

impl SerializeLinkedDataWith<RdfSerializationOptions> for ListObject {
	fn serialize_rdf_with<S>(
		&self,
		opts: RdfSerializationOptions,
		serializer: S,
		graph: Option<&Term>,
	) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		RestWith(self.as_slice(), opts).serialize_rdf(serializer, graph)
	}
}

impl SerializeLinkedData for ListObject {
	fn serialize_rdf<S>(&self, serializer: S, graph: Option<&Term>) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		self.serialize_rdf_with(RdfSerializationOptions::default(), serializer, graph)
	}
}

struct RestWith<'a>(&'a [IndexedObject], RdfSerializationOptions);

impl SerializeLinkedData for RestWith<'_> {
	fn serialize_rdf<S>(&self, mut serializer: S, graph: Option<&Term>) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		let node = match self.0.split_first() {
			Some((first, rest)) => {
				let subject = serializer.interpret(None)?;
				let predicate = Term::iri(RDF_FIRST.to_owned());
				first.serialize_rdf_objects_with(
					self.1.clone(),
					serializer.as_dyn_mut(),
					&subject,
					&predicate,
					graph,
				)?;
				let predicate = Term::iri(RDF_REST.to_owned());
				RestWith(rest, self.1.clone()).serialize_rdf_objects(
					serializer.as_dyn_mut(),
					&subject,
					&predicate,
					graph,
				)?;
				subject
			}
			None => Term::iri(RDF_NIL.to_owned()),
		};

		serializer.serialize_resource(Some(node))?;
		serializer.end()
	}
}
