use super::{RdfDirection, ValidId, Value};
use crate::{flattening::NodeMap, id, ExpandedDocument, FlattenedDocument, LdQuads};
use rdf_types::vocabulary::IriVocabularyMut;
use rdf_types::{Triple, Vocabulary};
use std::borrow::Cow;
use std::convert::TryInto;
use std::hash::Hash;

pub type Quad<T, B> = rdf_types::Quad<ValidId<T, B>, ValidId<T, B>, Value<T, B>, ValidId<T, B>>;

pub type QuadRef<'a, T, B> =
	rdf_types::Quad<Cow<'a, ValidId<T, B>>, Cow<'a, ValidId<T, B>>, Value<T, B>, &'a ValidId<T, B>>;

struct Compound<'a, T, B, M> {
	graph: Option<&'a ValidId<T, B>>,
	triples: super::CompoundValueTriples<'a, T, B, M>,
}

/// Iterator over the RDF Quads of a JSON-LD document.
pub struct Quads<'a, 'n, 'g, N: Vocabulary, M, G: id::Generator<N, M>> {
	vocabulary: &'n mut N,
	generator: &'g mut G,
	rdf_direction: Option<RdfDirection>,
	compound_value: Option<Compound<'a, N::Iri, N::BlankId, M>>,
	quads: crate::quad::Quads<'a, N::Iri, N::BlankId, M>,
	produce_generalized_rdf: bool,
}

impl<'a, 'n, 'g, N: Vocabulary, M, G: id::Generator<N, M>> Quads<'a, 'n, 'g, N, M, G> {
	pub fn cloned(self) -> ClonedQuads<'a, 'n, 'g, N, M, G> {
		ClonedQuads { inner: self }
	}
}

impl<'a, 'n, 'g, N: Vocabulary + IriVocabularyMut, M, G: id::Generator<N, M>> Iterator
	for Quads<'a, 'n, 'g, N, M, G>
where
	N::Iri: Clone,
	N::BlankId: Clone,
{
	type Item = QuadRef<'a, N::Iri, N::BlankId>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			if let Some(compound_value) = &mut self.compound_value {
				match compound_value.triples.next(
					self.vocabulary,
					self.generator,
					self.rdf_direction,
				) {
					Some(Triple(subject, property, object)) => {
						if self.produce_generalized_rdf || !property.is_blank() {
							break Some(rdf_types::Quad(
								Cow::Owned(subject),
								Cow::Owned(property),
								object,
								compound_value.graph,
							));
						}
					}
					None => self.compound_value = None,
				}
			}

			match self.quads.next() {
				Some(crate::quad::QuadRef(graph, subject, property, object)) => {
					let rdf_graph: Option<&'a ValidId<N::Iri, N::BlankId>> =
						match graph.map(|r| r.try_into()) {
							Some(Ok(r)) => Some(r),
							None => None,
							_ => continue,
						};

					let rdf_subject: &'a ValidId<N::Iri, N::BlankId> = match subject.try_into() {
						Ok(r) => r,
						Err(_) => continue,
					};

					let rdf_property: Cow<ValidId<N::Iri, N::BlankId>> = match property {
						crate::quad::PropertyRef::Type => {
							Cow::Owned(ValidId::Iri(self.vocabulary.insert(super::RDF_TYPE)))
						}
						crate::quad::PropertyRef::Ref(r) => match r.try_into() {
							Ok(r) => Cow::Borrowed(r),
							Err(_) => continue,
						},
					};

					if !self.produce_generalized_rdf && (*rdf_property).is_blank() {
						// Skip gRDF quad.
						continue;
					}

					if let Some(compound_value) =
						object.rdf_value_with(self.vocabulary, self.generator, self.rdf_direction)
					{
						if let Some(rdf_value_triples) = compound_value.triples {
							self.compound_value = Some(Compound {
								graph: rdf_graph,
								triples: rdf_value_triples,
							});
						}

						break Some(rdf_types::Quad(
							Cow::Borrowed(rdf_subject),
							rdf_property,
							compound_value.value,
							rdf_graph,
						));
					}
				}
				None => break None,
			}
		}
	}
}

/// Iterator over the RDF Quads of a JSON-LD document where borrowed values are
/// cloned.
pub struct ClonedQuads<'a, 'n, 'g, N: Vocabulary, M, G: id::Generator<N, M>> {
	inner: Quads<'a, 'n, 'g, N, M, G>,
}

impl<'a, 'n, 'g, N: Vocabulary + IriVocabularyMut, M, G: id::Generator<N, M>> Iterator
	for ClonedQuads<'a, 'n, 'g, N, M, G>
where
	N::Iri: Clone,
	N::BlankId: Clone,
{
	type Item = Quad<N::Iri, N::BlankId>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|rdf_types::Quad(s, p, o, g)| {
			rdf_types::Quad(s.into_owned(), p.into_owned(), o, g.cloned())
		})
	}
}

pub trait RdfQuads<T, B, M> {
	fn rdf_quads_full<'n, 'g, V: Vocabulary<Iri = T, BlankId = B>, G: id::Generator<V, M>>(
		&self,
		vocabulary: &'n mut V,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
		produce_generalized_rdf: bool,
	) -> Quads<'_, 'n, 'g, V, M, G>;

	fn rdf_quads_with<'n, 'g, V: Vocabulary<Iri = T, BlankId = B>, G: id::Generator<V, M>>(
		&self,
		vocabulary: &'n mut V,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Quads<'_, 'n, 'g, V, M, G> {
		self.rdf_quads_full(vocabulary, generator, rdf_direction, false)
	}

	fn rdf_quads<'g, G: id::Generator<(), M>>(
		&self,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Quads<'_, 'static, 'g, (), M, G>
	where
		(): Vocabulary<Iri = T, BlankId = B>,
	{
		self.rdf_quads_with(
			rdf_types::vocabulary::no_vocabulary_mut(),
			generator,
			rdf_direction,
		)
	}
}

impl<T, B, M> RdfQuads<T, B, M> for ExpandedDocument<T, B, M> {
	fn rdf_quads_full<'n, 'g, V: Vocabulary<Iri = T, BlankId = B>, G: id::Generator<V, M>>(
		&self,
		vocabulary: &'n mut V,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
		produce_generalized_rdf: bool,
	) -> Quads<'_, 'n, 'g, V, M, G> {
		Quads {
			vocabulary,
			generator,
			rdf_direction,
			compound_value: None,
			quads: self.quads(),
			produce_generalized_rdf,
		}
	}
}

impl<T, B, M> RdfQuads<T, B, M> for FlattenedDocument<T, B, M> {
	fn rdf_quads_full<'n, 'g, V: Vocabulary<Iri = T, BlankId = B>, G: id::Generator<V, M>>(
		&self,
		vocabulary: &'n mut V,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
		produce_generalized_rdf: bool,
	) -> Quads<'_, 'n, 'g, V, M, G> {
		Quads {
			vocabulary,
			generator,
			rdf_direction,
			compound_value: None,
			quads: self.quads(),
			produce_generalized_rdf,
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> RdfQuads<T, B, M> for NodeMap<T, B, M> {
	fn rdf_quads_full<'n, 'g, V: Vocabulary<Iri = T, BlankId = B>, G: id::Generator<V, M>>(
		&self,
		vocabulary: &'n mut V,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
		produce_generalized_rdf: bool,
	) -> Quads<'_, 'n, 'g, V, M, G> {
		Quads {
			vocabulary,
			generator,
			rdf_direction,
			compound_value: None,
			quads: self.quads(),
			produce_generalized_rdf,
		}
	}
}
