#![feature(generic_associated_types)]

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
// pub use document::*;

// /// Entry of a map in a JSON-LD document.
// ///
// /// This object stores the location of the key, without storing the key itself.
// pub struct Entry<T, S, P>(Location<S, P>, Loc<T, S, P>);

// impl<T, S, P> Entry<T, S, P> {
// 	pub fn key_location(&self) -> &Location<S, P> {
// 		&self.0
// 	}

// 	pub fn value(&self) -> &Loc<T, S, P> {
// 		&self.1
// 	}
// }
