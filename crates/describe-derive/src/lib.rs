//! Derive macros for fstools_describe.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DataStruct, DeriveInput, Fields, Ident,
    LitStr, Result,
};

#[proc_macro_derive(Describe, attributes(describe))]
pub fn output_node_derive(_input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(_input as DeriveInput);

    match expand_output_node(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_output_node(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let ident = input.ident;
    let name = parse_container_name(&ident, &input.attrs)?;

    match input.data {
        Data::Struct(data) => expand_struct(&ident, &name, data),
        _ => Err(syn::Error::new(
            ident.span(),
            "Describe derive currently supports named structs only",
        )),
    }
}

fn expand_struct(
    ident: &Ident,
    node_name: &str,
    data: DataStruct,
) -> Result<proc_macro2::TokenStream> {
    let fields = match data.fields {
        Fields::Named(named) => named,
        _ => {
            return Err(syn::Error::new(
                ident.span(),
                "Describe derive currently supports named structs",
            ))
        }
    };

    let mut statements = Vec::new();

    for field in fields.named {
        let field_ident = field.ident.expect("named field");
        let config = DescribeField::parse(&field.attrs)?;
        let span = field_ident.span();

        let Some(role) = config.role else {
            continue;
        };

        match role {
            FieldRole::Attribute => {
                let key = config.rename.unwrap_or_else(|| field_ident.to_string());
                let key_lit = LitStr::new(&key, span);

                let stmt = if config.optional {
                    quote! {
                        if let Some(value) = &self.#field_ident {
                            node.push_attribute(::fstools_describe::DescribedAttribute::new(#key_lit, format!("{}", value)));
                        }
                    }
                } else {
                    quote! {
                        node.push_attribute(::fstools_describe::DescribedAttribute::new(#key_lit, format!("{}", self.#field_ident)));
                    }
                };

                statements.push(stmt);
            }
            FieldRole::Child => {
                let stmt = if config.optional {
                    quote! {
                        if let Some(value) = &self.#field_ident {
                            node.push_child(::fstools_describe::Describe::describe(value)?);
                        }
                    }
                } else {
                    quote! {
                        node.push_child(::fstools_describe::Describe::describe(&self.#field_ident)?);
                    }
                };

                statements.push(stmt);
            }
            FieldRole::Children => {
                let stmt = if config.optional {
                    quote! {
                        if let Some(values) = &self.#field_ident {
                            for value in values {
                                node.push_child(::fstools_describe::Describe::describe(value)?);
                            }
                        }
                    }
                } else {
                    quote! {
                        for value in &self.#field_ident {
                            node.push_child(::fstools_describe::Describe::describe(value)?);
                        }
                    }
                };

                statements.push(stmt);
            }
        }
    }

    let node_name_lit = LitStr::new(node_name, Span::call_site());

    Ok(quote! {
        impl ::fstools_describe::Describe for #ident {
            fn describe(&self) -> ::fstools_describe::Result<::fstools_describe::Description> {
                let mut node = ::fstools_describe::Description::new(#node_name_lit);
                #(#statements)*
                Ok(node)
            }
        }
    })
}

fn parse_container_name(ident: &Ident, attrs: &[Attribute]) -> Result<String> {
    let mut name = ident.to_string();

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("describe")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                let value: LitStr = meta.value()?.parse()?;
                name = value.value();
                Ok(())
            } else {
                Err(meta.error("unsupported describe attribute"))
            }
        })?;
    }

    Ok(name)
}

struct DescribeField {
    role: Option<FieldRole>,
    rename: Option<String>,
    optional: bool,
}

impl DescribeField {
    fn parse(attrs: &[Attribute]) -> Result<Self> {
        let mut result = DescribeField {
            role: None,
            rename: None,
            optional: false,
        };

        for attr in attrs.iter().filter(|attr| attr.path().is_ident("describe")) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("attribute") {
                    ensure_role(meta.path.span(), &mut result.role, FieldRole::Attribute)
                } else if meta.path.is_ident("child") {
                    ensure_role(meta.path.span(), &mut result.role, FieldRole::Child)
                } else if meta.path.is_ident("children") {
                    ensure_role(meta.path.span(), &mut result.role, FieldRole::Children)
                } else if meta.path.is_ident("rename") {
                    let value: LitStr = meta.value()?.parse()?;
                    result.rename = Some(value.value());
                    Ok(())
                } else if meta.path.is_ident("optional") {
                    result.optional = true;
                    Ok(())
                } else {
                    Err(meta.error("unsupported describe attribute"))
                }
            })?;
        }

        Ok(result)
    }
}

fn ensure_role(span: Span, slot: &mut Option<FieldRole>, role: FieldRole) -> Result<()> {
    if slot.is_some() {
        return Err(syn::Error::new(span, "multiple describe roles specified"));
    }

    *slot = Some(role);
    Ok(())
}

enum FieldRole {
    Attribute,
    Child,
    Children,
}
