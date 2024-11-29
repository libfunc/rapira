use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    GenericParam, Generics, Path, PathSegment, PredicateType, TraitBound, TraitBoundModifier, Type,
    TypeParam, TypeParamBound, TypePath, WherePredicate,
    punctuated::Punctuated,
    token::{Colon, Comma},
};

pub fn build_ident(name: &Ident, mut generics: Generics) -> TokenStream {
    if generics.params.is_empty() {
        return quote! { impl rapira::Rapira for #name };
    }

    let predicates: Punctuated<WherePredicate, Comma> = generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(TypeParam { ident, .. }) => {
                let path_segment = PathSegment::from(ident.clone());
                let path = Path::from(path_segment);
                let type_path = TypePath { qself: None, path };
                let ty = Type::from(type_path);

                let rapira_path: Path = syn::parse_quote! { rapira::Rapira };
                let trait_bound = TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path: rapira_path,
                };
                let type_param_bound = TypeParamBound::Trait(trait_bound);
                let mut bounds = Punctuated::new();
                bounds.push(type_param_bound);

                let predicate = WherePredicate::Type(PredicateType {
                    lifetimes: None,
                    bounded_ty: ty,
                    colon_token: Colon::default(),
                    bounds,
                });
                Some(predicate)
            }
            _ => None,
        })
        .collect();

    let where_clause = generics.make_where_clause();
    where_clause.predicates.extend(predicates);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! { impl #impl_generics rapira::Rapira for #name #ty_generics #where_clause }
}
