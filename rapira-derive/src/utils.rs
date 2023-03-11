use syn::{Attribute, ExprPath, Lit, Meta, MetaList, MetaNameValue, NestedMeta, Path};

pub fn extract_with_attr(attrs: &[Attribute]) -> Option<ExprPath> {
    attrs.iter().find_map(|attr| {
        attr.path.segments.first().and_then(|segment| {
            if segment.ident != "rapira" {
                return None;
            }
            match attr.parse_meta() {
                Ok(Meta::List(MetaList { nested, .. })) => {
                    nested.iter().find_map(|meta| match meta {
                        NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                            path,
                            lit: Lit::Str(lit_str),
                            ..
                        })) => {
                            if path.segments.first().unwrap().ident != "with" {
                                None
                            } else {
                                match lit_str.parse::<ExprPath>() {
                                    Ok(path) => Some(path),
                                    Err(_) => {
                                        panic!("invalid path");
                                    }
                                }
                            }
                        }
                        _ => None,
                    })
                }
                Ok(_) => None,
                Err(_) => None,
            }
        })
    })
}

pub fn extract_idx_attr(attrs: &[Attribute]) -> Option<u32> {
    attrs.iter().find_map(|attr| {
        attr.path.segments.first().and_then(|segment| {
            if segment.ident != "idx" {
                return None;
            }
            match attr.parse_meta() {
                Ok(Meta::List(list)) => {
                    let a = list.nested.first().unwrap();
                    let int: u32 = match a {
                        NestedMeta::Lit(Lit::Int(i)) => i.base10_parse::<u32>().unwrap(),
                        _ => {
                            panic!("error meta type")
                        }
                    };
                    Some(int)
                }
                Ok(Meta::NameValue(nv)) => match nv.lit {
                    Lit::Int(i) => Some(i.base10_parse::<u32>().unwrap()),
                    _ => {
                        panic!("error meta type")
                    }
                },
                Ok(_) => None,
                Err(_) => None,
            }
        })
    })
}

pub fn skip_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path
            .segments
            .first()
            .map(|segment| {
                if segment.ident != "rapira" {
                    return false;
                }

                match attr.parse_meta() {
                    Ok(Meta::List(list)) => list.nested.iter().any(|nested| match nested {
                        NestedMeta::Meta(Meta::Path(Path { segments, .. })) => {
                            segments.iter().any(|segment| segment.ident == "skip")
                        }
                        _ => false,
                    }),
                    Ok(_) => false,
                    Err(_) => false,
                }
            })
            .unwrap_or(false)
    })
}
