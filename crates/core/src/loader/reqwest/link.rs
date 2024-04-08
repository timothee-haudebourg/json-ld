use hashbrown::HashMap;
use iref::{IriRef, IriRefBuf};
use reqwest::header::HeaderValue;

pub struct Link {
	href: IriRefBuf,
	params: HashMap<Vec<u8>, Vec<u8>>,
}

impl Link {
	pub fn new(value: &HeaderValue) -> Option<Self> {
		enum State {
			BeginHref,
			Href,
			NextParam,
			BeginKey,
			Key,
			BeginValue,
			Value,
		}

		let mut state = State::BeginHref;
		let mut href = Vec::new();
		let mut current_key = Vec::new();
		let mut current_value = Vec::new();
		let mut params = HashMap::new();

		let mut bytes = value.as_bytes().iter();

		loop {
			match state {
				State::BeginHref => match bytes.next().copied() {
					Some(b'<') => state = State::Href,
					_ => break None,
				},
				State::Href => match bytes.next().copied() {
					Some(b'>') => state = State::NextParam,
					Some(b) => {
						href.push(b);
					}
					None => break None,
				},
				State::NextParam => match bytes.next().copied() {
					Some(b';') => state = State::BeginKey,
					Some(_) => break None,
					None => {
						break match IriRefBuf::from_vec(href) {
							Ok(href) => Some(Self { href, params }),
							Err(_) => None,
						}
					}
				},
				State::BeginKey => match bytes.next().copied() {
					Some(b' ') => (),
					Some(b) => {
						current_key.push(b);
						state = State::Key
					}
					None => break None,
				},
				State::Key => match bytes.next().copied() {
					Some(b'=') => state = State::BeginValue,
					Some(b) => current_key.push(b),
					None => break None,
				},
				State::BeginValue => match bytes.next().copied() {
					Some(b'"') => state = State::Value,
					_ => break None,
				},
				State::Value => match bytes.next().copied() {
					Some(b'"') => {
						params.insert(
							std::mem::take(&mut current_key),
							std::mem::take(&mut current_value),
						);

						state = State::NextParam
					}
					Some(b) => current_value.push(b),
					None => break None,
				},
			}
		}
	}

	pub fn href(&self) -> &IriRef {
		self.href.as_iri_ref()
	}

	pub fn rel(&self) -> Option<&[u8]> {
		self.params.get(b"rel".as_slice()).map(Vec::as_slice)
	}

	pub fn type_(&self) -> Option<&[u8]> {
		self.params.get(b"type".as_slice()).map(Vec::as_slice)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_link_1() {
		let link = Link::new(
			&HeaderValue::from_str(
				"<http://www.example.org/context>; rel=\"context\"; type=\"application/ld+json\"",
			)
			.unwrap(),
		)
		.unwrap();
		assert_eq!(link.href(), "http://www.example.org/context");
		assert_eq!(link.rel(), Some(b"context".as_slice()));
		assert_eq!(link.type_(), Some(b"application/ld+json".as_slice()))
	}

	#[test]
	fn parse_link_2() {
		let link = Link::new(&HeaderValue::from_str("<http://www.example.org/context>; rel=\"context\"; type=\"application/ld+json\"; foo=\"bar\"").unwrap()).unwrap();
		assert_eq!(link.href(), "http://www.example.org/context");
		assert_eq!(link.rel(), Some(b"context".as_slice()));
		assert_eq!(link.type_(), Some(b"application/ld+json".as_slice()))
	}

	#[test]
	fn parse_link_3() {
		let link =
			Link::new(&HeaderValue::from_str("<http://www.example.org/context>").unwrap()).unwrap();
		assert_eq!(link.href(), "http://www.example.org/context")
	}
}
