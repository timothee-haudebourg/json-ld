use contextual::{DisplayWithContext, WithContext};

/// Warning handler.
///
/// This trait is implemented by the unit type `()` which ignores warnings.
/// You can use [`Print`] or [`PrintWith`] to print warnings on the standard
/// output or implement your own handler.
pub trait Handler<N, W> {
	/// Handle a warning with the given `vocabulary`.
	fn handle(&mut self, vocabulary: &N, warning: W);
}

impl<N, W> Handler<N, W> for () {
	fn handle(&mut self, _vocabulary: &N, _warning: W) {}
}

impl<N, W, H: Handler<N, W>> Handler<N, W> for &mut H {
	fn handle(&mut self, vocabulary: &N, warning: W) {
		H::handle(*self, vocabulary, warning)
	}
}

/// Prints warnings that can be displayed without vocabulary on the standard
/// output.
pub struct Print;

impl<N, W: std::fmt::Display> Handler<N, W> for Print {
	fn handle(&mut self, _vocabulary: &N, warning: W) {
		eprintln!("{warning}")
	}
}

/// Prints warnings with a given vocabulary on the standard output.
pub struct PrintWith;

impl<N, W: DisplayWithContext<N>> Handler<N, W> for PrintWith {
	fn handle(&mut self, vocabulary: &N, warning: W) {
		eprintln!("{}", warning.with(vocabulary))
	}
}
