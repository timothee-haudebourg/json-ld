use std::hash::Hash;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Import;

impl Import {
	pub fn into_str(self) -> &'static str {
		"@import"
	}
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for Import {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.into_str().hash(state)
	}
}
