use crate::{BorrowWithNamespace, DisplayWithNamespace};

pub trait Handler<N, W> {
	fn handle(&mut self, namespace: &N, warning: W);
}

impl<N, W, F> Handler<N, W> for F
where
	F: FnMut(&N, W),
{
	fn handle(&mut self, namespace: &N, warning: W) {
		(*self)(namespace, warning)
	}
}

impl<N, W> Handler<N, W> for () {
	fn handle(&mut self, _namespace: &N, _warning: W) {}
}

pub fn print<N, W: std::fmt::Display>(_namespace: &N, warning: W) {
	eprintln!("{}", warning)
}

pub fn print_in<N, W: DisplayWithNamespace<N>>(namespace: &N, warning: W) {
	eprintln!("{}", warning.with_namespace(namespace))
}
