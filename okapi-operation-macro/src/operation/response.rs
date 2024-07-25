use std::ops::Deref;

use darling::FromMeta;
use quote::{quote, ToTokens};
use syn::{
    punctuated::Punctuated, token::Paren, ItemFn, Meta, Path, ReturnType, Token, Type, TypeTuple,
};

use crate::{
    operation::{
        header::{Header, HEADER_ATTRIBUTE_NAME},
        reference::{Reference, REFERENCE_ATTRIBUTE_NAME},
    },
    utils::meta_to_meta_list,
};

// TODO: throw error if responses from different sources overlap OR merge them via oneOf

static RESPONSE_ATTRIBUTE_NAME: &str = "response";
static IGNORE_RETURN_TYPE_ATTRIBUTE_NAME: &str = "ignore_return_type";
static FROM_TYPE_ATTRIBUTE_NAME: &str = "from_type";

#[derive(Debug, Default)]
struct Headers {
    headers: Vec<Header>,
    refs: Vec<Reference>,
}

impl FromMeta for Headers {
    fn from_meta(meta: &Meta) -> Result<Self, darling::Error> {
        let meta_list = meta_to_meta_list(meta)?;
        let mut this = Self::default();
        for meta in meta_list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)? {
            let meta_ident = meta
                .path()
                .get_ident()
                .ok_or_else(|| darling::Error::custom("Should have Ident").with_span(&meta))?;
            if meta_ident == HEADER_ATTRIBUTE_NAME {
                this.headers.push(Header::from_meta(&meta)?);
            } else if meta_ident == REFERENCE_ATTRIBUTE_NAME {
                this.refs.push(Reference::from_meta(&meta)?);
            } else {
                return Err(darling::Error::custom(
                    "Response's header definition should have 'header' or 'reference' Ident",
                )
                .with_span(meta_ident));
            }
        }
        Ok(this)
    }
}

impl ToTokens for Headers {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let headers = self
            .headers
            .iter()
            .map(|x| {
                let value = x.for_response();
                let name = &x.name;
                quote! {
                    let _ = map.insert(#name.into(), okapi::openapi3::RefOr::Object(#value));
                }
            })
            .chain(self.refs.iter().map(|x| {
                let name = x.name();
                quote! {
                    let _ = map.insert(#name.into(), #x);
                }
            }));
        tokens.extend(quote! {{
            let mut map = okapi::map! {};
            #(#headers;)*
            map
        }});
    }
}

#[derive(Debug, FromMeta)]
struct Response {
    status: String,
    description: String,
    content: Path,
    #[darling(default)]
    headers: Headers,
}

impl ToTokens for Response {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let description = &self.description;
        let ty = &self.content;
        let headers = &self.headers;
        let new_tokens = quote! {
            okapi::openapi3::RefOr::Object(okapi::openapi3::Response {
                description: #description.into(),
                content: <#ty as ToMediaTypes>::generate(components)?,
                headers: #headers,
                ..Default::default()
            })
        };
        tokens.extend(new_tokens);
    }
}

#[derive(Debug, FromMeta)]
struct RefResponse {
    status: String,
    reference: Reference,
}

impl ToTokens for RefResponse {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let reference = &self.reference;
        tokens.extend(quote! { #reference });
    }
}

fn unit_type() -> Type {
    Type::Tuple(TypeTuple {
        paren_token: Paren::default(),
        elems: Punctuated::new(),
    })
}

#[derive(Debug)]
pub(super) struct Responses {
    responses: Vec<Response>,
    refs: Vec<RefResponse>,
    from_type: Vec<Path>,
    ret_type: Type,
    pub ignore_return_type: bool,
}

impl Responses {
    pub(crate) fn add_return_type(&mut self, item_fn: &ItemFn, ignore_return_type: bool) {
        self.ignore_return_type = ignore_return_type;
        self.ret_type = if let ReturnType::Type(_, ref ty) = item_fn.sig.output {
            ty.deref().clone()
        } else {
            Type::Tuple(TypeTuple {
                paren_token: Paren::default(),
                elems: Punctuated::new(),
            })
        };
    }
}

impl Default for Responses {
    fn default() -> Self {
        Self {
            responses: Default::default(),
            refs: Default::default(),
            from_type: Default::default(),
            ret_type: unit_type(),
            ignore_return_type: Default::default(),
        }
    }
}

impl FromMeta for Responses {
    fn from_meta(meta: &Meta) -> Result<Self, darling::Error> {
        let meta_list = meta_to_meta_list(meta)?;
        let mut this = Self::default();
        for meta in meta_list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)? {
            let meta_ident = meta
                .path()
                .get_ident()
                .ok_or_else(|| darling::Error::custom("Should have Ident").with_span(&meta))?;
            if meta_ident == RESPONSE_ATTRIBUTE_NAME {
                let parsed = Response::from_meta(&meta)?;
                this.responses.push(parsed);
            } else if meta_ident == REFERENCE_ATTRIBUTE_NAME {
                let parsed = RefResponse::from_meta(&meta)?;
                this.refs.push(parsed);
            } else if meta_ident == IGNORE_RETURN_TYPE_ATTRIBUTE_NAME {
                this.ignore_return_type = bool::from_meta(&meta)?;
            } else if meta_ident == FROM_TYPE_ATTRIBUTE_NAME {
                this.from_type.push(Path::from_meta(&meta)?);
            } else {
                return Err(darling::Error::custom(
                    "Response definition should have 'response', 'reference', 'from_type' or 'ignore_return_type' Ident",
                )
                .with_span(meta_ident));
            }
        }
        Ok(this)
    }
}

impl ToTokens for Responses {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let base_responses = if self.ignore_return_type {
            quote! { okapi::openapi3::Responses::default() }
        } else {
            let ret_type = &self.ret_type;
            quote! { <#ret_type as ToResponses>::generate(components)? }
        };
        let attrs = self
            .responses
            .iter()
            .map(|x| (&x.status, quote! {#x}))
            .chain(self.refs.iter().map(|x| (&x.status, quote! {#x})))
            .map(|(status, response)| {
                if status == "default" {
                    quote! { responses.default.replace(#response) }
                } else {
                    quote! { responses.responses.insert(#status.into(), #response) }
                }
            });
        let from_type = self.from_type.iter().map(|ty| {
            quote! {
                okapi::merge::merge_responses(
                    &mut responses,
                    &<#ty as ToResponses>::generate(components)?
                ).map_err(|err| anyhow::anyhow!("Failed to merge responses: {}", err))?
            }
        });
        tokens.extend(quote! {
            responses: {
                let mut responses = #base_responses;
                #(#attrs;)*
                #(#from_type;)*
                responses
            },
        });
    }
}
