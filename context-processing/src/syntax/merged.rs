use json_ld_syntax as syntax;
use locspan::Loc;
use iref::IriRef;

pub struct Merged<'a, C> {
	base: &'a C,
	imported: Option<&'a C>
}

impl<'a, C> Merged<'a, C> {
	pub fn new(
		base: &'a C,
		imported: Option<&'a C>
	) -> Self {
		Self {
			base,
			imported
		}
	}

	pub fn base<S, P>(&self) -> Option<Loc<syntax::Nullable<IriRef>, S, P>> where C: syntax::AnyContextDefinition<S, P> {
		self.imported.and_then(|i| i.base()).or_else(|| self.base.base())
	}

	pub fn vocab<S, P>(&self) -> Option<Loc<syntax::Nullable<syntax::context::VocabRef<S, P>>, S, P>> where C: syntax::AnyContextDefinition<S, P> {
		self.imported.and_then(|i| i.vocab()).or_else(|| self.base.vocab())
	}
}

impl<'a, C> From<&'a C> for Merged<'a, C> {
	fn from(base: &'a C) -> Self {
		Self {
			base,
			imported: None
		}
	}
} 