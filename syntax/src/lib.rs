#![feature(generic_associated_types)]
use std::convert::Infallible;
use std::fmt;

use locspan::{Loc, Location};

mod number;

pub use number::*;

mod compact_iri;
pub mod container;
pub mod context;
mod direction;
mod entry;
mod expandable;
mod into_json;
mod keyword;
mod lang;
mod nullable;
mod print;
mod try_from_json;
mod value;

pub use compact_iri::*;
pub use container::{Container, ContainerKind, ContainerRef};
pub use context::{Context, ContextRef};
pub use direction::*;
pub use entry::Entry;
pub use expandable::*;
pub use into_json::IntoJson;
pub use keyword::*;
pub use lang::*;
pub use nullable::*;
pub use try_from_json::*;
pub use value::*;

#[derive(Clone, Copy, Debug)]
pub struct Unexpected(json_syntax::Kind, &'static [json_syntax::Kind]);

#[derive(Debug)]
pub enum Error<F> {
	InvalidJson(json_syntax::parse::Error<Infallible, F>),
	InvalidJsonLd(ValueFromJsonError<Location<F>>),
}

impl<F> fmt::Display for Error<F> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::InvalidJson(e) => write!(f, "invalid JSON: {}", e),
			Self::InvalidJsonLd(e) => write!(f, "invalid JSON-LD: {}", e),
		}
	}
}

pub fn from_str_in<F: Clone>(
	file: F,
	s: &str,
) -> Result<Loc<Value<context::Value<Location<F>>, Location<F>>, F>, Loc<Error<F>, F>> {
	use decoded_char::DecodedChars;
	use json_syntax::Parse;
	use locspan::MapLocErr;
	let json = json_syntax::Value::parse(file, s.decoded_chars().map(Ok))
		.map_loc_err(Error::InvalidJson)?;
	Value::try_from_json(json).map_loc_err(Error::InvalidJsonLd)
}

pub fn from_str(
	s: &str,
) -> Result<Loc<Value<context::Value<Location<()>>, Location<()>>, ()>, Loc<Error<()>, ()>> {
	from_str_in((), s)
}
