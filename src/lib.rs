#![feature(proc_macro_hygiene)]
#![feature(str_strip)]
#![feature(arbitrary_self_types)]

#[macro_use]
extern crate log;
extern crate json;
extern crate iref;
#[macro_use]
extern crate static_iref;

mod syntax;
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
pub mod object;
pub mod context;
pub mod expansion;
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

pub use object::Object;
pub use context::{
	Context,
	ContextMut,
	JsonContext
};
