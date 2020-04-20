#![feature(proc_macro_hygiene)]
#![feature(str_strip)]

#[macro_use]
extern crate log;
extern crate json;
extern crate iref;
#[macro_use]
extern crate static_iref;

mod mode;
mod util;
mod error;
mod keyword;
mod direction;
mod lang;
mod id;
mod blank;
mod term;
mod property;
mod container;
pub mod object;
mod vocab;
mod context;
pub mod expansion;

pub use mode::*;
pub use error::*;
pub use keyword::*;
pub use direction::*;
pub use lang::*;
pub use container::*;
pub use id::*;
pub use blank::*;
pub use term::*;
pub use property::*;
pub use object::Object;
pub use vocab::*;
pub use context::*;
pub use expansion::{expand, ExpansionOptions};
pub use util::{AsJson, json_ld_eq};
