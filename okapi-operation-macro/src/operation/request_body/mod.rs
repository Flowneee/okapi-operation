use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{FnArg, ItemFn, PatType, Path, Type};

use crate::{
    error::Error,
    utils::{attribute_to_args, quote_option},
};

#[cfg(feature = "axum")]
mod axum;

static REQUEST_BODY_ATTRIBUTE_NAME_DEPRECATED: &str = "request_body";
static REQUEST_BODY_ATTRIBUTE_NAME: &str = "body";

/// Request body definition for inline attribute.
#[derive(Debug, FromMeta, Default)]
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
        for pt in item_fn.sig.inputs.iter_mut().filter_map(|x| match x {
            FnArg::Receiver(_) => None,
            FnArg::Typed(y) => Some(y),
        }) {
            if let Some(x) = Self::try_find_in_arg_attrs(pt)? {
                return Ok(Some(x));
            }

            if let Some(x) = Self::try_find_framework_specific(pt)? {
                return Ok(Some(x));
            }
        }

        Ok(None)
    }

    // NOTE: also removes all related attributes
    fn try_find_in_arg_attrs(pt: &mut PatType) -> Result<Option<Self>, Error> {
        let mut non_matched_attrs = vec![];
        let mut matched_attrs = vec![];

        // Check attributes, removing matching
        for attr in pt.attrs.drain(..) {
            if attr.path().get_ident().is_some_and(|x| {
                x == REQUEST_BODY_ATTRIBUTE_NAME || x == REQUEST_BODY_ATTRIBUTE_NAME_DEPRECATED
            }) {
                matched_attrs.push(attr);
            } else {
                non_matched_attrs.push(attr);
            }
        }
        pt.attrs = non_matched_attrs;

        if matched_attrs.len() > 1 {
            return Err(Error::syn_spanned(
                pt,
                "Only single #[body] argument allowed",
            ));
        }
        let Some(attr) = matched_attrs.into_iter().next() else {
            return Ok(None);
        };
        let parsed_attrs = RequestBodyAttrs::from_list(&attribute_to_args(&attr)?)?;

        Ok(Some(Self {
            attrs: parsed_attrs,
            argument_type: *pt.ty.clone(),
        }))
    }

    // TODO: allow disable this behaviour
    #[allow(unused)]
    fn try_find_framework_specific(pt: &PatType) -> Result<Option<Self>, Error> {
        #[cfg(feature = "axum")]
        if let Some(x) = Self::try_find_axum(pt)? {
            return Ok(Some(x));
        }

        Ok(None)
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
