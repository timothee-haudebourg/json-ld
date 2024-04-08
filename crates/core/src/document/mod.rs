use std::ops::Deref;
use std::{borrow::Borrow, hash::Hash};

use iref::IriBuf;
use linked_data::{LinkedData, LinkedDataGraph, LinkedDataResource, LinkedDataSubject};
use rdf_types::{vocabulary::IriVocabularyMut, BlankIdBuf, Interpretation, Vocabulary};

pub mod expanded;
pub mod flattened;

pub use expanded::ExpandedDocument;
pub use flattened::FlattenedDocument;

use crate::RemoteDocument;

/// JSON-LD document in both compact and expanded form.
#[derive(Debug, Clone)]
pub struct Document<I = IriBuf, B = BlankIdBuf> {
	remote: RemoteDocument<I>,
	expanded: ExpandedDocument<I, B>,
}

impl<I, B> Document<I, B> {
	pub fn new(remote: RemoteDocument<I>, expanded: ExpandedDocument<I, B>) -> Self {
		Self { remote, expanded }
	}

	pub fn into_remote(self) -> RemoteDocument<I> {
		self.remote
	}

	pub fn into_compact(self) -> json_ld_syntax::Value {
		self.remote.into_document()
	}

	pub fn into_expanded(self) -> ExpandedDocument<I, B> {
		self.expanded
	}

	#[allow(clippy::type_complexity)]
	pub fn into_parts(self) -> (RemoteDocument<I>, ExpandedDocument<I, B>) {
		(self.remote, self.expanded)
	}

	pub fn as_remote(&self) -> &RemoteDocument<I> {
		&self.remote
	}

	pub fn as_compact(&self) -> &json_ld_syntax::Value {
		self.remote.document()
	}

	pub fn as_expanded(&self) -> &ExpandedDocument<I, B> {
		&self.expanded
	}
}

impl<I, B> Deref for Document<I, B> {
	type Target = ExpandedDocument<I, B>;

	fn deref(&self) -> &Self::Target {
		&self.expanded
	}
}

impl<I, B> Borrow<RemoteDocument<I>> for Document<I, B> {
	fn borrow(&self) -> &RemoteDocument<I> {
		&self.remote
	}
}

impl<I, B> Borrow<json_ld_syntax::Value> for Document<I, B> {
	fn borrow(&self) -> &json_ld_syntax::Value {
		self.remote.document()
	}
}

impl<I, B> Borrow<ExpandedDocument<I, B>> for Document<I, B> {
	fn borrow(&self) -> &ExpandedDocument<I, B> {
		&self.expanded
	}
}

impl<I: Eq + Hash, B: Eq + Hash> PartialEq for Document<I, B> {
	fn eq(&self, other: &Self) -> bool {
		self.expanded.eq(&other.expanded)
	}
}

impl<I: Eq + Hash, B: Eq + Hash> Eq for Document<I, B> {}

#[cfg(feature = "serde")]
impl<I, B> serde::Serialize for Document<I, B> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.remote.document().serialize(serializer)
	}
}

impl<V: Vocabulary, I: Interpretation> LinkedData<I, V> for Document<V::Iri, V::BlankId>
where
	V: IriVocabularyMut,
	V::Iri: LinkedDataSubject<I, V> + LinkedDataResource<I, V>,
	V::BlankId: LinkedDataSubject<I, V> + LinkedDataResource<I, V>,
{
	fn visit<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::Visitor<I, V>,
	{
		self.expanded.visit(visitor)
	}
}

impl<V: Vocabulary, I: Interpretation> LinkedDataGraph<I, V> for Document<V::Iri, V::BlankId>
where
	V: IriVocabularyMut,
	V::Iri: LinkedDataSubject<I, V> + LinkedDataResource<I, V>,
	V::BlankId: LinkedDataSubject<I, V> + LinkedDataResource<I, V>,
{
	fn visit_graph<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::GraphVisitor<I, V>,
	{
		self.expanded.visit_graph(visitor)
	}
}
