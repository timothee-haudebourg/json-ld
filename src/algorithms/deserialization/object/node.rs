use linked_data::{ser::SerializeLinkedDataProperties, LinkedDataSerializer, SerializeLinkedData};
use rdf_types::{Term, RDF_TYPE};

use crate::{
	object::node::{Properties, ReverseProperties},
	Id, NodeObject,
};

impl SerializeLinkedData for NodeObject {
	fn serialize_rdf<S>(&self, mut serializer: S, graph: Option<&Term>) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		let subject = match &self.id {
			Some(Id::Valid(id)) => id.clone().into(),
			_ => serializer.new_resource()?,
		};

		if !self.types().is_empty() {
			let predicate = Term::iri(RDF_TYPE.to_owned());
			self.types()
				.serialize_rdf_objects(&mut serializer, graph, &subject, &predicate)?;
		}

		self.properties()
			.serialize_rdf_properties(&mut serializer, graph, &subject)?;

		self.reverse_properties().serialize_rdf_reverse_properties(
			&mut serializer,
			graph,
			&subject,
		)?;

		self.graph.serialize_rdf(&mut serializer, Some(&subject))?;

		serializer.serialize_resource(subject)?;

		self.included.serialize_rdf(serializer, graph)
	}
}

impl SerializeLinkedDataProperties for Properties {
	fn serialize_rdf_properties<S>(
		&self,
		mut serializer: S,
		graph: Option<&Term>,
		subject: &Term,
	) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer,
	{
		for (id, object) in self {
			let property = id.serialize_rdf_term(&mut serializer)?;
			object.serialize_rdf_objects(&mut serializer, graph, subject, &property)?;
		}

		serializer.end()
	}
}

impl SerializeLinkedDataProperties for ReverseProperties {
	fn serialize_rdf_properties<S>(
		&self,
		mut serializer: S,
		graph: Option<&Term>,
		subject: &Term,
	) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer,
	{
		for (id, object) in self {
			let property = id.serialize_rdf_term(&mut serializer)?;
			object.serialize_rdf_objects(&mut serializer, graph, subject, &property)?;
		}

		serializer.end()
	}
}
