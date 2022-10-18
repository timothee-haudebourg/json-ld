use super::Type;

pub struct UnknownType;

pub struct Parsed {
	pub ty: Type,
	pub required: bool,
	pub multiple: bool,
}

pub fn parse(ty: syn::Type) -> Result<Parsed, UnknownType> {
	match ty {
		syn::Type::Reference(r) => match reference_into_multiple(r) {
			Ok(ty) => Ok(Parsed {
				ty: parse_multiple(ty)?,
				required: true,
				multiple: true,
			}),
			Err(r) => Ok(Parsed {
				ty: parse_reference(r)?,
				required: true,
				multiple: false,
			}),
		},
		syn::Type::Path(p) => match path_into_option(p) {
			Ok(ty) => parse_optional(ty),
			Err(p) => Ok(Parsed {
				ty: parse_path(p)?,
				required: true,
				multiple: false,
			}),
		},
		_ => Err(UnknownType),
	}
}

fn parse_optional(ty: syn::Type) -> Result<Parsed, UnknownType> {
	match ty {
		syn::Type::Reference(r) => match reference_into_multiple(r) {
			Ok(ty) => Ok(Parsed {
				ty: parse_multiple(ty)?,
				required: false,
				multiple: true,
			}),
			Err(r) => Ok(Parsed {
				ty: parse_reference(r)?,
				required: false,
				multiple: false,
			}),
		},
		syn::Type::Path(p) => Ok(Parsed {
			ty: parse_path(p)?,
			required: false,
			multiple: false,
		}),
		_ => Err(UnknownType),
	}
}

fn parse_multiple(ty: syn::Type) -> Result<Type, UnknownType> {
	match ty {
		syn::Type::Reference(r) => parse_reference(r),
		syn::Type::Path(p) => parse_path(p),
		_ => Err(UnknownType),
	}
}

fn parse_reference(r: syn::TypeReference) -> Result<Type, UnknownType> {
	if r.mutability.is_none() {
		if let Some(lft) = r.lifetime {
			if lft.ident == "static" && is_str(&r.elem) {
				return Ok(Type::String);
			}
		}
	}

	Err(UnknownType)
}

fn reference_into_multiple(r: syn::TypeReference) -> Result<syn::Type, syn::TypeReference> {
	if r.mutability.is_none() {
		if let Some(lft) = &r.lifetime {
			if lft.ident == "static" && matches!(r.elem.as_ref(), syn::Type::Slice(_)) {
				match *r.elem {
					syn::Type::Slice(e) => return Ok(*e.elem),
					_ => unreachable!(),
				}
			}
		}
	}

	Err(r)
}

fn parse_path(p: syn::TypePath) -> Result<Type, UnknownType> {
	if is_bool_path(&p) {
		Ok(Type::Bool)
	} else if is_iri_path(&p) {
		Ok(Type::Iri)
	} else if is_processing_mode_path(&p) {
		Ok(Type::ProcessingMode)
	} else if p.path.leading_colon.is_none()
		&& p.path.segments.len() == 1
		&& p.path.segments[0].arguments.is_empty()
	{
		Ok(Type::Ref(p.path.segments[0].ident.clone()))
	} else {
		Err(UnknownType)
	}
}

fn segment_is_ident(path: &syn::Path, i: usize, id: &str) -> bool {
	let segment = &path.segments[i];
	segment.ident == id
}

fn segment_is_empty_ident(path: &syn::Path, i: usize, id: &str) -> bool {
	let segment = &path.segments[i];
	segment.arguments.is_empty() && segment.ident == id
}

fn is_static_arguments(args: &syn::PathArguments) -> bool {
	match args {
		syn::PathArguments::AngleBracketed(args) => {
			args.args.len() == 1 && is_static_argument(&args.args[0])
		}
		_ => false,
	}
}

fn is_static_argument(arg: &syn::GenericArgument) -> bool {
	match arg {
		syn::GenericArgument::Lifetime(lft) => lft.ident == "static",
		_ => false,
	}
}

fn is_str(ty: &syn::Type) -> bool {
	match ty {
		syn::Type::Path(p) => is_str_path(p),
		_ => false,
	}
}

fn is_bool_path(p: &syn::TypePath) -> bool {
	p.qself.is_none()
		&& ((p.path.segments.len() == 3
			&& segment_is_empty_ident(&p.path, 0, "core")
			&& segment_is_empty_ident(&p.path, 1, "primitive")
			&& segment_is_empty_ident(&p.path, 2, "bool"))
			|| (p.path.leading_colon.is_none()
				&& p.path.segments.len() == 1
				&& segment_is_empty_ident(&p.path, 0, "bool")))
}

fn is_str_path(p: &syn::TypePath) -> bool {
	p.qself.is_none()
		&& ((p.path.segments.len() == 2
			&& (segment_is_empty_ident(&p.path, 0, "std")
				|| segment_is_empty_ident(&p.path, 0, "core")
				|| segment_is_empty_ident(&p.path, 0, "alloc"))
			&& segment_is_empty_ident(&p.path, 1, "str"))
			|| (p.path.leading_colon.is_none()
				&& p.path.segments.len() == 1
				&& segment_is_empty_ident(&p.path, 0, "str")))
}

fn is_iri_path(p: &syn::TypePath) -> bool {
	p.qself.is_none()
		&& ((p.path.segments.len() == 2
			&& segment_is_empty_ident(&p.path, 0, "iref")
			&& segment_is_ident(&p.path, 1, "Iri"))
			|| (p.path.leading_colon.is_none()
				&& p.path.segments.len() == 1
				&& segment_is_ident(&p.path, 0, "Iri")))
		&& is_static_arguments(&p.path.segments.last().unwrap().arguments)
}

fn is_processing_mode_path(p: &syn::TypePath) -> bool {
	p.qself.is_none()
		&& ((p.path.segments.len() == 2
			&& segment_is_empty_ident(&p.path, 0, "json_ld")
			&& segment_is_empty_ident(&p.path, 1, "ProcessingMode"))
			|| (p.path.leading_colon.is_none()
				&& p.path.segments.len() == 1
				&& segment_is_empty_ident(&p.path, 0, "ProcessingMode")))
}

fn is_option_path(p: &syn::TypePath) -> bool {
	p.qself.is_none()
		&& ((p.path.segments.len() == 3
			&& (segment_is_empty_ident(&p.path, 0, "std")
				|| segment_is_empty_ident(&p.path, 0, "core"))
			&& segment_is_empty_ident(&p.path, 1, "option")
			&& segment_is_ident(&p.path, 2, "Option"))
			|| (p.path.leading_colon.is_none()
				&& p.path.segments.len() == 1
				&& segment_is_ident(&p.path, 0, "Option")))
		&& is_option_arguments(&p.path.segments.last().unwrap().arguments)
}

fn is_option_arguments(args: &syn::PathArguments) -> bool {
	match args {
		syn::PathArguments::AngleBracketed(args) => {
			args.args.len() == 1 && matches!(&args.args[0], syn::GenericArgument::Type(_))
		}
		_ => false,
	}
}

fn path_into_option(p: syn::TypePath) -> Result<syn::Type, syn::TypePath> {
	if is_option_path(&p) {
		match p.path.segments.into_iter().last().unwrap().arguments {
			syn::PathArguments::AngleBracketed(args) => {
				match args.args.into_iter().last().unwrap() {
					syn::GenericArgument::Type(ty) => Ok(ty),
					_ => unreachable!(),
				}
			}
			_ => unreachable!(),
		}
	} else {
		Err(p)
	}
}
