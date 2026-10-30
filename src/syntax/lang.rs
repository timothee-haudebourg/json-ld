pub use langtag::{InvalidLangTag, LangTag, LangTagBuf};

use crate::{
	JsonLdValidate,
	syntax::utils::{CaselessStr, CaselessString},
};

impl JsonLdValidate for LangTagBuf {
	type Invalid = CaselessString;
}

impl<'a> JsonLdValidate for &'a LangTag {
	type Invalid = &'a CaselessStr;
}
