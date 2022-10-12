use contextual::{DisplayWithContext, WithContext};

pub trait Handler<N, W> {
	fn handle(&mut self, vocabulary: &N, warning: W);
}

impl<N, W> Handler<N, W> for () {
	fn handle(&mut self, _vocabulary: &N, _warning: W) {}
}

impl<'a, N, W, H: Handler<N, W>> Handler<N, W> for &'a mut H {
	fn handle(&mut self, vocabulary: &N, warning: W) {
		H::handle(*self, vocabulary, warning)
	}
}

pub struct Print;

impl<N, W: std::fmt::Display> Handler<N, W> for Print {
	fn handle(&mut self, _vocabulary: &N, warning: W) {
		eprintln!("{}", warning)
	}
}

pub struct PrintWith;

impl<N, W: DisplayWithContext<N>> Handler<N, W> for PrintWith {
	fn handle(&mut self, vocabulary: &N, warning: W) {
		eprintln!("{}", warning.with(vocabulary))
	}
}
