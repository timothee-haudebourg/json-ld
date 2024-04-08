use std::str::FromStr;

use hashbrown::HashMap;
use mime::Mime;
use reqwest::header::HeaderValue;

pub struct ContentType {
	media_type: Mime,
	params: HashMap<Vec<u8>, Vec<u8>>,
}

impl ContentType {
	pub fn new(value: &HeaderValue) -> Option<Self> {
		enum State {
			Mime,
			NextParam,
			BeginKey,
			Key,
			BeginValue,
			QuotedValue,
			Value,
		}

		let mut state = State::Mime;
		let mut mime = Vec::new();
		let mut current_key = Vec::new();
		let mut current_value = Vec::new();
		let mut params = HashMap::new();

		let mut bytes = value.as_bytes().iter();

		loop {
			match state {
				State::Mime => match bytes.next().copied() {
					Some(b';') => state = State::BeginKey,
					Some(b) => {
						mime.push(b);
					}
					None => break,
				},
				State::NextParam => match bytes.next().copied() {
					Some(b';') => state = State::BeginKey,
					Some(_) => return None,
					None => break,
				},
				State::BeginKey => match bytes.next().copied() {
					Some(b' ') => (),
					Some(b) => {
						current_key.push(b);
						state = State::Key
					}
					None => return None,
				},
				State::Key => match bytes.next().copied() {
					Some(b'=') => state = State::BeginValue,
					Some(b) => current_key.push(b),
					None => return None,
				},
				State::BeginValue => match bytes.next().copied() {
					Some(b'"') => state = State::QuotedValue,
					Some(b) => {
						state = State::Value;
						current_value.push(b);
					}
					_ => return None,
				},
				State::QuotedValue => match bytes.next().copied() {
					Some(b'"') => {
						params.insert(
							std::mem::take(&mut current_key),
							std::mem::take(&mut current_value),
						);

						state = State::NextParam
					}
					Some(b) => current_value.push(b),
					None => return None,
				},
				State::Value => match bytes.next().copied() {
					Some(b';') => {
						params.insert(
							std::mem::take(&mut current_key),
							std::mem::take(&mut current_value),
						);

						state = State::BeginKey
					}
					Some(b) => current_value.push(b),
					None => {
						params.insert(
							std::mem::take(&mut current_key),
							std::mem::take(&mut current_value),
						);
						break;
					}
				},
			}
		}

		match Mime::from_str(std::str::from_utf8(&mime).ok()?) {
			Ok(media_type) => Some(Self { media_type, params }),
			Err(_) => None,
		}
	}

	pub fn is_json_ld(&self) -> bool {
		self.media_type == "application/json" || self.media_type == "application/ld+json"
	}

	pub fn media_type(&self) -> &Mime {
		&self.media_type
	}

	pub fn into_media_type(self) -> Mime {
		self.media_type
	}

	pub fn profile(&self) -> Option<&[u8]> {
		self.params.get(b"profile".as_slice()).map(Vec::as_slice)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_content_type_1() {
		let content_type = ContentType::new(
			&HeaderValue::from_str(
				"application/ld+json;profile=http://www.w3.org/ns/json-ld#expanded",
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(*content_type.media_type(), "application/ld+json");
		assert_eq!(
			content_type.profile(),
			Some(b"http://www.w3.org/ns/json-ld#expanded".as_slice())
		)
	}

	#[test]
	fn parse_content_type_2() {
		let content_type = ContentType::new(
			&HeaderValue::from_str(
				"application/ld+json; profile=http://www.w3.org/ns/json-ld#expanded",
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(*content_type.media_type(), "application/ld+json");
		assert_eq!(
			content_type.profile(),
			Some(b"http://www.w3.org/ns/json-ld#expanded".as_slice())
		)
	}

	#[test]
	fn parse_content_type_3() {
		let content_type = ContentType::new(
			&HeaderValue::from_str(
				"application/ld+json; profile=http://www.w3.org/ns/json-ld#expanded; q=1",
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(*content_type.media_type(), "application/ld+json");
		assert_eq!(
			content_type.profile(),
			Some(b"http://www.w3.org/ns/json-ld#expanded".as_slice())
		)
	}

	#[test]
	fn parse_content_type_4() {
		let content_type = ContentType::new(
			&HeaderValue::from_str(
				"application/ld+json; profile=\"http://www.w3.org/ns/json-ld#expanded\"; q=1",
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(*content_type.media_type(), "application/ld+json");
		assert_eq!(
			content_type.profile(),
			Some(b"http://www.w3.org/ns/json-ld#expanded".as_slice())
		)
	}

	#[test]
	fn parse_content_type_5() {
		let content_type = ContentType::new(
			&HeaderValue::from_str(
				"application/ld+json; profile=\"http://www.w3.org/ns/json-ld#expanded\"",
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(*content_type.media_type(), "application/ld+json");
		assert_eq!(
			content_type.profile(),
			Some(b"http://www.w3.org/ns/json-ld#expanded".as_slice())
		)
	}

	#[test]
	fn parse_content_type_6() {
		let content_type = ContentType::new(
			&HeaderValue::from_str(
				"application/ld+json;profile=\"http://www.w3.org/ns/json-ld#expanded\"; q=1",
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(*content_type.media_type(), "application/ld+json");
		assert_eq!(
			content_type.profile(),
			Some(b"http://www.w3.org/ns/json-ld#expanded".as_slice())
		)
	}

	#[test]
	fn parse_content_type_7() {
		let content_type = ContentType::new(&HeaderValue::from_str("application/ld+json; profile=\"http://www.w3.org/ns/json-ld#flattened http://www.w3.org/ns/json-ld#compacted\"; q=1").unwrap()).unwrap();
		assert_eq!(*content_type.media_type(), "application/ld+json");
		assert_eq!(
			content_type.profile(),
			Some(
				b"http://www.w3.org/ns/json-ld#flattened http://www.w3.org/ns/json-ld#compacted"
					.as_slice()
			)
		)
	}

	#[test]
	fn parse_content_type_8() {
		let content_type =
			ContentType::new(&HeaderValue::from_str("application/ld+json").unwrap()).unwrap();
		assert_eq!(*content_type.media_type(), "application/ld+json");
	}
}
