//! JSON-LD core types.
pub use crate::syntax::{Direction, LenientLangTag, LenientLangTagBuf, Nullable};

pub mod context;
mod document;
pub mod id;
mod indexed;
mod lang_string;
pub mod loader;
pub mod object;
mod processing_mode;
// pub mod quad;
// pub mod rdf;
// mod serialization;
mod term;
mod ty;
pub mod utils;
pub mod warning;

pub use context::ProcessedContext;
pub use document::*;
pub use id::*;
pub use indexed::*;
pub use lang_string::*;
pub use loader::*;
pub use object::{IndexedNode, IndexedObject, NodeObject, Nodes, Object, Objects, ValueObject};
pub use processing_mode::*;
// pub use quad::LdQuads;
// pub use rdf::RdfQuads;
pub use term::*;
pub use ty::*;

// pub struct Environment<'a, N, L, W> {
// 	pub vocabulary: &'a mut N,
// 	pub loader: &'a L,
// 	pub warnings: &'a mut W,
// }
