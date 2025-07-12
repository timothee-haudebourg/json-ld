use iref::IriBuf;
use json_ld_core::Context;
use rdf_types::BlankIdBuf;
use std::ops;

/// Processed context that also borrows the original, unprocessed, context.
pub struct Processed<'l, T = IriBuf, B = BlankIdBuf> {
	pub unprocessed: &'l json_ld_syntax::context::Context,
	pub processed: Context<T, B>,
}

impl<'l, T, B> Processed<'l, T, B> {
	pub fn new(
		unprocessed: &'l json_ld_syntax::context::Context,
		processed: Context<T, B>,
	) -> Self {
		Self {
			unprocessed,
			processed,
		}
	}

	pub fn unprocessed(&self) -> &'l json_ld_syntax::context::Context {
		self.unprocessed
	}

	pub fn into_processed(self) -> Context<T, B> {
		self.processed
	}

	pub fn as_ref(&self) -> ProcessedRef<'l, '_, T, B> {
		ProcessedRef {
			unprocessed: self.unprocessed,
			processed: &self.processed,
		}
	}

	pub fn into_owned(self) -> ProcessedOwned<T, B> {
		ProcessedOwned {
			unprocessed: self.unprocessed.clone(),
			processed: self.processed,
		}
	}
}

impl<'l, T, B> ops::Deref for Processed<'l, T, B> {
	type Target = Context<T, B>;

	fn deref(&self) -> &Self::Target {
		&self.processed
	}
}

impl<'l, T, B> ops::DerefMut for Processed<'l, T, B> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.processed
	}
}

/// Reference to a processed context that also borrows the original, unprocessed, context.
pub struct ProcessedRef<'l, 'a, T, B> {
	pub unprocessed: &'l json_ld_syntax::context::Context,
	pub processed: &'a Context<T, B>,
}

impl<'l, 'a, T, B> ProcessedRef<'l, 'a, T, B> {
	pub fn new(
		unprocessed: &'l json_ld_syntax::context::Context,
		processed: &'a Context<T, B>,
	) -> Self {
		Self {
			unprocessed,
			processed,
		}
	}

	pub fn unprocessed(&self) -> &'l json_ld_syntax::context::Context {
		self.unprocessed
	}

	pub fn processed(&self) -> &'a Context<T, B> {
		self.processed
	}
}

/// Processed context that also owns the original, unprocessed, context.
pub struct ProcessedOwned<T, B> {
	pub unprocessed: json_ld_syntax::context::Context,
	pub processed: Context<T, B>,
}

impl<T, B> ProcessedOwned<T, B> {
	pub fn new(unprocessed: json_ld_syntax::context::Context, processed: Context<T, B>) -> Self {
		Self {
			unprocessed,
			processed,
		}
	}

	pub fn unprocessed(&self) -> &json_ld_syntax::context::Context {
		&self.unprocessed
	}

	pub fn processed(&self) -> &Context<T, B> {
		&self.processed
	}

	pub fn as_ref(&self) -> ProcessedRef<T, B> {
		ProcessedRef {
			unprocessed: &self.unprocessed,
			processed: &self.processed,
		}
	}
}
