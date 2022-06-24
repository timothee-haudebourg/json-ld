use json_ld_syntax as syntax;
use locspan::Loc;
use iref::IriRef;
use syntax::AnyContextDefinition;

pub struct Merged<'a, C: syntax::AnyContextEntry> {
	base: &'a C::Definition,
	imported: Option<&'a C::Definition>
}

impl<'a, C: syntax::AnyContextEntry> Merged<'a, C> {
	pub fn new(
		base: &'a C::Definition,
		imported: Option<&'a C::Definition>
	) -> Self {
		Self {
			base,
			imported
		}
	}

	pub fn base(&self) -> Option<Loc<syntax::Nullable<IriRef>, C::Source, C::Span>> {
		self.imported.and_then(|i| i.base()).or_else(|| self.base.base())
	}

	pub fn vocab(&self) -> Option<Loc<syntax::Nullable<syntax::context::VocabRef>, C::Source, C::Span>> {
		self.imported.and_then(|i| i.vocab()).or_else(|| self.base.vocab())
	}

	pub fn get(&self, key: &syntax::context::KeyOrKeyword) -> Option<syntax::context::EntryRef<C>> {
		self.imported.and_then(|i| i.get(key)).or_else(|| self.base.get(key))
	}
}

impl<'a, C: syntax::AnyContextEntry> From<&'a C::Definition> for Merged<'a, C> {
	fn from(base: &'a C::Definition) -> Self {
		Self {
			base,
			imported: None
		}
	}
} 