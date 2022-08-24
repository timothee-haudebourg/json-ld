use iref::IriRef;
use json_ld_syntax::{
	self as syntax,
	context::definition::{AnyDefinition, TermBindingRef},
	Entry,
};
use locspan::Meta;

pub struct Merged<'a, C: syntax::context::AnyValue> {
	base: &'a C::Definition,
	imported: Option<C>,
}

impl<'a, C: syntax::context::AnyValue> Merged<'a, C> {
	pub fn new(base: &'a C::Definition, imported: Option<C>) -> Self {
		Self { base, imported }
	}

	pub fn imported(&self) -> Option<&C::Definition> {
		self.imported
			.as_ref()
			.and_then(|imported| match imported.as_value_ref() {
				syntax::context::ValueRef::One(Meta(
					syntax::ContextRef::Definition(import_context),
					_,
				)) => Some(import_context),
				_ => None,
			})
	}

	pub fn base(&self) -> Option<Entry<syntax::Nullable<IriRef>, C::Metadata>> {
		self.base
			.base()
			.or_else(|| self.imported().and_then(|i| i.base()))

		// self.imported()
		// 	.and_then(|i| i.base())
		// 	.or_else(|| self.base.base())
	}

	pub fn vocab(
		&self,
	) -> Option<Entry<syntax::Nullable<syntax::context::definition::VocabRef>, C::Metadata>> {
		self.base
			.vocab()
			.or_else(|| self.imported().and_then(|i| i.vocab()))
		// self.imported()
		// 	.and_then(|i| i.vocab())
		// 	.or_else(|| self.base.vocab())
	}

	pub fn language(
		&self,
	) -> Option<Entry<syntax::Nullable<syntax::LenientLanguageTag>, C::Metadata>> {
		self.base
			.language()
			.or_else(|| self.imported().and_then(|i| i.language()))
		// self.imported()
		// 	.and_then(|i| i.language())
		// 	.or_else(|| self.base.language())
	}

	pub fn direction(&self) -> Option<Entry<syntax::Nullable<syntax::Direction>, C::Metadata>> {
		self.base
			.direction()
			.or_else(|| self.imported().and_then(|i| i.direction()))
		// self.imported()
		// 	.and_then(|i| i.direction())
		// 	.or_else(|| self.base.direction())
	}

	pub fn protected(&self) -> Option<Entry<bool, C::Metadata>> {
		self.base
			.protected()
			.or_else(|| self.imported().and_then(|i| i.protected()))
		// self.imported()
		// 	.and_then(|i| i.protected())
		// 	.or_else(|| self.base.protected())
	}

	pub fn type_(
		&self,
	) -> Option<Entry<syntax::context::definition::Type<C::Metadata>, C::Metadata>> {
		self.base
			.type_()
			.or_else(|| self.imported().and_then(|i| i.type_()))
		// self.imported()
		// 	.and_then(|i| i.protected())
		// 	.or_else(|| self.base.protected())
	}

	pub fn bindings(&self) -> MergedBindings<C> {
		MergedBindings {
			base: self.base,
			base_bindings: self.base.bindings(),
			imported_bindings: self.imported().map(|i| i.bindings()),
		}
	}

	pub fn get(
		&self,
		key: &syntax::context::definition::KeyOrKeyword,
	) -> Option<syntax::context::definition::EntryValueRef<C>> {
		self.base
			.get(key)
			.or_else(|| self.imported().and_then(|i| i.get(key)))
		// self.imported()
		// 	.and_then(|i| i.get(key))
		// 	.or_else(|| self.base.get(key))
	}
}

impl<'a, C: syntax::context::AnyValue> From<&'a C::Definition> for Merged<'a, C> {
	fn from(base: &'a C::Definition) -> Self {
		Self {
			base,
			imported: None,
		}
	}
}

pub struct MergedBindings<'a, C: 'a + syntax::context::AnyValue> {
	base: &'a C::Definition,
	base_bindings: <C::Definition as syntax::context::AnyDefinition>::Bindings<'a>,
	imported_bindings: Option<<C::Definition as syntax::context::AnyDefinition>::Bindings<'a>>,
}

impl<'a, C: 'a + syntax::context::AnyValue> Iterator for MergedBindings<'a, C> {
	type Item = (
		syntax::context::definition::KeyRef<'a>,
		TermBindingRef<'a, C>,
	);

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.imported_bindings {
			Some(imported_bindings) => {
				while let Some((key_ref, def)) = imported_bindings.next() {
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
