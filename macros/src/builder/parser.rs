use crate::{
    builder::types::{BuilderAttribute, BuilderAttributeBody, BuilderInfo},
    errors::{ParseResult, SyntaxErrors},
};

pub fn parse(derive_input: syn::DeriveInput) -> ParseResult<BuilderInfo> {
    use syn::spanned::Spanned;
    use syn::Data;

    let span = derive_input.span();

    let syn::DeriveInput {
        ident,
        generics,
        data,
        attrs,
        ..
    } = derive_input;

    match data {
        Data::Struct(struct_) => parse_builder_struct(struct_, ident, generics, attrs, span),
        _ => Err(vec![syn::Error::new(
            span,
            "Can only derive `Builder` for a struct",
        )]
        .into()),
    }
}

fn parse_builder_struct(
    struct_: syn::DataStruct,
    name: syn::Ident,
    generics: syn::Generics,
    attrs: Vec<syn::Attribute>,
    span: proc_macro2::Span,
) -> ParseResult<BuilderInfo> {
    use syn::Fields;

    let mut errors = SyntaxErrors::default();

    for attr in attributes_from_syn(attrs)? {
        match attr {
            BuilderAttribute::Required(tts) => {
                errors.add(tts, "required is only valid on a field");
            }
        }
    }

    let fields = match struct_.fields {
        Fields::Named(fields) => fields,
        _ => {
            errors.extend(vec![syn::Error::new(span, "only named fields are supported")].into());
            return Err(errors
                .finish()
                .expect_err("just added an error so there should be one"));
        }
    };
    let fields = fields
        .named
        .into_iter()
        .map(|f| match attributes_from_syn(f.attrs) {
            Ok(attrs) => (f.ident, f.ty, attrs),
            Err(e) => {
                errors.extend(e);
                (f.ident, f.ty, vec![])
            }
        })
        .collect();

    errors.finish()?;

    Ok(BuilderInfo {
        name,
        generics,
        fields,
    })
}

fn attributes_from_syn(attrs: Vec<syn::Attribute>) -> ParseResult<Vec<BuilderAttribute>> {
    use syn::parse2;
    let mut ours = Vec::new();
    let mut errs = Vec::new();

    let parsed_attrs = attrs.into_iter().filter_map(|attr| {
        if attr.path.is_ident("builder") {
            Some(parse2::<BuilderAttributeBody>(attr.tokens).map(|body| body.0))
        } else {
            None
        }
    });

    for attr in parsed_attrs {
        match attr {
            Ok(v) => ours.extend(v),
            Err(e) => errs.push(e),
        }
    }
    if errs.is_empty() {
        Ok(ours)
    } else {
        Err(errs.into())
    }
}
