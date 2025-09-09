use linked_data::SerializeLinkedData;
use rdf_types::{Term, RDF_FIRST, RDF_NIL, RDF_REST};

use crate::{object::ListObject, IndexedObject};

impl SerializeLinkedData for ListObject {
	fn serialize_rdf<S>(
		&self,
		serializer: S,
		graph: Option<&rdf_types::Term>,
	) -> Result<S::Ok, S::Error>
	where
		S: linked_data::LinkedDataSerializer<rdf_types::Term>,
	{
		Rest(self.as_slice()).serialize_rdf(serializer, graph)
	}
}

struct Rest<'a>(&'a [IndexedObject]);

impl SerializeLinkedData for Rest<'_> {
	fn serialize_rdf<S>(
		&self,
		mut serializer: S,
		graph: Option<&rdf_types::Term>,
	) -> Result<S::Ok, S::Error>
	where
		S: linked_data::LinkedDataSerializer<rdf_types::Term>,
	{
		let node = match self.0.split_first() {
			Some((first, rest)) => {
				let subject = serializer.new_resource()?;
				let predicate = Term::iri(RDF_FIRST.to_owned());
				first.serialize_rdf_objects(&mut serializer, graph, &subject, &predicate)?;
				let predicate = Term::iri(RDF_REST.to_owned());
				Rest(rest).serialize_rdf_objects(&mut serializer, graph, &subject, &predicate)?;
				subject
			}
			None => Term::iri(RDF_NIL.to_owned()),
		};

		serializer.serialize_resource(node)?;
		serializer.end()
	}
}
