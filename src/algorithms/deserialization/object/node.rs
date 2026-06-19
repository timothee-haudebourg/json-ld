use linked_data::{
	ser::{IdSerializer, SerializeLinkedDataProperties},
	LinkedDataSerializer, RdfUnordered, SerializeLinkedData,
};
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
			_ => serializer.interpret(None)?,
		};

		if !self.types().is_empty() {
			let predicate = Term::iri(RDF_TYPE.to_owned());
			RdfUnordered(self.types()).serialize_rdf_objects(
				&mut serializer,
				&subject,
				&predicate,
				graph,
			)?;
		}

		self.properties()
			.serialize_rdf_properties(serializer.as_dyn_mut(), graph, &subject)?;

		self.reverse_properties().serialize_rdf_properties(
			serializer.as_dyn_mut(),
			graph,
			&subject,
		)?;

		self.graph
			.serialize_rdf_graph(serializer.as_dyn_mut(), Some(&subject))?;

		serializer.serialize_resource(Some(subject))?;

		self.included.serialize_rdf_graph(serializer, graph)
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
		for (id, objects) in self {
			let property = id.serialize_rdf_term(&mut serializer)?;
			RdfUnordered(objects).serialize_rdf_objects(
				serializer.as_dyn_mut(),
				subject,
				&property,
				graph,
			)?;
		}

		serializer.end()
	}
}

impl SerializeLinkedDataProperties for ReverseProperties {
	fn serialize_rdf_properties<S>(
		&self,
		mut serializer: S,
		graph: Option<&Term>,
		object: &Term,
	) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer,
	{
		for (id, subjects) in self {
			let property = id.serialize_rdf_term(&mut serializer)?;

			for subject in subjects {
				let subject = match subject
					.serialize_rdf(IdSerializer::new(serializer.as_dyn_mut()), graph)?
				{
					Some(subject) => subject,
					None => serializer.interpret(None)?,
				};

				object.serialize_rdf_objects(
					serializer.as_dyn_mut(),
					&subject,
					&property,
					graph,
				)?;
			}
		}

		serializer.end()
	}
}
