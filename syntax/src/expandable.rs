use crate::{
	context::{
		Key,
		KeyRef,
		KeyOrKeyword,
		KeyOrKeywordRef
	},
	Keyword
};
use iref::{IriRef, IriRefBuf};

pub enum Expandable {
	Null,
	Keyword(Keyword),
	IriRef(IriRefBuf),
	Key(Key),
}

pub enum ExpandableRef<'a> {
	Keyword(Keyword),
	IriRef(IriRef<'a>),
	Key(KeyRef<'a>)
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