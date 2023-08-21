use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Result as SynResult;

pub enum BuilderAttribute {
    Required(proc_macro2::TokenStream),
}

impl syn::parse::Parse for BuilderAttribute {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        use syn::Ident;

        let input_tts = input.cursor().token_stream();
        let name: Ident = input.parse()?;
        if name == "required" {
            Ok(BuilderAttribute::Required(input_tts))
        } else {
            Err(syn::Error::new(name.span(), "expected `required`"))
        }
    }
}

pub struct BuilderAttributeBody(pub Vec<BuilderAttribute>);

impl syn::parse::Parse for BuilderAttributeBody {
    fn parse(input: syn::parse::ParseStream) -> SynResult<Self> {
        use syn::punctuated::Punctuated;
        use syn::token::Comma;

        let inside;
        syn::parenthesized!(inside in input);

        let parse_comma_list = Punctuated::<BuilderAttribute, Comma>::parse_terminated;
        let list = parse_comma_list(&inside)?;

        Ok(BuilderAttributeBody(
            list.into_pairs().map(|p| p.into_value()).collect(),
        ))
    }
}

pub struct BuilderInfo {
    pub name: syn::Ident,
    pub generics: syn::Generics,
    pub fields: Vec<(Option<syn::Ident>, syn::Type, Vec<BuilderAttribute>)>,
}

impl From<BuilderInfo> for TokenStream {
    fn from(other: BuilderInfo) -> TokenStream {
        other.generate_builder().into()
    }
}

impl BuilderInfo {
    fn generate_builder(self) -> proc_macro2::TokenStream {
        let gen_typ = syn::Ident::new("__Builder_T", proc_macro2::Span::call_site());

        let setters = self.fields.iter().map(|(n, t, _)| {
            quote! {
                fn #n<#gen_typ: Into<#t>>(mut self, val: #gen_typ) -> Self {
                    self.#n = Some(val.into());
                    self
                }
            }
        });

        let builder_fields = self.fields.iter().map(|(n, t, _)| {
            quote! {
                #n: Option<#t>,
            }
        });

        let builder_defaults = self.fields.iter().map(|(n, _, _)| {
            quote! {
                #n: None,
            }
        });

        let builder_build = self.fields.iter().map(|(n, _t, a)| {
            if a.is_empty() {
                quote! {
                #n: self.#n.unwrap_or_else(Default::default),
                }
            } else {
                quote! {
                #n: self.#n.unwrap(),
                }
            }
        });

        let name = self.name;
        let (impl_generics, ty_generics, maybe_where) = self.generics.split_for_impl();
        let builder_name = syn::Ident::new(&format!("{}Builder", name), name.span());

        quote! {
            impl #impl_generics #name #ty_generics #maybe_where {
                fn builder() -> #builder_name #ty_generics {
                    #builder_name::new()
                }
            }

            impl #impl_generics Default for #builder_name #ty_generics #maybe_where {
                fn default() -> Self {
                    #builder_name {
                        #(#builder_defaults)*
                    }
                }
            }

            struct #builder_name #ty_generics #maybe_where {
                #(#builder_fields)*
            }

            impl #impl_generics #builder_name #ty_generics #maybe_where {
                fn new() -> Self {
                    Default::default()
                }

                #(#setters)*

                fn build(self) -> #name #ty_generics {
                    #name {
                        #(#builder_build)*
                    }
                }
            }
        }
    }
}
