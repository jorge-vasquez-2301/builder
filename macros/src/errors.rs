use quote::quote;
use std::fmt;

pub type ParseResult<T> = std::result::Result<T, SyntaxErrors>;

#[derive(Debug, Default)]
pub struct SyntaxErrors {
    inner: Vec<syn::Error>,
}

impl From<Vec<syn::Error>> for SyntaxErrors {
    fn from(errors: Vec<syn::Error>) -> Self {
        Self { inner: errors }
    }
}

impl From<SyntaxErrors> for Vec<syn::Error> {
    fn from(errors: SyntaxErrors) -> Self {
        errors.inner
    }
}

impl SyntaxErrors {
    pub fn add<D, T>(&mut self, tts: T, description: D)
    where
        D: fmt::Display,
        T: quote::ToTokens,
    {
        self.inner.push(syn::Error::new_spanned(tts, description));
    }

    pub fn extend<T: Into<Vec<syn::Error>>>(&mut self, errors: T) {
        self.inner.extend(errors.into());
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn to_compile_errors(self) -> proc_macro2::TokenStream {
        let compile_errors = self.inner.iter().map(syn::Error::to_compile_error);
        quote! { #(#compile_errors)* }
    }
}
