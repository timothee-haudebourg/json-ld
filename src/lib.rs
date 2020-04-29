#![feature(str_strip)]
#![feature(arbitrary_self_types)]

#[macro_use]
extern crate log;
extern crate json;
extern crate iref;

mod mode;
mod error;
mod direction;
mod lang;
mod id;
mod blank;
mod reference;
mod lenient;
mod indexed;
mod vocab;
mod document;
mod loader;
pub mod syntax;
pub mod object;
pub mod context;
pub mod expansion;
pub mod compaction;
pub mod util;

#[cfg(feature="reqwest-loader")]
pub mod reqwest;

pub use mode::*;
pub use error::*;
pub use direction::*;
pub use lang::*;
pub use id::*;
pub use blank::*;
pub use reference::*;
pub use lenient::*;
pub use indexed::*;
pub use vocab::*;
pub use document::*;
pub use loader::*;

pub use object::{Object, Node, Value};
pub use context::{
	Context,
	ContextMut,
	JsonContext
};
