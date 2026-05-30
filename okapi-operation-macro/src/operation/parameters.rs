use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{ItemFn, Meta, Token, punctuated::Punctuated};

use super::cookie::{COOKIE_ATTRIBUTE_NAME, Cookie};
use crate::{
    operation::{
        header::{HEADER_ATTRIBUTE_NAME, Header},
        path::{PATH_ATTRIBUTE_NAME, Path},
        query::{QUERY_ATTRIBUTE_NAME, Query},
        reference::{REFERENCE_ATTRIBUTE_NAME, Reference},
    },
    utils::meta_to_meta_list,
};

#[derive(Debug, FromMeta)]
#[darling(rename_all = "camelCase")]
pub(super) enum ParameterStyle {
    Matrix,
    Label,
    Form,
    Simple,
    SpaceDelimited,
    PipeDelimited,
    DeepObject,
}

impl ToTokens for ParameterStyle {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let new_tokens = match self {
            Self::Matrix => quote! { okapi::openapi3::ParameterStyle::Matrix },
            Self::Label => quote! { okapi::openapi3::ParameterStyle::Label },
            Self::Form => quote! { okapi::openapi3::ParameterStyle::Form },
            Self::Simple => quote! { okapi::openapi3::ParameterStyle::Simple },
            Self::SpaceDelimited => quote! { okapi::openapi3::ParameterStyle::SpaceDelimited },
            Self::PipeDelimited => quote! { okapi::openapi3::ParameterStyle::PipeDelimited },
            Self::DeepObject => quote! { okapi::openapi3::ParameterStyle::DeepObject },
        };
        tokens.extend(new_tokens);
    }
}

/// Parameters description (header/path/query/cookie).
#[derive(Default, Debug)]
pub(super) struct Parameters {
    header_parameters: Vec<Header>,
    path_parameters: Vec<Path>,
    query_parameters: Vec<Query>,
    cookie_parameters: Vec<Cookie>,
    ref_parameters: Vec<Reference>,
}

impl Parameters {
    /// Append parameters that can be inferred from the function signature
    /// (currently: axum `Path<...>` extractor). Names already declared via the
    /// macro arguments win — inferred entries with a duplicate name are
    /// dropped.
    pub(super) fn add_inferred_from_signature(&mut self, item_fn: &ItemFn) {
        #[cfg(feature = "axum")]
        {
            let inferred = super::path_inference::infer_path_parameters(item_fn);
            for param in inferred {
                if self
                    .path_parameters
                    .iter()
                    .any(|p| p.name() == param.name())
                {
                    continue;
                }
                self.path_parameters.push(param);
            }
        }
        #[cfg(not(feature = "axum"))]
        {
            let _ = item_fn;
        }
    }
}

impl FromMeta for Parameters {
    fn from_meta(meta: &Meta) -> Result<Self, darling::Error> {
        let meta_list = meta_to_meta_list(meta)?;
        let mut this = Self::default();
        for meta in meta_list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)? {
            let meta_ident = meta
                .path()
                .get_ident()
                .ok_or_else(|| darling::Error::custom("Should have Ident").with_span(&meta))?;
            if meta_ident == HEADER_ATTRIBUTE_NAME {
                this.header_parameters.push(Header::from_meta(&meta)?);
            } else if meta_ident == PATH_ATTRIBUTE_NAME {
                this.path_parameters.push(Path::from_meta(&meta)?);
            } else if meta_ident == QUERY_ATTRIBUTE_NAME {
                this.query_parameters.push(Query::from_meta(&meta)?);
            } else if meta_ident == COOKIE_ATTRIBUTE_NAME {
                this.cookie_parameters.push(Cookie::from_meta(&meta)?);
            } else if meta_ident == REFERENCE_ATTRIBUTE_NAME {
                this.ref_parameters.push(Reference::from_meta(&meta)?);
            } else {
                return Err(
                    darling::Error::custom("Unsupported type of parameter").with_span(meta_ident)
                );
            }
        }
        Ok(this)
    }
}

impl ToTokens for Parameters {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let header_parameters = self.header_parameters.iter().map(|x| x.for_parameter());
        let path_parameters = &self.path_parameters;
        let query_parameters = &self.query_parameters;
        let cookie_parameters = &self.cookie_parameters;
        let ref_parameters = &self.ref_parameters;
        tokens.extend(quote! {
            parameters: {
                let mut v = Vec::new();
                #(v.push(okapi::openapi3::RefOr::Object(#header_parameters));)*
                #(v.push(okapi::openapi3::RefOr::Object(#path_parameters));)*
                #(v.push(okapi::openapi3::RefOr::Object(#query_parameters));)*
                #(v.push(okapi::openapi3::RefOr::Object(#cookie_parameters));)*
                #(v.push(#ref_parameters);)*
                v
            },
        });
    }
}
