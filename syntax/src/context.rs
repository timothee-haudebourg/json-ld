use crate::{
	CompactIriBuf, ComponentRef, Container, ContainerType, Direction, Keyword,
	LenientLanguageTagBuf, Nullable, TryFromJson,
};
use derivative::Derivative;
use indexmap::IndexMap;
use iref::{IriBuf, IriRefBuf};
use locspan::Meta;
use locspan_derive::StrippedPartialEq;
use rdf_types::BlankIdBuf;

mod key;
mod print;
mod reference;

pub use key::*;
pub use reference::*;

/// Context entry.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Debug)]
#[stripped_ignore(M)]
pub enum ContextEntry<M> {
	One(Meta<Context<M>, M>),
	Many(Vec<Meta<Context<M>, M>>),
}

impl<M> ContextEntry<M> {
	pub fn as_slice(&self) -> &[Meta<Context<M>, M>] {
		match self {
			Self::One(c) => std::slice::from_ref(c),
			Self::Many(list) => list,
		}
	}
}

impl<M> From<Meta<Context<M>, M>> for ContextEntry<M> {
	fn from(c: Meta<Context<M>, M>) -> Self {
		Self::One(c)
	}
}

impl<M: Default> From<Context<M>> for ContextEntry<M> {
	fn from(c: Context<M>) -> Self {
		Self::One(Meta(c, M::default()))
	}
}

impl<M: Default> From<IriRefBuf> for ContextEntry<M> {
	fn from(i: IriRefBuf) -> Self {
		Self::One(Meta(Context::IriRef(i), M::default()))
	}
}

impl<'a, M: Default> From<iref::IriRef<'a>> for ContextEntry<M> {
	fn from(i: iref::IriRef<'a>) -> Self {
		Self::One(Meta(Context::IriRef(i.into()), M::default()))
	}
}

impl<M: Default> From<iref::IriBuf> for ContextEntry<M> {
	fn from(i: iref::IriBuf) -> Self {
		Self::One(Meta(Context::IriRef(i.into()), M::default()))
	}
}

impl<'a, M: Default> From<iref::Iri<'a>> for ContextEntry<M> {
	fn from(i: iref::Iri<'a>) -> Self {
		Self::One(Meta(Context::IriRef(i.into()), M::default()))
	}
}

impl<M: Default> From<ContextDefinition<M>> for ContextEntry<M> {
	fn from(c: ContextDefinition<M>) -> Self {
		Self::One(Meta(Context::Definition(c), M::default()))
	}
}

impl<M> From<Meta<IriRefBuf, M>> for ContextEntry<M> {
	fn from(Meta(i, meta): Meta<IriRefBuf, M>) -> Self {
		Self::One(Meta(Context::IriRef(i), meta))
	}
}

impl<'a, M> From<Meta<iref::IriRef<'a>, M>> for ContextEntry<M> {
	fn from(Meta(i, meta): Meta<iref::IriRef<'a>, M>) -> Self {
		Self::One(Meta(Context::IriRef(i.into()), meta))
	}
}

impl<M> From<Meta<iref::IriBuf, M>> for ContextEntry<M> {
	fn from(Meta(i, meta): Meta<iref::IriBuf, M>) -> Self {
		Self::One(Meta(Context::IriRef(i.into()), meta))
	}
}

impl<'a, M> From<Meta<iref::Iri<'a>, M>> for ContextEntry<M> {
	fn from(Meta(i, meta): Meta<iref::Iri<'a>, M>) -> Self {
		Self::One(Meta(Context::IriRef(i.into()), meta))
	}
}

impl<M> From<Meta<ContextDefinition<M>, M>> for ContextEntry<M> {
	fn from(Meta(c, meta): Meta<ContextDefinition<M>, M>) -> Self {
		Self::One(Meta(Context::Definition(c), meta))
	}
}

pub trait Count<C: AnyContextEntry> {
	fn count<F>(&self, f: F) -> usize
	where
		F: Clone + Fn(ComponentRef<C>) -> bool;
}

pub trait IntoCount<C: AnyContextEntry> {
	fn into_count<F>(self, f: F) -> usize
	where
		F: Clone + Fn(ComponentRef<C>) -> bool;
}

/// Context.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Debug)]
#[stripped_ignore(M)]
pub enum Context<M> {
	Null,
	IriRef(#[stripped] IriRefBuf),
	Definition(ContextDefinition<M>),
}

impl<M> From<IriRefBuf> for Context<M> {
	fn from(i: IriRefBuf) -> Self {
		Context::IriRef(i)
	}
}

impl<'a, M> From<iref::IriRef<'a>> for Context<M> {
	fn from(i: iref::IriRef<'a>) -> Self {
		Context::IriRef(i.into())
	}
}

impl<M> From<iref::IriBuf> for Context<M> {
	fn from(i: iref::IriBuf) -> Self {
		Context::IriRef(i.into())
	}
}

impl<'a, M> From<iref::Iri<'a>> for Context<M> {
	fn from(i: iref::Iri<'a>) -> Self {
		Context::IriRef(i.into())
	}
}

impl<M> From<ContextDefinition<M>> for Context<M> {
	fn from(c: ContextDefinition<M>) -> Self {
		Context::Definition(c)
	}
}

/// Context definition.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Derivative, Debug)]
#[stripped_ignore(M)]
#[derivative(Default(bound = ""))]
pub struct ContextDefinition<M> {
	#[stripped_option_deref]
	pub base: Option<Meta<Nullable<IriRefBuf>, M>>,
	#[stripped_option_deref]
	pub import: Option<Meta<IriRefBuf, M>>,
	pub language: Option<Meta<Nullable<LenientLanguageTagBuf>, M>>,
	pub direction: Option<Meta<Nullable<Direction>, M>>,
	pub propagate: Option<Meta<bool, M>>,
	pub protected: Option<Meta<bool, M>>,
	pub type_: Option<Meta<ContextType<M>, M>>,
	pub version: Option<Meta<Version, M>>,
	pub vocab: Option<Meta<Nullable<Vocab>, M>>,
	pub bindings: Bindings<M>,
}

impl<M> ContextDefinition<M> {
	pub fn new() -> Self {
		Self::default()
	}
}

/// Context bindings.
#[derive(PartialEq, Eq, Clone, Derivative, Debug)]
#[derivative(Default(bound = ""))]
pub struct Bindings<M>(IndexMap<Key, TermBinding<M>>);

impl<M> Bindings<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn get(&self, key: &Key) -> Option<&TermBinding<M>> {
		self.0.get(key)
	}

	pub fn iter(&self) -> indexmap::map::Iter<Key, TermBinding<M>> {
		self.0.iter()
	}

	pub fn insert(
		&mut self,
		Meta(key, key_metadata): Meta<Key, M>,
		def: Meta<Nullable<TermDefinition<M>>, M>,
	) -> Option<TermBinding<M>> {
		self.0.insert(key, TermBinding::new(key_metadata, def))
	}
}

impl<M> locspan::StrippedPartialEq for Bindings<M> {
	fn stripped_eq(&self, other: &Self) -> bool {
		self.len() == other.len()
			&& self
				.iter()
				.all(|(key, a)| other.get(key).map(|b| a.stripped_eq(b)).unwrap_or(false))
	}
}

/// Term binding.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Debug)]
#[stripped_ignore(M)]
pub struct TermBinding<M> {
	#[stripped_ignore]
	pub key_metadata: M,
	pub definition: Meta<Nullable<TermDefinition<M>>, M>,
}

impl<M> TermBinding<M> {
	pub fn new(key_metadata: M, definition: Meta<Nullable<TermDefinition<M>>, M>) -> Self {
		Self {
			key_metadata,
			definition,
		}
	}
}

/// Term definition.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Debug)]
#[stripped_ignore(M)]
pub enum TermDefinition<M> {
	Iri(#[stripped] IriBuf),
	CompactIri(#[stripped] CompactIriBuf),
	Blank(#[stripped] BlankIdBuf),
	Expanded(ExpandedTermDefinition<M>),
}

/// Expanded term definition.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Derivative, Debug)]
#[stripped_ignore(M)]
#[derivative(Default(bound = ""))]
pub struct ExpandedTermDefinition<M> {
	pub id: Option<Meta<Nullable<Id>, M>>,
	pub type_: Option<Meta<Nullable<TermDefinitionType>, M>>,
	pub context: Option<Box<Meta<ContextEntry<M>, M>>>,
	pub reverse: Option<Meta<Key, M>>,
	pub index: Option<Meta<Index, M>>,
	pub language: Option<Meta<Nullable<LenientLanguageTagBuf>, M>>,
	pub direction: Option<Meta<Nullable<Direction>, M>>,
	pub container: Option<Meta<Nullable<Container<M>>, M>>,
	pub nest: Option<Meta<Nest, M>>,
	pub prefix: Option<Meta<bool, M>>,
	pub propagate: Option<Meta<bool, M>>,
	pub protected: Option<Meta<bool, M>>,
}

impl<M> ExpandedTermDefinition<M> {
	pub fn new() -> Self {
		Self::default()
	}
}

#[derive(Clone, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Nest {
	Nest,
	Term(#[stripped] String),
}

impl From<String> for Nest {
	fn from(s: String) -> Self {
		if s == "@nest" {
			Self::Nest
		} else {
			Self::Term(s)
		}
	}
}

#[derive(Clone, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Index {
	Iri(#[stripped] IriBuf),
	CompactIri(#[stripped] CompactIriBuf),
	Term(#[stripped] String),
}

impl Index {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Iri(i) => i.as_str(),
			Self::CompactIri(c) => c.as_str(),
			Self::Term(t) => t.as_str(),
		}
	}
}

impl From<String> for Index {
	fn from(s: String) -> Self {
		match CompactIriBuf::new(s) {
			Ok(c) => Self::CompactIri(c),
			Err(crate::InvalidCompactIri(s)) => match IriBuf::from_string(s) {
				Ok(iri) => Self::Iri(iri),
				Err((_, s)) => Self::Term(s),
			},
		}
	}
}

#[derive(Clone, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Id {
	Iri(#[stripped] IriBuf),
	Blank(#[stripped] BlankIdBuf),
	CompactIri(#[stripped] CompactIriBuf),
	Term(#[stripped] String),
	Keyword(Keyword),
}

impl From<String> for Id {
	fn from(s: String) -> Self {
		match Keyword::try_from(s.as_str()) {
			Ok(k) => Self::Keyword(k),
			Err(_) => match BlankIdBuf::new(s) {
				Ok(b) => Self::Blank(b),
				Err(rdf_types::InvalidBlankId(s)) => match CompactIriBuf::new(s) {
					Ok(c) => Self::CompactIri(c),
					Err(crate::InvalidCompactIri(s)) => match IriBuf::from_string(s) {
						Ok(iri) => Self::Iri(iri),
						Err((_, s)) => Self::Term(s),
					},
				},
			},
		}
	}
}

#[derive(Clone, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum TermDefinitionType {
	Iri(#[stripped] IriBuf),
	CompactIri(#[stripped] CompactIriBuf),
	Term(#[stripped] String),
	Keyword(TypeKeyword),
}

impl From<String> for TermDefinitionType {
	fn from(s: String) -> Self {
		match TypeKeyword::try_from(s.as_str()) {
			Ok(k) => Self::Keyword(k),
			Err(_) => match CompactIriBuf::new(s) {
				Ok(c) => Self::CompactIri(c),
				Err(crate::InvalidCompactIri(s)) => match IriBuf::from_string(s) {
					Ok(iri) => Self::Iri(iri),
					Err((_, s)) => Self::Term(s),
				},
			},
		}
	}
}

/// Subset of keyword acceptable for as value for the `@type` entry
/// of an expanded term definition.
#[derive(Clone, Copy, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum TypeKeyword {
	Id,
	Json,
	None,
	Vocab,
}

impl TypeKeyword {
	pub fn keyword(&self) -> Keyword {
		self.into_keyword()
	}

	pub fn into_keyword(self) -> Keyword {
		self.into()
	}

	pub fn into_str(self) -> &'static str {
		self.into_keyword().into_str()
	}
}

pub struct NotATypeKeyword(pub Keyword);

pub enum InvalidTypeKeyword<T> {
	NotAKeyword(T),
	NotATypeKeyword(Keyword),
}

impl<T> From<NotATypeKeyword> for InvalidTypeKeyword<T> {
	fn from(NotATypeKeyword(k): NotATypeKeyword) -> Self {
		Self::NotATypeKeyword(k)
	}
}

impl<T> From<crate::NotAKeyword<T>> for InvalidTypeKeyword<T> {
	fn from(crate::NotAKeyword(t): crate::NotAKeyword<T>) -> Self {
		Self::NotAKeyword(t)
	}
}

impl From<TypeKeyword> for Keyword {
	fn from(k: TypeKeyword) -> Self {
		match k {
			TypeKeyword::Id => Self::Id,
			TypeKeyword::Json => Self::Json,
			TypeKeyword::None => Self::None,
			TypeKeyword::Vocab => Self::Vocab,
		}
	}
}

impl TryFrom<Keyword> for TypeKeyword {
	type Error = NotATypeKeyword;

	fn try_from(k: Keyword) -> Result<Self, Self::Error> {
		match k {
			Keyword::Id => Ok(Self::Id),
			Keyword::Json => Ok(Self::Json),
			Keyword::None => Ok(Self::None),
			Keyword::Vocab => Ok(Self::Vocab),
			_ => Err(NotATypeKeyword(k)),
		}
	}
}

impl<'a> TryFrom<&'a str> for TypeKeyword {
	type Error = InvalidTypeKeyword<&'a str>;

	fn try_from(s: &'a str) -> Result<Self, Self::Error> {
		Ok(Self::try_from(Keyword::try_from(s)?)?)
	}
}

/// Version number.
///
/// The only allowed value is a number with the value `1.1`.
#[derive(Clone, Copy, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Version {
	V1_1,
}

#[derive(Clone, Copy, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Import;

#[derive(Clone, Copy, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[stripped_ignore(M)]
pub struct ContextType<M> {
	pub container: Meta<TypeContainer, M>,
	pub protected: Option<Meta<bool, M>>,
}

impl<M> ContextType<M> {
	pub fn iter(&self) -> ContextTypeEntries<M> {
		ContextTypeEntries {
			container: Some(&self.container),
			protected: self.protected.as_ref(),
		}
	}
}

pub struct ContextTypeEntries<'a, M> {
	container: Option<&'a Meta<TypeContainer, M>>,
	protected: Option<&'a Meta<bool, M>>,
}

impl<'a, M> Iterator for ContextTypeEntries<'a, M> {
	type Item = ContextTypeEntry<'a, M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let mut len = 0;

		if self.container.is_some() {
			len += 1;
		}

		if self.protected.is_some() {
			len += 1;
		}

		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		match self.container.take() {
			Some(c) => Some(ContextTypeEntry::Container(c)),
			None => self.protected.take().map(ContextTypeEntry::Protected),
		}
	}
}

impl<'a, M> ExactSizeIterator for ContextTypeEntries<'a, M> {}

pub enum ContextTypeEntry<'a, M> {
	Container(&'a Meta<TypeContainer, M>),
	Protected(&'a Meta<bool, M>),
}

impl<'a, M> ContextTypeEntry<'a, M> {
	pub fn key(&self) -> ContextTypeKey {
		match self {
			Self::Container(_) => ContextTypeKey::Container,
			Self::Protected(_) => ContextTypeKey::Protected,
		}
	}
}

pub enum ContextTypeKey {
	Container,
	Protected,
}

impl ContextTypeKey {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Container => "@container",
			Self::Protected => "@protected",
		}
	}
}

#[derive(Clone, Copy, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum TypeContainer {
	Set,
}

impl TypeContainer {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Set => "@set",
		}
	}
}

#[derive(Clone, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Vocab {
	IriRef(#[stripped] IriRefBuf),
	CompactIri(#[stripped] CompactIriBuf),
	Blank(#[stripped] BlankIdBuf),
	Term(#[stripped] String),
}

impl From<String> for Vocab {
	fn from(s: String) -> Self {
		match BlankIdBuf::new(s) {
			Ok(b) => Self::Blank(b),
			Err(rdf_types::InvalidBlankId(s)) => match CompactIriBuf::new(s) {
				Ok(c) => Self::CompactIri(c),
				Err(crate::InvalidCompactIri(s)) => match IriRefBuf::from_string(s) {
					Ok(iri_ref) => Self::IriRef(iri_ref),
					Err((_, s)) => Self::Term(s),
				},
			},
		}
	}
}

#[derive(Clone, Debug)]
pub enum InvalidContext {
	InvalidIriRef(iref::Error),
	Unexpected(json_syntax::Kind, &'static [json_syntax::Kind]),
	InvalidDirection,
	DuplicateKey,
	InvalidTermDefinition,
}

impl From<crate::Unexpected> for InvalidContext {
	fn from(crate::Unexpected(u, e): crate::Unexpected) -> Self {
		Self::Unexpected(u, e)
	}
}

pub trait TryFromStrippedJson<M>: Sized {
	fn try_from_stripped_json(value: json_syntax::Value<M>) -> Result<Self, InvalidContext>;
}

impl<M> TryFromStrippedJson<M> for IriRefBuf {
	fn try_from_stripped_json(value: json_syntax::Value<M>) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => match IriRefBuf::from_string(s.into_string()) {
				Ok(iri_ref) => Ok(iri_ref),
				Err((e, _)) => Err(InvalidContext::InvalidIriRef(e)),
			},
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl<M> TryFromJson<M> for IriRefBuf {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::String(s) => match IriRefBuf::from_string(s.into_string()) {
				Ok(iri_ref) => Ok(Meta(iri_ref, meta)),
				Err((e, _)) => Err(Meta(InvalidContext::InvalidIriRef(e), meta)),
			},
			unexpected => Err(Meta(
				InvalidContext::Unexpected(unexpected.kind(), &[json_syntax::Kind::String]),
				meta,
			)),
		}
	}
}

impl<M> TryFromStrippedJson<M> for LenientLanguageTagBuf {
	fn try_from_stripped_json(value: json_syntax::Value<M>) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => {
				let (lang, _) = LenientLanguageTagBuf::new(s.into_string());
				Ok(lang)
			}
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl<M> TryFromStrippedJson<M> for Direction {
	fn try_from_stripped_json(value: json_syntax::Value<M>) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => match Direction::try_from(s.as_str()) {
				Ok(d) => Ok(d),
				Err(_) => Err(InvalidContext::InvalidDirection),
			},
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl<M, T: TryFromStrippedJson<M>> TryFromJson<M> for Nullable<T> {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::Null => Ok(Meta(Self::Null, meta)),
			some => match T::try_from_stripped_json(some) {
				Ok(some) => Ok(Meta(Self::Some(some), meta)),
				Err(e) => Err(Meta(e, meta)),
			},
		}
	}
}

impl<M> TryFromJson<M> for TypeContainer {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::String(s) => match Keyword::try_from(s.as_str()) {
				Ok(Keyword::Set) => Ok(Meta(Self::Set, meta)),
				_ => Err(Meta(InvalidContext::InvalidTermDefinition, meta)),
			},
			unexpected => Err(Meta(
				InvalidContext::Unexpected(unexpected.kind(), &[json_syntax::Kind::String]),
				meta,
			)),
		}
	}
}

impl<M> TryFromJson<M> for ContextType<M> {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::Object(o) => {
				let mut container = None;
				let mut protected = None;

				for json_syntax::object::Entry {
					key: Meta(key, key_metadata),
					value,
				} in o
				{
					match Keyword::try_from(key.as_str()) {
						Ok(Keyword::Container) => {
							if let Some(prev) =
								container.replace(TypeContainer::try_from_json(value)?)
							{
								return Err(Meta(
									InvalidContext::DuplicateKey,
									prev.into_metadata(),
								));
							}
						}
						Ok(Keyword::Protected) => {
							if let Some(prev) =
								protected.replace(bool::try_from_json(value).map_err(Meta::cast)?)
							{
								return Err(Meta(
									InvalidContext::DuplicateKey,
									prev.into_metadata(),
								));
							}
						}
						_ => return Err(Meta(InvalidContext::InvalidTermDefinition, key_metadata)),
					}
				}

				match container {
					Some(container) => Ok(Meta(
						Self {
							container,
							protected,
						},
						meta,
					)),
					None => Err(Meta(InvalidContext::InvalidTermDefinition, meta)),
				}
			}
			unexpected => Err(Meta(
				InvalidContext::Unexpected(unexpected.kind(), &[json_syntax::Kind::Object]),
				meta,
			)),
		}
	}
}

impl<M> TryFromJson<M> for Version {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::Number(n) => match n.as_str() {
				"1.1" => Ok(Meta(Self::V1_1, meta)),
				_ => Err(Meta(InvalidContext::InvalidTermDefinition, meta)),
			},
			unexpected => Err(Meta(
				InvalidContext::Unexpected(unexpected.kind(), &[json_syntax::Kind::Number]),
				meta,
			)),
		}
	}
}

impl<M> TryFromStrippedJson<M> for Vocab {
	fn try_from_stripped_json(value: json_syntax::Value<M>) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => Ok(Self::from(s.into_string())),
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl<M> TryFromStrippedJson<M> for Id {
	fn try_from_stripped_json(value: json_syntax::Value<M>) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => Ok(Self::from(s.into_string())),
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl<M> TryFromStrippedJson<M> for TermDefinitionType {
	fn try_from_stripped_json(value: json_syntax::Value<M>) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => Ok(Self::from(s.into_string())),
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl<M> TryFromJson<M> for Key {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<Self::Error, M>> {
		match value {
			json_syntax::Value::String(s) => Ok(Meta(Self::from(s.into_string()), meta)),
			unexpected => Err(Meta(
				InvalidContext::Unexpected(unexpected.kind(), &[json_syntax::Kind::String]),
				meta,
			)),
		}
	}
}

impl<M> TryFromJson<M> for Index {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::String(s) => Ok(Meta(Self::from(s.into_string()), meta)),
			unexpected => Err(Meta(
				InvalidContext::Unexpected(unexpected.kind(), &[json_syntax::Kind::String]),
				meta,
			)),
		}
	}
}

impl<M> TryFromJson<M> for Nest {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::String(s) => Ok(Meta(Self::from(s.into_string()), meta)),
			unexpected => Err(Meta(
				InvalidContext::Unexpected(unexpected.kind(), &[json_syntax::Kind::String]),
				meta,
			)),
		}
	}
}

impl<M> TryFromJson<M> for Container<M> {
	type Error = InvalidContext;

	fn try_from_json(
		value: Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			Meta(json_syntax::Value::Array(a), meta) => {
				let mut container = Vec::new();

				for item in a {
					container.push(ContainerType::try_from_json(item)?)
				}

				Ok(Meta(Self::Many(container), meta))
			}
			other => ContainerType::try_from_json(other).map(Meta::cast),
		}
	}
}

impl<M> TryFromJson<M> for ContainerType {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::String(s) => match ContainerType::try_from(s.as_str()) {
				Ok(t) => Ok(Meta(t, meta)),
				Err(_) => Err(Meta(InvalidContext::InvalidTermDefinition, meta)),
			},
			unexpected => Err(Meta(
				InvalidContext::Unexpected(unexpected.kind(), &[json_syntax::Kind::String]),
				meta,
			)),
		}
	}
}

impl<M: Clone> TryFromJson<M> for ContextEntry<M> {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::Array(a) => {
				let mut many = Vec::with_capacity(a.len());

				for item in a {
					many.push(Context::try_from_json(item)?)
				}

				Ok(Meta(Self::Many(many), meta))
			}
			context => Ok(Meta(
				Self::One(Context::try_from_json(Meta(context, meta.clone()))?),
				meta,
			)),
		}
	}
}

impl<M: Clone> TryFromJson<M> for Context<M> {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::Null => Ok(Meta(Self::Null, meta)),
			json_syntax::Value::String(s) => match IriRefBuf::new(&s) {
				Ok(iri_ref) => Ok(Meta(Self::IriRef(iri_ref), meta)),
				Err(e) => Err(Meta(InvalidContext::InvalidIriRef(e), meta)),
			},
			json_syntax::Value::Object(o) => {
				let mut def = ContextDefinition::new();

				for json_syntax::object::Entry {
					key: Meta(key, key_metadata),
					value,
				} in o
				{
					match Keyword::try_from(key.as_str()) {
						Ok(Keyword::Base) => def.base = Some(Nullable::try_from_json(value)?),
						Ok(Keyword::Import) => def.import = Some(IriRefBuf::try_from_json(value)?),
						Ok(Keyword::Language) => {
							def.language = Some(Nullable::try_from_json(value)?)
						}
						Ok(Keyword::Direction) => {
							def.direction = Some(Nullable::try_from_json(value)?)
						}
						Ok(Keyword::Propagate) => {
							def.propagate = Some(bool::try_from_json(value).map_err(Meta::cast)?)
						}
						Ok(Keyword::Protected) => {
							def.protected = Some(bool::try_from_json(value).map_err(Meta::cast)?)
						}
						Ok(Keyword::Type) => def.type_ = Some(ContextType::try_from_json(value)?),
						Ok(Keyword::Version) => def.version = Some(Version::try_from_json(value)?),
						Ok(Keyword::Vocab) => def.vocab = Some(Nullable::try_from_json(value)?),
						_ => {
							let term_def = match value {
								Meta(json_syntax::Value::Null, meta) => Meta(Nullable::Null, meta),
								other => TermDefinition::try_from_json(other)?.map(Nullable::Some),
							};

							if let Some(binding) = def
								.bindings
								.insert(Meta(key.into(), key_metadata), term_def)
							{
								return Err(Meta(
									InvalidContext::DuplicateKey,
									binding.key_metadata,
								));
							}
						}
					}
				}

				Ok(Meta(Self::Definition(def), meta))
			}
			unexpected => Err(Meta(
				InvalidContext::Unexpected(
					unexpected.kind(),
					&[
						json_syntax::Kind::Null,
						json_syntax::Kind::String,
						json_syntax::Kind::Object,
					],
				),
				meta,
			)),
		}
	}
}

impl<M: Clone> TryFromJson<M> for TermDefinition<M> {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::String(s) => match BlankIdBuf::new(s.to_string()) {
				Ok(b) => Ok(Meta(Self::Blank(b), meta)),
				Err(rdf_types::InvalidBlankId(s)) => match CompactIriBuf::new(s) {
					Ok(s) => Ok(Meta(Self::CompactIri(s), meta)),
					Err(crate::InvalidCompactIri(k)) => match IriBuf::from_string(k) {
						Ok(iri) => Ok(Meta(Self::Iri(iri), meta)),
						Err(_) => Err(Meta(InvalidContext::InvalidTermDefinition, meta)),
					},
				},
			},
			json_syntax::Value::Object(o) => {
				let mut def = ExpandedTermDefinition::new();

				for json_syntax::object::Entry {
					key: Meta(key, key_metadata),
					value,
				} in o
				{
					match Keyword::try_from(key.as_str()) {
						Ok(Keyword::Id) => def.id = Some(Nullable::try_from_json(value)?),
						Ok(Keyword::Type) => def.type_ = Some(Nullable::try_from_json(value)?),
						Ok(Keyword::Context) => {
							def.context = Some(Box::new(ContextEntry::try_from_json(value)?))
						}
						Ok(Keyword::Reverse) => def.reverse = Some(Key::try_from_json(value)?),
						Ok(Keyword::Index) => def.index = Some(Index::try_from_json(value)?),
						Ok(Keyword::Language) => {
							def.language = Some(Nullable::try_from_json(value)?)
						}
						Ok(Keyword::Direction) => {
							def.direction = Some(Nullable::try_from_json(value)?)
						}
						Ok(Keyword::Container) => {
							let container = match value {
								Meta(json_syntax::Value::Null, meta) => Meta(Nullable::Null, meta),
								other => {
									let Meta(container, meta) = Container::try_from_json(other)?;
									Meta(Nullable::Some(container), meta)
								}
							};

							def.container = Some(container)
						}
						Ok(Keyword::Nest) => def.nest = Some(Nest::try_from_json(value)?),
						Ok(Keyword::Prefix) => {
							def.prefix = Some(bool::try_from_json(value).map_err(Meta::cast)?)
						}
						Ok(Keyword::Propagate) => {
							def.propagate = Some(bool::try_from_json(value).map_err(Meta::cast)?)
						}
						Ok(Keyword::Protected) => {
							def.protected = Some(bool::try_from_json(value).map_err(Meta::cast)?)
						}
						_ => return Err(Meta(InvalidContext::InvalidTermDefinition, key_metadata)),
					}
				}

				Ok(Meta(Self::Expanded(def), meta))
			}
			unexpected => Err(Meta(
				InvalidContext::Unexpected(
					unexpected.kind(),
					&[json_syntax::Kind::String, json_syntax::Kind::Object],
				),
				meta,
			)),
		}
	}
}

pub enum ContextComponentRef<'a, C: AnyContextEntry> {
	ContextArray,
	Context(ContextRef<'a, C::Definition>),
	ContextEntry(ValueRef<'a, C>),
	ExpandedTermDefinition,
	ExpandedTermDefinitionEntry(TermDefinitionEntryRef<'a, C>),
	ExpandedTermDefinitionContainer(&'a Meta<ContainerType, C::Metadata>),
}

impl<'a, C: AnyContextEntry> ContextComponentRef<'a, C> {
	pub fn is_array(&self) -> bool {
		match self {
			Self::ContextArray => true,
			Self::ExpandedTermDefinitionEntry(e) => e.is_array(),
			_ => false,
		}
	}

	pub fn is_object(&self) -> bool {
		match self {
			Self::ContextArray => false,
			Self::Context(c) => c.is_object(),
			Self::ContextEntry(c) => c.is_object(),
			Self::ExpandedTermDefinition => true,
			Self::ExpandedTermDefinitionEntry(e) => e.is_object(),
			Self::ExpandedTermDefinitionContainer(_) => false,
		}
	}
}
