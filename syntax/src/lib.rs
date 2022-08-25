#![feature(generic_associated_types)]
use std::convert::Infallible;
use std::fmt;

use locspan::{Location, MapLocErr, Meta, Span};

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
pub enum Error<M> {
	InvalidJson(json_syntax::parse::Error<Infallible, M>),
	InvalidJsonLd(ValueFromJsonError<M>),
}

pub type MetaError<M> = Meta<Error<M>, M>;

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
) -> Result<MetaValue<Location<F>>, MetaError<Location<F>>> {
	use json_syntax::Parse;
	let json = json_syntax::Value::parse_str(s, |span| Location::new(file.clone(), span))
		.map_loc_err(Error::InvalidJson)?;
	Value::try_from_json(json).map_loc_err(Error::InvalidJsonLd)
}

pub fn from_str(s: &str) -> Result<MetaValue<Span>, MetaError<Span>> {
	use json_syntax::Parse;
	let json = json_syntax::Value::parse_str(s, |span| span).map_loc_err(Error::InvalidJson)?;
	Value::try_from_json(json).map_loc_err(Error::InvalidJsonLd)
}
