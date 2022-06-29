#![feature(generic_associated_types)]
use locspan::{Loc, Location};

mod json;
mod number;

pub use json::*;
pub use number::*;

mod compact_iri;
mod keyword;
mod container;
mod nullable;
mod direction;
mod lang;
mod expandable;
pub mod context;
mod document;

pub use compact_iri::*;
pub use keyword::*;
pub use container::*;
pub use nullable::*;
pub use direction::*;
pub use lang::*;
pub use expandable::*;
pub use context::{
	ContextEntry,
	Context,
	ContextDefinition,
	AnyContextEntry,
	AnyContextDefinition,
	ContextEntryRef,
	ContextRef
};
pub use document::*;

/// Entry of a map in a JSON-LD document.
/// 
/// This object stores the location of the key, without storing the key itself.
pub struct Entry<T, S, P>(Location<S, P>, Loc<T, S, P>);

impl<T, S, P> Entry<T, S, P> {
	pub fn key_location(&self) -> &Location<S, P> {
		&self.0
	}

	pub fn value(&self) -> &Loc<T, S, P> {
		&self.1
	}
}