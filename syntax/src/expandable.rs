use crate::{
	context::{
		Key,
		KeyRef,
		KeyOrKeyword,
		KeyOrKeywordRef
	},
	Keyword,
	CompactIri
};
use iref::{Iri, IriRef, IriRefBuf};

pub enum Expandable {
	Keyword(Keyword),
	IriRef(IriRefBuf),
	Key(Key),
}

pub enum ExpandableRef<'a> {
	/// Keyword.
	Keyword(Keyword),

	/// Key.
	Key(KeyRef<'a>),

	/// Any IRI reference that is not a key.
	IriRef(IriRef<'a>),
}

impl<'a> From<KeyOrKeywordRef<'a>> for ExpandableRef<'a> {
	fn from(k: KeyOrKeywordRef<'a>) -> Self {
		match k {
			KeyOrKeywordRef::Keyword(k) => Self::Keyword(k),
			KeyOrKeywordRef::Key(k) => Self::Key(k),
		}
	}
}

impl<'a> From<&'a KeyOrKeyword> for ExpandableRef<'a> {
	fn from(k: &'a KeyOrKeyword) -> Self {
		match k {
			KeyOrKeyword::Keyword(k) => Self::Keyword(*k),
			KeyOrKeyword::Key(k) => Self::Key(k.into()),
		}
	}
}

impl<'a> From<&'a str> for ExpandableRef<'a> {
	fn from(s: &'a str) -> Self {
		match Keyword::try_from(s) {
			Ok(k) => Self::Keyword(k),
			Err(_) => match CompactIri::new(s) {
				Ok(c) => Self::Key(KeyRef::CompactIri(c)),
				Err(_) => match Iri::new(s) {
					Ok(i) => Self::Key(KeyRef::Iri(i)),
					Err(_) => match IriRef::new(s) {
						Ok(i) => Self::IriRef(i),
						Err(_) => Self::Key(KeyRef::Term(s))
					}
				}
			}
		}
	}
}