#![feature(proc_macro_hygiene)]
#![feature(str_strip)]

#[macro_use]
extern crate log;
extern crate json;
extern crate iref;
#[macro_use]
extern crate static_iref;

mod util;
mod error;
mod keyword;
mod direction;
mod id;
mod blank;
mod key;
mod property;
mod container;
mod literal;
mod value;
mod node;
mod object;
mod vocab;
pub mod context;
pub mod expansion;
mod pp;

pub use error::*;
pub use keyword::*;
pub use direction::*;
pub use container::*;
pub use id::*;
pub use blank::*;
pub use key::*;
pub use property::*;
pub use literal::*;
pub use value::*;
pub use node::*;
pub use object::*;
pub use vocab::*;
pub use expansion::expand;
pub use pp::*;
pub use util::{AsJson, json_ld_eq};
