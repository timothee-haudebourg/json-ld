/// ProcessingMode
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ProcessingMode {
	/// JSON-LD 1.0.
	JsonLd1_0,

	/// JSON-LD 1.1.
	JsonLd1_1
}

impl Default for ProcessingMode {
	fn default() -> ProcessingMode {
		ProcessingMode::JsonLd1_1
	}
}
