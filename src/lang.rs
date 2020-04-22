use crate::Direction;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LangString {
	data: String,
	language: Option<String>,
	direction: Option<Direction>
}

impl LangString {
	pub fn new(str: String, language: Option<String>, direction: Option<Direction>) -> LangString {
		LangString {
			data: str,
			language: language,
			direction: direction
		}
	}

	pub fn as_str(&self) -> &str {
		self.data.as_str()
	}

	pub fn language(&self) -> Option<&str> {
		match &self.language {
			Some(lang) => Some(lang.as_str()),
			None => None
		}
	}

	pub fn set_language(&mut self, language: Option<String>) {
		self.language = language
	}

	pub fn direction(&self) -> Option<Direction> {
		self.direction
	}

	pub fn set_direction(&mut self, direction: Option<Direction>) {
		self.direction = direction
	}
}
