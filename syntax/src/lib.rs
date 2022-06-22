#![feature(generic_associated_types)]

mod keyword;
pub mod context;

pub use keyword::*;
pub use context::{
	ContextEntry,
	Context,
	ContextDefinition,
	AnyContextEntry,
	AnyContextDefinition,
	ContextEntryRef,
	ContextRef
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Nullable<T> {
	Null,
	Some(T)
}

impl<T> Nullable<T> {
	pub fn as_ref(&self) -> Nullable<&T> {
		match self {
			Self::Null => Nullable::Null,
			Self::Some(t) => Nullable::Some(t)
		}
	}

	pub fn as_deref(&self) -> Nullable<&T::Target> where T: std::ops::Deref {
		match self {
			Self::Null => Nullable::Null,
			Self::Some(t) => Nullable::Some(t)
		}
	}

	pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Nullable<U> {
		match self {
			Self::Null => Nullable::Null,
			Self::Some(t) => Nullable::Some(f(t))
		}
	}

	pub fn cast<U>(self) -> Nullable<U> where T: Into<U> {
		match self {
			Self::Null => Nullable::Null,
			Self::Some(t) => Nullable::Some(t.into())
		}
	}
}

#[derive(Clone, Copy)]
pub enum Direction {
	Null,
	Ltr,
	Rtl	
}