//! Custom futures for `json-ld` functions.
//!
//! This modules defines futures used in this library. In particular it defines
//! a non-`Send` variant of the `futures::future::BoxFuture` type enabled when
//! compiling for `wasm32`.

#[cfg(target_arch = "wasm32")]
mod wasm32 {
	use std::{future::Future, pin::Pin};
	pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

	pub trait FutureExt<'a>: Future {
		fn boxed(self) -> BoxFuture<'a, Self::Output>;
	}

	impl<'a, F: Future + Sized + 'a> FutureExt<'a> for F {
		fn boxed(self) -> BoxFuture<'a, Self::Output> {
			Box::pin(self)
		}
	}
}

#[cfg(target_arch = "wasm32")]
pub use wasm32::{BoxFuture, FutureExt};

#[cfg(not(target_arch = "wasm32"))]
pub use futures::future::{BoxFuture, FutureExt};
