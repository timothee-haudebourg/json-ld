//! This library provide functions to parse JSON-LD contexts
//! and print JSON-LD documents.
mod compact_iri;
mod compare;
pub mod container;
pub mod context;
mod direction;
mod error;
mod expandable;
mod into_json;
mod keyword;
mod lang;
mod nullable;
mod print_ld;
mod try_from_json;
mod utils;

pub use compact_iri::*;
pub use compare::*;
pub use container::{Container, ContainerKind};
pub use context::{Context, ContextDocument, ContextEntry};
pub use direction::*;
pub use error::*;
pub use expandable::*;
pub use into_json::*;
pub use json_syntax::{
	object, parse, print, BorrowUnordered, Kind, Number, NumberBuf, Object, Parse, Print, String,
	Unordered, UnorderedEq, UnorderedHash, UnorderedPartialEq, Value,
};
pub use keyword::*;
pub use lang::*;
pub use nullable::*;
pub use try_from_json::*;

#[cfg(feature = "serde")]
pub use json_syntax::{from_value, to_value};

#[derive(Clone, Copy, Debug)]
pub struct Unexpected(json_syntax::Kind, &'static [json_syntax::Kind]);
