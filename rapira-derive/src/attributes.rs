extern crate proc_macro2;
extern crate quote;
extern crate syn;

use syn::{Attribute, Expr, Ident, MetaNameValue};

/// `#[primitive(PrimitiveName)]` in enums
pub fn get_primitive_name(attrs: &[Attribute]) -> Option<Ident> {
    attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("primitive") {
            return None;
        }

        attr.parse_args().unwrap()
    })
}

/// `#[rapira(static_size = None)]` in enums
pub fn enum_static_size(attrs: &[Attribute]) -> Option<Expr> {
    attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("rapira") {
            return None;
        }

        if let Ok(nv) = attr.parse_args::<MetaNameValue>() {
            return nv.path.is_ident("static_size").then_some(nv.value);
        }

        None
    })
}
