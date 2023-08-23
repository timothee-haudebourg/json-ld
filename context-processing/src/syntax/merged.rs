use iref::IriRefBuf;
use json_ld_syntax::{self as syntax, Entry};
use locspan::Meta;

pub struct Merged<'a, M> {
	base: &'a syntax::context::Definition<M>,
	imported: Option<syntax::context::Value<M>>,
}

impl<'a, M> Merged<'a, M> {
	pub fn new(
		base: &'a syntax::context::Definition<M>,
		imported: Option<syntax::context::Value<M>>,
	) -> Self {
		Self { base, imported }
	}

	pub fn imported(&self) -> Option<&syntax::context::Definition<M>> {
		self.imported.as_ref().and_then(|imported| match imported {
			syntax::context::Value::One(Meta(syntax::Context::Definition(import_context), _)) => {
				Some(import_context)
			}
			_ => None,
		})
	}

	pub fn base(&self) -> Option<&Entry<syntax::Nullable<IriRefBuf>, M>> {
		self.base
			.base
			.as_ref()
			.or_else(|| self.imported().and_then(|i| i.base.as_ref()))

		// self.imported()
		// 	.and_then(|i| i.base())
		// 	.or_else(|| self.base.base())
	}

	pub fn vocab(&self) -> Option<&Entry<syntax::Nullable<syntax::context::definition::Vocab>, M>> {
		self.base
			.vocab
			.as_ref()
			.or_else(|| self.imported().and_then(|i| i.vocab.as_ref()))
		// self.imported()
		// 	.and_then(|i| i.vocab())
		// 	.or_else(|| self.base.vocab())
	}

	pub fn language(&self) -> Option<&Entry<syntax::Nullable<syntax::LenientLanguageTagBuf>, M>> {
		self.base
			.language
			.as_ref()
			.or_else(|| self.imported().and_then(|i| i.language.as_ref()))
		// self.imported()
		// 	.and_then(|i| i.language())
		// 	.or_else(|| self.base.language())
	}

	pub fn direction(&self) -> Option<&Entry<syntax::Nullable<syntax::Direction>, M>> {
		self.base
			.direction
			.as_ref()
			.or_else(|| self.imported().and_then(|i| i.direction.as_ref()))
		// self.imported()
		// 	.and_then(|i| i.direction())
		// 	.or_else(|| self.base.direction())
	}

	pub fn protected(&self) -> Option<&Entry<bool, M>> {
		self.base
			.protected
			.as_ref()
			.or_else(|| self.imported().and_then(|i| i.protected.as_ref()))
		// self.imported()
		// 	.and_then(|i| i.protected())
		// 	.or_else(|| self.base.protected())
	}

	pub fn type_(&self) -> Option<&Entry<syntax::context::definition::Type<M>, M>> {
		self.base
			.type_
			.as_ref()
			.or_else(|| self.imported().and_then(|i| i.type_.as_ref()))
		// self.imported()
		// 	.and_then(|i| i.protected())
		// 	.or_else(|| self.base.protected())
	}

	pub fn bindings(&self) -> MergedBindings<M> {
		MergedBindings {
			base: self.base,
			base_bindings: self.base.bindings.iter(),
			imported_bindings: self.imported().map(|i| i.bindings.iter()),
		}
	}

	pub fn get(
		&self,
		key: &syntax::context::definition::KeyOrKeyword,
	) -> Option<syntax::context::definition::EntryValueRef<M>> {
		self.base
			.get(key)
			.or_else(|| self.imported().and_then(|i| i.get(key)))
		// self.imported()
		// 	.and_then(|i| i.get(key))
		// 	.or_else(|| self.base.get(key))
	}
}

impl<'a, M> From<&'a syntax::context::Definition<M>> for Merged<'a, M> {
	fn from(base: &'a syntax::context::Definition<M>) -> Self {
		Self {
			base,
			imported: None,
		}
	}
}

pub struct MergedBindings<'a, M> {
	base: &'a syntax::context::Definition<M>,
	base_bindings: syntax::context::definition::BindingsIter<'a, M>,
	imported_bindings: Option<syntax::context::definition::BindingsIter<'a, M>>,
}

impl<'a, M: Clone> Iterator for MergedBindings<'a, M> {
	type Item = (
		&'a syntax::context::definition::Key,
		&'a syntax::context::definition::TermBinding<M>,
	);

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.imported_bindings {
			Some(imported_bindings) => {
				for (key_ref, def) in imported_bindings {
					let key = key_ref.to_owned();
					if self.base.get_binding(&key).is_none() {
						return Some((key_ref, def));
					}
				}

				self.base_bindings.next()
			}
			None => self.base_bindings.next(),
		}
	}
}
