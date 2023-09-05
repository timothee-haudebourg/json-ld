use iref::IriBuf;
use json_ld_core::Context;
use locspan::Meta;
use rdf_types::BlankIdBuf;
use std::ops;

/// Processed context that also borrows the original, unprocessed, context.
pub struct Processed<'l, T = IriBuf, B = BlankIdBuf, M = ()> {
	unprocessed: Meta<&'l json_ld_syntax::context::Context<M>, &'l M>,
	processed: Context<T, B, M>,
}

impl<'l, T, B, M> Processed<'l, T, B, M> {
	pub(crate) fn new(
		unprocessed: Meta<&'l json_ld_syntax::context::Context<M>, &'l M>,
		processed: Context<T, B, M>,
	) -> Self {
		Self {
			unprocessed,
			processed,
		}
	}

	pub fn unprocessed(&self) -> Meta<&'l json_ld_syntax::context::Context<M>, &'l M> {
		self.unprocessed
	}

	pub fn into_processed(self) -> Context<T, B, M> {
		self.processed
	}

	pub fn as_ref(&self) -> ProcessedRef<'l, '_, T, B, M> {
		ProcessedRef {
			unprocessed: self.unprocessed,
			processed: &self.processed,
		}
	}

	pub fn into_owned(self) -> ProcessedOwned<T, B, M>
	where
		M: Clone,
	{
		ProcessedOwned {
			unprocessed: self.unprocessed.cloned(),
			processed: self.processed,
		}
	}
}

impl<'l, T, B, M> ops::Deref for Processed<'l, T, B, M> {
	type Target = Context<T, B, M>;

	fn deref(&self) -> &Self::Target {
		&self.processed
	}
}

impl<'l, T, B, M> ops::DerefMut for Processed<'l, T, B, M> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.processed
	}
}

/// Reference to a processed context that also borrows the original, unprocessed, context.
pub struct ProcessedRef<'l, 'a, T, B, M> {
	unprocessed: Meta<&'l json_ld_syntax::context::Context<M>, &'l M>,
	processed: &'a Context<T, B, M>,
}

impl<'l, 'a, T, B, M> ProcessedRef<'l, 'a, T, B, M> {
	pub fn unprocessed(&self) -> Meta<&'l json_ld_syntax::context::Context<M>, &'l M> {
		self.unprocessed
	}

	pub fn processed(&self) -> &'a Context<T, B, M> {
		self.processed
	}
}

/// Processed context that also owns the original, unprocessed, context.
pub struct ProcessedOwned<T, B, M> {
	unprocessed: Meta<json_ld_syntax::context::Context<M>, M>,
	processed: Context<T, B, M>,
}

impl<T, B, M> ProcessedOwned<T, B, M> {
	pub fn unprocessed(&self) -> &Meta<json_ld_syntax::context::Context<M>, M> {
		&self.unprocessed
	}

	pub fn processed(&self) -> &Context<T, B, M> {
		&self.processed
	}

	pub fn as_ref(&self) -> ProcessedRef<T, B, M> {
		ProcessedRef {
			unprocessed: self.unprocessed.borrow(),
			processed: &self.processed,
		}
	}
}
