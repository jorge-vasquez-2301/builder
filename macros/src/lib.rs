mod builder;
mod errors;

use proc_macro::TokenStream;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Could not parse type to derive Builder for");

    impl_builder_macro(ast)
}

fn impl_builder_macro(ty: syn::DeriveInput) -> TokenStream {
    match builder::parse(ty) {
        Ok(info) => info.into(),
        Err(e) => e.to_compile_errors().into(),
    }
}
