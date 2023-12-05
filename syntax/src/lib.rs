//! This library provide functions to parse JSON-LD contexts
//! and print JSON-LD documents.
mod compact_iri;
mod compare;
pub mod container;
pub mod context;
mod direction;
mod entry;
mod error;
mod expandable;
mod into_json;
mod keyword;
mod lang;
mod nullable;
#[allow(hidden_glob_reexports)] // Fixed in the next version.
mod print;
mod try_from_json;

pub use compact_iri::*;
pub use compare::*;
pub use container::{Container, ContainerKind, ContainerRef};
pub use context::{Context, ContextRef};
pub use direction::*;
pub use entry::Entry;
pub use error::*;
pub use expandable::*;
pub use into_json::*;
pub use json_syntax::*;
pub use keyword::*;
pub use lang::*;
pub use nullable::*;
pub use try_from_json::*;

#[derive(Clone, Copy, Debug)]
pub struct Unexpected(json_syntax::Kind, &'static [json_syntax::Kind]);
