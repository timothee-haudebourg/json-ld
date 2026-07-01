use linked_data::LinkedDataDeserializer;
use rdf_syntax::{CowGroundTerm, CowTerm, Quad, RDF_FIRST, RDF_LIST, RDF_NIL, RDF_REST, RDF_TYPE};

use crate::{Indexed, algorithms::serialization::object::deserialize_object, object::ListObject};

pub fn try_deserialize_list_object<R, D>(
	deserializer: &mut D,
	subject: &R,
	graph: Option<&R>,
) -> Result<Option<ListObject>, D::Error>
where
	R: Clone + PartialEq,
	D: LinkedDataDeserializer<R>,
{
	// Validate the entire list chain before consuming any quads.
	if !is_list_object(deserializer, subject, graph)? {
		return Ok(None);
	}

	// The subject is a valid RDF list. Walk the chain and collect elements.
	let mut items = Vec::new();
	let mut head = subject.clone();

	while !deserializer.is_iri(&head, RDF_NIL)? {
		let mut first = None;
		let mut rest = None;

		while let Some(Quad(_, predicate, object, _)) =
			deserializer.deserialize_quad(Quad(Some(&head), None, None, Some(graph)))?
		{
			for p_term in deserializer.terms_of(&predicate) {
				if let CowTerm::Ground(CowGroundTerm::Iri(p_iri)) = p_term? {
					if *p_iri == RDF_FIRST {
						first = Some(object.clone());
					}

					if *p_iri == RDF_REST {
						rest = Some(object.clone());
					}
				}
			}
		}

		items.push(Indexed::unindexed(deserialize_object(
			deserializer,
			&first.unwrap(),
			graph,
		)?));

		head = rest.unwrap();
	}

	Ok(Some(ListObject::new(items)))
}

fn is_list_object<R, D>(deserializer: &D, head: &R, graph: Option<&R>) -> Result<bool, D::Error>
where
	R: ToOwned + PartialEq,
	D: LinkedDataDeserializer<R>,
{
	// Check if head is rdf:nil (list terminator).
	if deserializer.is_iri(head, RDF_NIL)? {
		// Verify nil has no unexpected outgoing quads.
		for quad in deserializer.peek_quads(Quad(Some(head), None, None, Some(graph))) {
			let Quad(_, predicate, object, _) = quad?;

			// The only allowed quad is `s rdf:type rdf:List`.
			if !deserializer.is_iri(&*predicate, RDF_TYPE)?
				|| !deserializer.is_iri(&*object, RDF_LIST)?
			{
				return Ok(false);
			}
		}

		return Ok(true);
	}

	// Check that it is used at most once.
	if deserializer
		.peek_quads(Quad(None, None, Some(head), Some(graph)))
		.count()
		> 1
	{
		return Ok(false);
	}

	// Check the first value, and extract the rest of the list.
	let mut first = None;
	let mut rest = None;

	for quad in deserializer.peek_quads(Quad(Some(head), None, None, Some(graph))) {
		let Quad(_, predicate, object, _) = quad?;

		for p_term in deserializer.terms_of(&*predicate) {
			if let CowTerm::Ground(CowGroundTerm::Iri(p_iri)) = p_term? {
				if *p_iri == RDF_TYPE && !deserializer.is_iri(&*object, RDF_LIST)? {
					// Only allowed type is rdf:List.
					return Ok(false);
				}

				if *p_iri == RDF_FIRST {
					if let Some(other) = first.replace(object.clone()) {
						if other != object {
							// Can't have multiple first values.
							return Ok(false);
						}
					}
				}

				if *p_iri == RDF_REST {
					if let Some(other) = rest.replace(object.clone()) {
						if other != object {
							// Can't have multiple rest values.
							return Ok(false);
						}
					}
				}
			}
		}
	}

	if first.is_none() {
		// Missing rdf:first.
		return Ok(false);
	}

	let Some(rest) = rest else {
		// Missing rdf:rest.
		return Ok(false);
	};

	// Check the rest of the list.
	is_list_object(deserializer, &*rest, graph)
}
