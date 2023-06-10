use quote::ToTokens;
use syn::{Attribute, Expr, ExprLit, ExprPath, Lit, MetaNameValue, Path};

/// `#[idx = 1]` in fields
pub fn extract_idx_attr(attrs: &[Attribute]) -> Option<u32> {
    attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("idx") {
            return None;
        }

        let nv = attr.meta.require_name_value().unwrap();

        match &nv.value {
            Expr::Lit(ExprLit {
                lit: Lit::Int(i), ..
            }) => Some(i.base10_parse::<u32>().unwrap()),
            _ => {
                panic!("error meta type");
            }
        }
    })
}

/// `#[rapira(with = rapira::byte_rapira)]` in fields
pub fn extract_with_attr(attrs: &[Attribute]) -> Option<ExprPath> {
    attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("rapira") {
            return None;
        }

        let Ok(nv) = attr.parse_args::<MetaNameValue>() else {
            return None;
        };

        if !nv.path.is_ident("with") {
            return None;
        }

        let Expr::Path(path) = nv.value else {
            panic!("invalid 'with' path value: `{}`", nv.value.into_token_stream());
        };

        Some(path)
    })
}

/// `#[rapira(skip)]` in fields
pub fn skip_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("rapira") {
            return false;
        }

        if let Ok(path) = attr.parse_args::<Path>() {
            path.is_ident("skip")
        } else {
            false
        }
    })
}
