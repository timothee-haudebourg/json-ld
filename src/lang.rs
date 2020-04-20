use crate::Direction;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LangString {
	data: String,
	language: String,
	direction: Direction
}

impl LangString {
	pub fn as_str(&self) -> &str {
		self.data.as_str()
	}

	pub fn language(&self) -> &str {
		self.language.as_str()
	}

	pub fn direction(&self) -> Direction {
		self.direction
	}
}
