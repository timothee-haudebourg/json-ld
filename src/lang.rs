use crate::Direction;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LangString {
	data: String,
	language: Option<String>,
	direction: Option<Direction>
}

#[derive(Clone, Copy, Debug)]
pub struct InvalidLangString;

impl LangString {
	pub fn new(str: String, language: Option<String>, direction: Option<Direction>) -> Result<LangString, String> {
		if language.is_some() || direction.is_some() {
			Ok(LangString {
				data: str,
				language: language,
				direction: direction
			})
		} else {
			Err(str)
		}
	}

	pub fn as_str(&self) -> &str {
		self.data.as_str()
	}

	pub fn language(&self) -> Option<&String> {
		self.language.as_ref()
	}

	pub fn set_language(&mut self, language: Option<String>) -> Result<(), InvalidLangString> {
		if self.direction.is_some() || language.is_some() {
			self.language = language;
			Ok(())
		} else {
			Err(InvalidLangString)
		}
	}

	pub fn direction(&self) -> Option<Direction> {
		self.direction
	}

	pub fn set_direction(&mut self, direction: Option<Direction>) -> Result<(), InvalidLangString> {
		if direction.is_some() || self.language.is_some() {
			self.direction = direction;
			Ok(())
		} else {
			Err(InvalidLangString)
		}
	}

	pub fn set(&mut self, language: Option<String>, direction: Option<Direction>) -> Result<(), InvalidLangString> {
		if direction.is_some() || language.is_some() {
			self.language = language;
			self.direction = direction;
			Ok(())
		} else {
			Err(InvalidLangString)
		}
	}
}
