#![feature(generic_associated_types)]
use locspan::Meta;

mod number;

pub use number::*;

mod compact_iri;
mod container;
pub mod context;
mod direction;
mod document;
mod expandable;
mod keyword;
mod lang;
mod nullable;
mod print;

pub use compact_iri::*;
pub use container::*;
pub use context::{
	AnyContextDefinition, AnyContextEntry, Context, ContextDefinition, ContextEntry,
	ContextEntryRef, ContextRef,
};
pub use direction::*;
pub use document::*;
pub use expandable::*;
pub use keyword::*;
pub use lang::*;
pub use nullable::*;

pub trait TryFromJson<M>: Sized {
	type Error;

	fn try_from_json(
		value: Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<Self::Error, M>>;
}

#[derive(Clone, Copy, Debug)]
pub struct Unexpected(json_syntax::Kind, &'static [json_syntax::Kind]);
