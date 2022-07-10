use std::fmt::Display;

use proc_macro2::TokenStream;
use quote::ToTokens;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error(transparent)]
    Syn(#[from] syn::Error),
    #[error(transparent)]
    Darling(#[from] darling::Error),
}

impl Error {
    pub(crate) fn syn_spanned<T: ToTokens, U: Display>(tokens: T, message: U) -> Self {
        Self::Syn(syn::Error::new_spanned(tokens, message))
    }

    pub(crate) fn write(self) -> TokenStream {
        match self {
            Error::Syn(x) => x.into_compile_error(),
            Error::Darling(x) => x.write_errors(),
        }
    }
}
