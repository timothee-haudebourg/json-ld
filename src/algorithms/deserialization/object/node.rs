use linked_data::{
	ser::{IdSerializer, SerializeLinkedDataWith},
	LinkedDataSerializer, RdfUnordered, SerializeLinkedData,
};
use rdf_types::{Term, RDF_TYPE};

use crate::{Id, NodeObject};

use super::super::RdfSerializationOptions;

impl SerializeLinkedDataWith<RdfSerializationOptions> for NodeObject {
	fn serialize_rdf_with<S>(
		&self,
		opts: RdfSerializationOptions,
		mut serializer: S,
		graph: Option<&Term>,
	) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		let subject = match &self.id {
			Some(Id::Valid(id)) => id.clone().into(),
			Some(Id::Invalid(_)) => return serializer.end(),
			None => serializer.interpret(None)?,
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

		for (id, objects) in self.properties() {
			let property = id.serialize_rdf_term(&mut serializer)?;
			if !opts.produce_generalized_rdf && !property.is_iri() {
				continue;
			}
			for obj in objects {
				obj.serialize_rdf_objects_with(
					opts.clone(),
					serializer.as_dyn_mut(),
					&subject,
					&property,
					graph,
				)?;
			}
		}

		if let Some(reverse) = self.reverse_properties() {
			for (id, subjects) in reverse {
				let property = id.serialize_rdf_term(&mut serializer)?;
				if !opts.produce_generalized_rdf && !property.is_iri() {
					continue;
				}
				for rev_subject in subjects {
					let rev_subject = match rev_subject
						.serialize_rdf(IdSerializer::new(serializer.as_dyn_mut()), graph)?
					{
						Some(s) => s,
						None => serializer.interpret(None)?,
					};
					subject.serialize_rdf_objects(
						serializer.as_dyn_mut(),
						&rev_subject,
						&property,
						graph,
					)?;
				}
			}
		}

		self.graph.serialize_rdf_graph_with(
			opts.clone(),
			serializer.as_dyn_mut(),
			Some(&subject),
		)?;

		serializer.serialize_resource(Some(subject))?;

		self.included
			.serialize_rdf_graph_with(opts, serializer, graph)
	}
}

impl SerializeLinkedData for NodeObject {
	fn serialize_rdf<S>(&self, serializer: S, graph: Option<&Term>) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		self.serialize_rdf_with(RdfSerializationOptions::default(), serializer, graph)
	}
}
