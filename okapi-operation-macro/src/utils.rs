use darling::ast::NestedMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, Attribute, Meta, MetaList, MetaNameValue, Token};

use crate::error::Error;

pub(super) fn quote_option<T: ToTokens>(v: &Option<T>) -> TokenStream {
    v.as_ref()
        .map_or(quote! { None }, |x| quote! { Some(#x.into()) })
}

pub(super) fn attribute_to_args(attr: &Attribute) -> Result<Vec<NestedMeta>, Error> {
    match &attr.meta {
        Meta::Path(_) => Ok(Vec::new()),
        Meta::List(x) => Ok(x
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
            .into_iter()
            .map(NestedMeta::Meta)
            .collect()),
        Meta::NameValue(_) => Err(Error::syn_spanned(attr, "expected parentheses")),
    }
}

pub(super) fn take_attributes(attrs: &mut Vec<Attribute>, attr_name: &str) -> Vec<Attribute> {
    let mut non_matched_attrs = vec![];
    let mut result = vec![];
    for attr in attrs.drain(..) {
        if attr.path().get_ident().map_or(false, |x| x == attr_name) {
            result.push(attr);
        } else {
            non_matched_attrs.push(attr);
        }
    }
    *attrs = non_matched_attrs;
    result
}

pub(super) fn meta_to_meta_list(meta: &Meta) -> Result<&MetaList, darling::Error> {
    match meta {
        Meta::List(list) => Ok(list),
        rest => Err(darling::Error::custom(format!(
            "'{}' should be Meta::List",
            rest.path()
                .get_ident()
                .map(|x| x.to_string())
                .unwrap_or_else(|| "<unknown>".into())
        ))
        .with_span(rest)),
    }
}

pub(super) fn meta_to_meta_name_value(meta: &Meta) -> Result<&MetaNameValue, darling::Error> {
    match meta {
        Meta::NameValue(name_value) => Ok(name_value),
        rest => Err(darling::Error::custom(format!(
            "'{}' should be Meta::NameValue",
            rest.path()
                .get_ident()
                .map(|x| x.to_string())
                .unwrap_or_else(|| "<unknown>".into())
        ))
        .with_span(rest)),
    }
}
