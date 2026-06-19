use linked_data::{ser::SerializeLinkedDataWith, LinkedDataSerializer, SerializeLinkedData};
use rdf_types::Term;

use crate::{ExpandedDocument, Id};

mod object;

/// Options for serializing an expanded JSON-LD document to RDF.
#[derive(Clone, Default)]
pub struct RdfSerializationOptions {
	/// Allow predicates to be arbitrary terms (generalized RDF).
	///
	/// When `false` (the default), triples whose predicate is not a well-formed
	/// IRI are silently skipped, producing only well-formed RDF output.
	/// When `true`, any predicate term is accepted (generalized RDF mode).
	pub produce_generalized_rdf: bool,
}

impl SerializeLinkedDataWith<RdfSerializationOptions> for ExpandedDocument {
	fn serialize_rdf_with<S>(
		&self,
		opts: RdfSerializationOptions,
		mut serializer: S,
		graph: Option<&Term>,
	) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		for object in self {
			object.serialize_rdf_with(opts.clone(), serializer.as_dyn_mut(), graph)?;
		}

		serializer.end()
	}
}

impl SerializeLinkedData for ExpandedDocument {
	fn serialize_rdf<S>(&self, serializer: S, graph: Option<&Term>) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		self.serialize_rdf_with(RdfSerializationOptions::default(), serializer, graph)
	}
}

impl Id {
	pub fn serialize_rdf_term<S>(&self, serializer: &mut S) -> Result<Term, S::Error>
	where
		S: LinkedDataSerializer,
	{
		match self {
			Self::Valid(id) => Ok(id.clone().into()),
			Self::Invalid(_) => serializer.interpret(None),
		}
	}
}

impl SerializeLinkedDataWith<RdfSerializationOptions> for Id {
	fn serialize_rdf_with<S>(
		&self,
		_opts: RdfSerializationOptions,
		serializer: S,
		graph: Option<&Term>,
	) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		// Options are irrelevant for Id serialization.
		self.serialize_rdf(serializer, graph)
	}
}

impl SerializeLinkedData for Id {
	fn serialize_rdf<S>(&self, serializer: S, graph: Option<&Term>) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		match self {
			Self::Valid(id) => id.serialize_rdf(serializer, graph),
			Self::Invalid(_) => serializer.end(),
		}
	}
}
