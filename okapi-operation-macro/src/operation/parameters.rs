use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Meta;

use crate::{
    operation::{
        header::{Header, HEADER_ATTRIBUTE_NAME},
        path::{Path, PATH_ATTRIBUTE_NAME},
        query::{Query, QUERY_ATTRIBUTE_NAME},
        reference::{Reference, REFERENCE_ATTRIBUTE_NAME},
    },
    utils::{meta_to_meta_list, nested_meta_to_meta},
};

// TODO: support cookie parameters
// TODO: support parameters from function signature

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

/// Parameters description (header/path/query) .
#[derive(Default, Debug)]
pub(super) struct Parameters {
    header_parameters: Vec<Header>,
    path_parameters: Vec<Path>,
    query_parameters: Vec<Query>,
    ref_parameters: Vec<Reference>,
}

impl FromMeta for Parameters {
    fn from_meta(meta: &Meta) -> Result<Self, darling::Error> {
        let meta_list = meta_to_meta_list(meta)?;
        let mut this = Self::default();
        for nested_meta in meta_list.nested.iter() {
            let meta = nested_meta_to_meta(nested_meta)?;
            let meta_ident = meta
                .path()
                .get_ident()
                .ok_or_else(|| darling::Error::custom("Should have Ident").with_span(meta))?;
            if meta_ident == HEADER_ATTRIBUTE_NAME {
                this.header_parameters.push(Header::from_meta(meta)?);
            } else if meta_ident == PATH_ATTRIBUTE_NAME {
                this.path_parameters.push(Path::from_meta(meta)?);
            } else if meta_ident == QUERY_ATTRIBUTE_NAME {
                this.query_parameters.push(Query::from_meta(meta)?);
            } else if meta_ident == REFERENCE_ATTRIBUTE_NAME {
                this.ref_parameters.push(Reference::from_meta(meta)?);
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
        let ref_parameters = &self.ref_parameters;
        tokens.extend(quote! {
            parameters: {
                let mut v = Vec::new();
                #(v.push(okapi::openapi3::RefOr::Object(#header_parameters));)*
                #(v.push(okapi::openapi3::RefOr::Object(#path_parameters));)*
                #(v.push(okapi::openapi3::RefOr::Object(#query_parameters));)*
                #(v.push(#ref_parameters);)*
                v
            },
        });
    }
}
