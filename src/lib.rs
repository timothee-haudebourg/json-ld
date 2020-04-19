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
mod id;
mod blank;
mod key;
mod typ;
mod property;
mod container;
mod literal;
mod value;
mod node;
mod object;
mod vocab;
mod context;
pub mod expansion;
mod pp;

pub use mode::*;
pub use error::*;
pub use keyword::*;
pub use direction::*;
pub use container::*;
pub use id::*;
pub use blank::*;
pub use key::*;
pub use typ::*;
pub use property::*;
pub use literal::*;
pub use value::*;
pub use node::*;
pub use object::*;
pub use vocab::*;
pub use context::*;
pub use expansion::{expand, ExpansionOptions};
pub use pp::*;
pub use util::{AsJson, json_ld_eq};
