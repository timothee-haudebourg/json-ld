pub enum Error {
	ContextProcessing(json_ld_context_processing::Error),
	InvalidIndexValue,
	InvalidSetOrListObject
}

impl From<json_ld_context_processing::Error> for Error {
	fn from(e: json_ld_context_processing::Error) -> Self {
		Self::ContextProcessing(e)
	}
}