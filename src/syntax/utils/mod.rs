use core::fmt;
use std::{
	borrow::Borrow,
	cmp::Ordering,
	hash::{Hash, Hasher},
	ops::Deref,
};

pub fn into_smallcase(c: char) -> char {
	c.to_lowercase().next().unwrap_or(c)
}

pub fn case_insensitive_eq(a: &str, b: &str) -> bool {
	a.chars()
		.map(into_smallcase)
		.eq(b.chars().map(into_smallcase))
}

pub fn case_insensitive_hash<H: Hasher>(s: &str, hasher: &mut H) {
	for c in s.chars().map(into_smallcase) {
		c.hash(hasher)
	}
}

pub fn case_insensitive_cmp(a: &str, b: &str) -> Ordering {
	a.chars()
		.map(into_smallcase)
		.cmp(b.chars().map(into_smallcase))
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[repr(transparent)]
pub struct CaselessStr(str);

impl CaselessStr {
	pub fn from_ref(str: &str) -> &Self {
		unsafe { std::mem::transmute(str) }
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl Deref for CaselessStr {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl ToOwned for CaselessStr {
	type Owned = CaselessString;

	fn to_owned(&self) -> Self::Owned {
		CaselessString(self.as_str().to_owned())
	}
}

impl PartialEq for CaselessStr {
	fn eq(&self, other: &Self) -> bool {
		case_insensitive_eq(self, other)
	}
}

impl Eq for CaselessStr {}

impl PartialOrd for CaselessStr {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for CaselessStr {
	fn cmp(&self, other: &Self) -> Ordering {
		case_insensitive_cmp(self, other)
	}
}

impl Hash for CaselessStr {
	fn hash<H: Hasher>(&self, state: &mut H) {
		case_insensitive_hash(self, state);
	}
}

impl fmt::Display for CaselessStr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl AsRef<str> for CaselessStr {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[repr(transparent)]
pub struct CaselessString(String);

impl CaselessString {
	pub fn as_caseless_str(&self) -> &CaselessStr {
		CaselessStr::from_ref(&self.0)
	}
}

impl Borrow<CaselessStr> for CaselessString {
	fn borrow(&self) -> &CaselessStr {
		self.as_caseless_str()
	}
}

impl Deref for CaselessString {
	type Target = CaselessStr;

	fn deref(&self) -> &Self::Target {
		self.as_caseless_str()
	}
}

impl PartialEq for CaselessString {
	fn eq(&self, other: &Self) -> bool {
		self.as_caseless_str().eq(other.as_caseless_str())
	}
}

impl Eq for CaselessString {}

impl PartialOrd for CaselessString {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for CaselessString {
	fn cmp(&self, other: &Self) -> Ordering {
		self.as_caseless_str().cmp(other.as_caseless_str())
	}
}

impl Hash for CaselessString {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_caseless_str().hash(state);
	}
}

impl fmt::Display for CaselessString {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.as_caseless_str().fmt(f)
	}
}

impl AsRef<str> for CaselessString {
	fn as_ref(&self) -> &str {
		self.as_caseless_str().as_ref()
	}
}

impl From<String> for CaselessString {
	fn from(value: String) -> Self {
		Self(value)
	}
}
