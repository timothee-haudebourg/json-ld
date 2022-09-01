use contextual::{DisplayWithContext, WithContext};

pub trait Handler<N, W> {
	fn handle(&mut self, vocabulary: &N, warning: W);
}

impl<N, W, F> Handler<N, W> for F
where
	F: FnMut(&N, W),
{
	fn handle(&mut self, vocabulary: &N, warning: W) {
		(*self)(vocabulary, warning)
	}
}

impl<N, W> Handler<N, W> for () {
	fn handle(&mut self, _namespace: &N, _warning: W) {}
}

pub fn print<N, W: std::fmt::Display>(_namespace: &N, warning: W) {
	eprintln!("{}", warning)
}

pub fn print_in<N, W: DisplayWithContext<N>>(vocabulary: &N, warning: W) {
	eprintln!("{}", warning.with(vocabulary))
}
