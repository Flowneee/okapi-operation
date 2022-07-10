use std::ops::Deref;

use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{FnArg, ItemFn, Path, Type};

use crate::{
    error::Error,
    utils::{attribute_to_args, quote_option, remove_attributes},
};

static REQUEST_BODY_ATTRIBUTE_NAME: &str = "request_body";

/// Request body definition for inline attribute.
#[derive(Debug, FromMeta)]
struct RequestBodyAttrs {
    #[darling(default)]
    description: Option<String>,
    #[darling(default)]
    required: bool,
    #[darling(default)]
    content: Option<Path>,
}

#[derive(Debug)]
pub(super) struct RequestBody {
    attrs: RequestBodyAttrs,
    argument_type: Type,
}

impl RequestBody {
    /// Create body definition from function signature.
    pub(super) fn from_item_fn(item_fn: &mut ItemFn) -> Result<Option<Self>, Error> {
        let (ty, attr) = if let Some(x) = item_fn
            .sig
            .inputs
            .iter_mut()
            .filter_map(|x| match x {
                FnArg::Receiver(_) => None,
                FnArg::Typed(y) => Some(y),
            })
            .find_map(|pt| {
                remove_attributes(&mut pt.attrs, REQUEST_BODY_ATTRIBUTE_NAME)
                    .into_iter()
                    .next()
                    .map(|x| (pt.ty.deref().clone(), x))
            }) {
            x
        } else {
            return Ok(None);
        };
        let parsed_attrs = RequestBodyAttrs::from_list(&attribute_to_args(&attr)?)?;
        Ok(Some(Self {
            attrs: parsed_attrs,
            argument_type: ty,
        }))
    }
}

impl ToTokens for RequestBody {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let description = quote_option(&self.attrs.description);
        let required = self.attrs.required;
        let content_generator = if let Some(ref x) = self.attrs.content {
            quote! {
                <#x as ToMediaTypes>::generate
            }
        } else {
            let ty = &self.argument_type;
            quote! {
                <#ty as ToMediaTypes>::generate
            }
        };
        tokens.extend(quote! {
            okapi::openapi3::RequestBody {
                description: #description,
                required: #required,
                content: #content_generator(components)?,
                ..Default::default()
            }
        })
    }
}
