#![allow(clippy::derive_hash_xor_eq)]
#![feature(generic_associated_types)]
#![feature(trait_alias)]

pub use json_ld_syntax::{Direction, LenientLanguageTag, LenientLanguageTagBuf, Nullable};

mod container;
pub mod context;
mod document;
mod error;
pub mod flattening;
pub mod id;
mod indexed;
mod lang_string;
pub mod loader;
mod mode;
pub mod namespace;
pub mod object;
pub mod print;
pub mod quad;
pub mod rdf;
mod reference;
mod term;
mod ty;
pub mod utils;
pub mod warning;

pub use container::Container;
pub use context::Context;
pub use document::*;
pub use error::*;
pub use id::*;
pub use indexed::*;
pub use lang_string::*;
pub use loader::*;
pub use mode::*;
pub use namespace::{
	BlankIdNamespace, BlankIdNamespaceMut, BorrowWithNamespace, DisplayWithNamespace,
	IndexNamespace, IriNamespace, IriNamespaceMut, Namespace, NamespaceMut,
};
pub use object::{
	Node, Nodes, Object, Objects, StrippedIndexedNode, StrippedIndexedObject, TryFromJson, Value,
};
pub use print::Print;
pub use reference::*;
pub use term::*;
pub use ty::*;
