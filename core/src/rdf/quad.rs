use super::{RdfDirection, ValidId, Value};
use crate::{flattening::NodeMap, ExpandedDocument, FlattenedDocument, LdQuads};
use rdf_types::vocabulary::IriVocabularyMut;
use rdf_types::{
	BlankIdVocabulary, Generator, IriVocabulary, LanguageTagVocabularyMut, LiteralVocabulary,
	LiteralVocabularyMut, Triple, Vocabulary,
};
use std::borrow::Cow;
use std::convert::TryInto;
use std::hash::Hash;

pub type Quad<T, B, L> =
	rdf_types::Quad<ValidId<T, B>, ValidId<T, B>, Value<T, B, L>, ValidId<T, B>>;

pub type QuadRef<'a, T, B, L> = rdf_types::Quad<
	Cow<'a, ValidId<T, B>>,
	Cow<'a, ValidId<T, B>>,
	Value<T, B, L>,
	&'a ValidId<T, B>,
>;

struct Compound<'a, T, B, L> {
	graph: Option<&'a ValidId<T, B>>,
	triples: super::CompoundValueTriples<'a, T, B, L>,
}

type VocabularyCompoundLiteral<'a, N> = Compound<
	'a,
	<N as IriVocabulary>::Iri,
	<N as BlankIdVocabulary>::BlankId,
	<N as LiteralVocabulary>::Literal,
>;

/// Iterator over the RDF Quads of a JSON-LD document.
pub struct Quads<'a, 'n, 'g, N: Vocabulary, G: Generator<N>> {
	vocabulary: &'n mut N,
	generator: &'g mut G,
	rdf_direction: Option<RdfDirection>,
	compound_value: Option<VocabularyCompoundLiteral<'a, N>>,
	quads: crate::quad::Quads<'a, N::Iri, N::BlankId>,
	produce_generalized_rdf: bool,
}

impl<'a, 'n, 'g, N: Vocabulary, G: Generator<N>> Quads<'a, 'n, 'g, N, G> {
	pub fn cloned(self) -> ClonedQuads<'a, 'n, 'g, N, G> {
		ClonedQuads { inner: self }
	}
}

impl<'a, 'n, 'g, N: Vocabulary + IriVocabularyMut + LanguageTagVocabularyMut, G: Generator<N>>
	Iterator for Quads<'a, 'n, 'g, N, G>
where
	N::Iri: Clone,
	N::BlankId: Clone,
	N::Literal: Clone,
	N: LiteralVocabularyMut<
		Type = rdf_types::literal::Type<N::Iri, N::LanguageTag>,
		Value = String,
	>,
{
	type Item = QuadRef<'a, N::Iri, N::BlankId, N::Literal>;

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
pub struct ClonedQuads<'a, 'n, 'g, N: Vocabulary, G: Generator<N>> {
	inner: Quads<'a, 'n, 'g, N, G>,
}

impl<'a, 'n, 'g, N: Vocabulary + IriVocabularyMut + LanguageTagVocabularyMut, G: Generator<N>>
	Iterator for ClonedQuads<'a, 'n, 'g, N, G>
where
	N::Iri: Clone,
	N::BlankId: Clone,
	N::Literal: Clone,
	N: LiteralVocabularyMut<
		Type = rdf_types::literal::Type<N::Iri, N::LanguageTag>,
		Value = String,
	>,
{
	type Item = Quad<N::Iri, N::BlankId, N::Literal>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|rdf_types::Quad(s, p, o, g)| {
			rdf_types::Quad(s.into_owned(), p.into_owned(), o, g.cloned())
		})
	}
}

pub trait RdfQuads<T, B> {
	fn rdf_quads_full<'n, 'g, V: Vocabulary<Iri = T, BlankId = B>, G: Generator<V>>(
		&self,
		vocabulary: &'n mut V,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
		produce_generalized_rdf: bool,
	) -> Quads<'_, 'n, 'g, V, G>;

	fn rdf_quads_with<'n, 'g, V: Vocabulary<Iri = T, BlankId = B>, G: Generator<V>>(
		&self,
		vocabulary: &'n mut V,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Quads<'_, 'n, 'g, V, G> {
		self.rdf_quads_full(vocabulary, generator, rdf_direction, false)
	}

	fn rdf_quads<'g, G: Generator>(
		&self,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
	) -> Quads<'_, 'static, 'g, (), G>
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

impl<T, B> RdfQuads<T, B> for ExpandedDocument<T, B> {
	fn rdf_quads_full<'n, 'g, V: Vocabulary<Iri = T, BlankId = B>, G: Generator<V>>(
		&self,
		vocabulary: &'n mut V,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
		produce_generalized_rdf: bool,
	) -> Quads<'_, 'n, 'g, V, G> {
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

impl<T, B> RdfQuads<T, B> for FlattenedDocument<T, B> {
	fn rdf_quads_full<'n, 'g, V: Vocabulary<Iri = T, BlankId = B>, G: Generator<V>>(
		&self,
		vocabulary: &'n mut V,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
		produce_generalized_rdf: bool,
	) -> Quads<'_, 'n, 'g, V, G> {
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

impl<T: Eq + Hash, B: Eq + Hash> RdfQuads<T, B> for NodeMap<T, B> {
	fn rdf_quads_full<'n, 'g, V: Vocabulary<Iri = T, BlankId = B>, G: Generator<V>>(
		&self,
		vocabulary: &'n mut V,
		generator: &'g mut G,
		rdf_direction: Option<RdfDirection>,
		produce_generalized_rdf: bool,
	) -> Quads<'_, 'n, 'g, V, G> {
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
