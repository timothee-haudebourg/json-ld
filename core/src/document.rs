use std::ops::Deref;
use std::{borrow::Borrow, hash::Hash};

use iref::IriBuf;
use linked_data::{LinkedData, LinkedDataGraph, LinkedDataResource, LinkedDataSubject};
use locspan::{Meta, StrippedEq, StrippedPartialEq};
use rdf_types::{
	BlankIdBuf, Interpretation, IriVocabularyMut, LanguageTagVocabularyMut, Vocabulary,
};

pub mod expanded;
pub mod flattened;

pub use expanded::ExpandedDocument;
pub use flattened::FlattenedDocument;

use crate::RemoteDocument;

/// JSON-LD document in both compact and expanded form.
pub struct Document<I = IriBuf, B = BlankIdBuf, M = ()> {
	remote: RemoteDocument<I, M>,
	expanded: Meta<ExpandedDocument<I, B, M>, M>,
}

impl<I, B, M> Document<I, B, M> {
	pub fn new(remote: RemoteDocument<I, M>, expanded: Meta<ExpandedDocument<I, B, M>, M>) -> Self {
		Self { remote, expanded }
	}

	pub fn into_remote(self) -> RemoteDocument<I, M> {
		self.remote
	}

	pub fn into_compact(self) -> json_ld_syntax::MetaValue<M> {
		self.remote.into_document()
	}

	pub fn into_expanded(self) -> Meta<ExpandedDocument<I, B, M>, M> {
		self.expanded
	}

	pub fn into_parts(self) -> (RemoteDocument<I, M>, Meta<ExpandedDocument<I, B, M>, M>) {
		(self.remote, self.expanded)
	}

	pub fn as_remote(&self) -> &RemoteDocument<I, M> {
		&self.remote
	}

	pub fn as_compact(&self) -> &json_ld_syntax::MetaValue<M> {
		self.remote.document()
	}

	pub fn as_expanded(&self) -> &Meta<ExpandedDocument<I, B, M>, M> {
		&self.expanded
	}
}

impl<I, B, M> Deref for Document<I, B, M> {
	type Target = ExpandedDocument<I, B, M>;

	fn deref(&self) -> &Self::Target {
		self.expanded.value()
	}
}

impl<I, B, M> Borrow<RemoteDocument<I, M>> for Document<I, B, M> {
	fn borrow(&self) -> &RemoteDocument<I, M> {
		&self.remote
	}
}

impl<I, B, M> Borrow<json_ld_syntax::Value<M>> for Document<I, B, M> {
	fn borrow(&self) -> &json_ld_syntax::Value<M> {
		self.remote.document().value()
	}
}

impl<I, B, M> Borrow<ExpandedDocument<I, B, M>> for Document<I, B, M> {
	fn borrow(&self) -> &ExpandedDocument<I, B, M> {
		self.expanded.value()
	}
}

impl<I: Eq + Hash, B: Eq + Hash, M> StrippedPartialEq for Document<I, B, M> {
	fn stripped_eq(&self, other: &Self) -> bool {
		self.expanded.stripped_eq(&other.expanded)
	}
}

impl<I: Eq + Hash, B: Eq + Hash, M> StrippedEq for Document<I, B, M> {}

#[cfg(feature = "serde")]
impl<I, B, M> serde::Serialize for Document<I, B, M> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.remote.document().value().serialize(serializer)
	}
}

impl<V: Vocabulary, I: Interpretation, M> LinkedData<V, I> for Document<V::Iri, V::BlankId, M>
where
	V: IriVocabularyMut + LanguageTagVocabularyMut,
	V::Iri: LinkedDataSubject<V, I> + LinkedDataResource<V, I>,
	V::BlankId: LinkedDataSubject<V, I> + LinkedDataResource<V, I>,
	M: Clone,
{
	fn visit<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::Visitor<V, I>,
	{
		self.expanded.value().visit(visitor)
	}
}

impl<V: Vocabulary, I: Interpretation, M> LinkedDataGraph<V, I> for Document<V::Iri, V::BlankId, M>
where
	V: IriVocabularyMut + LanguageTagVocabularyMut,
	V::Iri: LinkedDataSubject<V, I> + LinkedDataResource<V, I>,
	V::BlankId: LinkedDataSubject<V, I> + LinkedDataResource<V, I>,
	M: Clone,
{
	fn visit_graph<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::GraphVisitor<V, I>,
	{
		self.expanded.value().visit_graph(visitor)
	}
}
