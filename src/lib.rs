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

use std::fmt;
use std::hash::Hash;
use std::convert::TryFrom;
use iref::{Iri, IriBuf};
pub use error::*;
pub use keyword::*;
pub use direction::*;
pub use container::*;
pub use key::*;
pub use property::*;
pub use literal::*;
pub use value::*;
pub use node::*;
pub use object::*;
pub use vocab::*;
pub use expansion::expand;
pub use pp::*;

use json::JsonValue;

pub(crate) fn as_array(json: &JsonValue) -> &[JsonValue] {
	match json {
		JsonValue::Array(ary) => ary,
		_ => unsafe { std::mem::transmute::<&JsonValue, &[JsonValue; 1]>(json) as &[JsonValue] }
	}
}

pub trait Id: Clone + PartialEq + Eq + Hash + fmt::Display {
	fn from_iri(iri: Iri) -> Self;

	fn iri(&self) -> Iri;
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BlankId(String);

impl BlankId {
	pub fn new(name: &str) -> BlankId {
		BlankId("_:".to_string() + name)
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}

	pub fn name(&self) -> &str {
		&self.0[2..self.0.len()]
	}
}

impl<'a> TryFrom<&'a str> for BlankId {
	type Error = ();

	fn try_from(str: &'a str) -> Result<BlankId, ()> {
		if let Some(name) = str.strip_prefix("_:") {
			Ok(BlankId::new(name))
		} else {
			Err(())
		}
	}
}

impl fmt::Display for BlankId {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl Id for IriBuf {
	fn from_iri(iri: Iri) -> IriBuf {
		iri.into()
	}

	fn iri(&self) -> Iri {
		self.as_iri()
	}
}
