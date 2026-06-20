use linked_data::{ser::SerializeLinkedDataWith, LinkedDataSerializer, SerializeLinkedData};
use rdf_syntax::{Id, Term};

use crate::{ExpandedDocument, Lenient, Validate};

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

impl Lenient<Id> {
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

impl<T, Q> SerializeLinkedDataWith<Q> for Lenient<T>
where
	T: Validate + SerializeLinkedDataWith<Q>,
{
	fn serialize_rdf_with<S>(
		&self,
		state: Q,
		serializer: S,
		graph: Option<&Term>,
	) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		match self {
			Self::Valid(t) => t.serialize_rdf_with(state, serializer, graph),
			Self::Invalid(_) => serializer.end(),
		}
	}
}

impl<T> SerializeLinkedData for Lenient<T>
where
	T: Validate + SerializeLinkedData,
{
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
