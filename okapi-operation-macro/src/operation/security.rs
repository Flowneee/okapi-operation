use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Meta;

use crate::utils::{meta_to_meta_list, meta_to_meta_name_value, nested_meta_to_meta};

static SECURITY_SCHEME_ATTRIBUTE_NAME: &str = "security_scheme";
static SECURITY_SCHEME_NAME_ATTRIBUTE_NAME: &str = "name";
static SECURITY_SCHEME_SCOPES_ATTRIBUTE_NAME: &str = "scopes";

#[derive(Default, Debug)]
pub struct Security {
    schemes: Vec<SecurityScheme>,
}

#[derive(Default, Debug)]
struct SecurityScheme {
    name: String,
    scopes: Vec<String>,
}

impl FromMeta for Security {
    fn from_meta(meta: &Meta) -> Result<Self, darling::Error> {
        let meta_list = meta_to_meta_list(meta)?;
        let mut this = Self::default();

        for nested_meta in meta_list.nested.iter() {
            let meta = nested_meta_to_meta(nested_meta)?;
            let meta_ident = meta
                .path()
                .get_ident()
                .ok_or_else(|| darling::Error::custom("Should have Ident").with_span(meta))?;

            match meta_ident {
                _ if meta_ident == SECURITY_SCHEME_ATTRIBUTE_NAME => {
                    this.schemes.push(SecurityScheme::from_meta(meta)?)
                }
                _ => {
                    return Err(darling::Error::custom("Unsupported type of parameter")
                        .with_span(meta_ident))
                }
            }
        }
        Ok(this)
    }
}

impl ToTokens for Security {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let schemes = &self.schemes;
        tokens.extend(quote! {
            vec![#(
                {
                    let mut val = okapi::openapi3::SecurityRequirement::new();
                    let (sch_key, sch_val) = #schemes;
                    val.insert(sch_key, sch_val);
                    val
                }
            ),*]
        });
    }
}

impl FromMeta for SecurityScheme {
    fn from_meta(meta: &Meta) -> Result<Self, darling::Error> {
        let meta_list = meta_to_meta_list(meta)?;
        let mut this = Self::default();

        for nested_meta in meta_list.nested.iter() {
            let meta = nested_meta_to_meta(nested_meta)?;
            let meta = meta_to_meta_name_value(meta)?;
            let meta_ident = meta
                .path
                .get_ident()
                .ok_or_else(|| darling::Error::custom("Should have Ident").with_span(meta))?;

            match meta_ident {
                _ if meta_ident == SECURITY_SCHEME_NAME_ATTRIBUTE_NAME => {
                    this.name = String::from_value(&meta.lit)?;
                }
                _ if meta_ident == SECURITY_SCHEME_SCOPES_ATTRIBUTE_NAME => {
                    let val = String::from_value(&meta.lit)?;
                    this.scopes = val.split(',').map(|v| v.to_owned()).collect();
                }
                _ => {
                    return Err(darling::Error::custom("Unsupported type of parameter")
                        .with_span(meta_ident))
                }
            }
        }

        if this.name.is_empty() {
            return Err(darling::Error::custom(format!(
                "Required attribute '{}' is missing",
                SECURITY_SCHEME_NAME_ATTRIBUTE_NAME
            ))
            .with_span(meta));
        }

        Ok(this)
    }
}

impl ToTokens for SecurityScheme {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let scopes = &self.scopes;
        tokens.extend(quote! {
            (
                std::borrow::ToOwned::to_owned(#name),
                vec![#(std::borrow::ToOwned::to_owned(#scopes)),*],
            )
        });
    }
}
