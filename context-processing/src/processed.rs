use json_ld_core::Context;
use locspan::Meta;
use std::ops;

/// Processed context that also borrows the original, unprocessed, context.
pub struct Processed<'l, T, B, C, M> {
	unprocessed: Meta<&'l C, &'l M>,
	processed: Context<T, B, C, M>,
}

impl<'l, T, B, C, M> Processed<'l, T, B, C, M> {
	pub(crate) fn new(unprocessed: Meta<&'l C, &'l M>, processed: Context<T, B, C, M>) -> Self {
		Self {
			unprocessed,
			processed,
		}
	}

	pub fn unprocessed(&self) -> Meta<&'l C, &'l M> {
		self.unprocessed
	}

	pub fn into_processed(self) -> Context<T, B, C, M> {
		self.processed
	}

	pub fn as_ref(&self) -> ProcessedRef<'l, '_, T, B, C, M> {
		ProcessedRef {
			unprocessed: self.unprocessed,
			processed: &self.processed,
		}
	}

	pub fn into_owned(self) -> ProcessedOwned<T, B, C, M>
	where
		C: Clone,
		M: Clone,
	{
		ProcessedOwned {
			unprocessed: self.unprocessed.cloned(),
			processed: self.processed,
		}
	}
}

impl<'l, T, B, C, M> ops::Deref for Processed<'l, T, B, C, M> {
	type Target = Context<T, B, C, M>;

	fn deref(&self) -> &Self::Target {
		&self.processed
	}
}

impl<'l, T, B, C, M> ops::DerefMut for Processed<'l, T, B, C, M> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.processed
	}
}

/// Reference to a processed context that also borrows the original, unprocessed, context.
pub struct ProcessedRef<'l, 'a, T, B, C, M> {
	unprocessed: Meta<&'l C, &'l M>,
	processed: &'a Context<T, B, C, M>,
}

impl<'l, 'a, T, B, C, M> ProcessedRef<'l, 'a, T, B, C, M> {
	pub fn unprocessed(&self) -> Meta<&'l C, &'l M> {
		self.unprocessed
	}

	pub fn processed(&self) -> &'a Context<T, B, C, M> {
		self.processed
	}
}

/// Processed context that also owns the original, unprocessed, context.
pub struct ProcessedOwned<T, B, C, M> {
	unprocessed: Meta<C, M>,
	processed: Context<T, B, C, M>,
}

impl<T, B, C, M> ProcessedOwned<T, B, C, M> {
	pub fn unprocessed(&self) -> &Meta<C, M> {
		&self.unprocessed
	}

	pub fn processed(&self) -> &Context<T, B, C, M> {
		&self.processed
	}

	pub fn as_ref(&self) -> ProcessedRef<T, B, C, M> {
		ProcessedRef {
			unprocessed: self.unprocessed.borrow(),
			processed: &self.processed,
		}
	}
}
