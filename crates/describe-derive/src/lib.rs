//! Derive macros for `fstools_describe`.

use darling::{ast::Data, util::Flag, FromDeriveInput, FromField};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, LitStr, Type};

#[proc_macro_derive(Describe, attributes(describe))]
pub fn describe_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let args = match DescribeInput::from_derive_input(&input) {
        Ok(args) => args,
        Err(err) => return err.write_errors().into(),
    };

    match expand_describe(args) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[derive(FromDeriveInput)]
#[darling(attributes(describe))]
struct DescribeInput {
    ident: Ident,
    data: Data<darling::util::Ignored, DescribeField>,
    #[darling(default, rename = "name")]
    node_name: Option<String>,
}

#[derive(FromField)]
#[darling(attributes(describe))]
struct DescribeField {
    ident: Option<Ident>,
    ty: Type,
    #[darling(default)]
    rename: Option<String>,
    #[darling(default)]
    skip: Flag,
    #[darling(default)]
    format: Option<String>,
    #[darling(default)]
    bytes: Flag,
}

fn expand_describe(input: DescribeInput) -> syn::Result<TokenStream2> {
    let ident = input.ident;
    let node_name = input.node_name.unwrap_or_else(|| ident.to_string());

    let fields = match input.data {
        Data::Struct(fields) => fields,
        Data::Enum(_) => {
            return Err(syn::Error::new(
                ident.span(),
                "Describe derive currently supports named structs only",
            ))
        }
    };

    let mut statements = Vec::new();

    for field in &fields.fields {
        let Some(field_ident) = field.ident.clone() else {
            return Err(syn::Error::new(
                ident.span(),
                "Describe derive requires named fields",
            ));
        };

        if field.skip.is_present() {
            continue;
        }

        let span = field_ident.span();
        let key = field
            .rename
            .clone()
            .unwrap_or_else(|| field_ident.to_string());
        let key_lit = LitStr::new(&key, span);

        let field_format = attribute_format(field, span)?;

        let stmt = match field_format {
            FieldFormat::Default => default_field_statement(&field_ident, &key_lit),
            FieldFormat::FormatString(format_lit) => {
                formatted_field_statement(&field_ident, &key_lit, &field.ty, format_lit)
            }
            FieldFormat::ByteSize => byte_size_field_statement(&field_ident, &key_lit, &field.ty),
        }?;

        statements.push(stmt);
    }

    let node_name_lit = LitStr::new(&node_name, Span::call_site());

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

enum FieldFormat {
    Default,
    FormatString(LitStr),
    ByteSize,
}

fn attribute_format(field: &DescribeField, span: Span) -> syn::Result<FieldFormat> {
    if field.bytes.is_present() && field.format.is_some() {
        return Err(syn::Error::new(
            span,
            "`bytes` cannot be combined with an explicit format",
        ));
    }

    if field.bytes.is_present() {
        Ok(FieldFormat::ByteSize)
    } else if let Some(format) = &field.format {
        Ok(FieldFormat::FormatString(LitStr::new(format, span)))
    } else {
        Ok(FieldFormat::Default)
    }
}

fn default_field_statement(field_ident: &Ident, key_lit: &LitStr) -> syn::Result<TokenStream2> {
    Ok(quote! {
        {
            let visitor = ::fstools_describe::FieldVisitor::new(#key_lit);
            match visitor.visit(&self.#field_ident)? {
                ::fstools_describe::FieldDescription::Attribute(attribute) => {
                    node.push_attribute(attribute);
                }
                ::fstools_describe::FieldDescription::Children(items) => {
                    for item in items {
                        node.push_child(item);
                    }
                }
                ::fstools_describe::FieldDescription::Skip => {}
            }
        }
    })
}

fn formatted_field_statement(
    field_ident: &Ident,
    key_lit: &LitStr,
    field_ty: &Type,
    format_lit: LitStr,
) -> syn::Result<TokenStream2> {
    if is_option_type(field_ty) {
        Ok(quote! {
            if let Some(value) = &self.#field_ident {
                let formatted = format!(#format_lit, value);
                node.push_attribute(::fstools_describe::DescribedAttribute::new(#key_lit, formatted));
            }
        })
    } else {
        Ok(quote! {
            {
                let value = &self.#field_ident;
                let formatted = format!(#format_lit, value);
                node.push_attribute(::fstools_describe::DescribedAttribute::new(#key_lit, formatted));
            }
        })
    }
}

fn byte_size_field_statement(
    field_ident: &Ident,
    key_lit: &LitStr,
    field_ty: &Type,
) -> syn::Result<TokenStream2> {
    if is_option_type(field_ty) {
        Ok(quote! {
            if let Some(value) = &self.#field_ident {
                let formatted = ::fstools_describe::format_byte_size((*value) as u64);
                node.push_attribute(::fstools_describe::DescribedAttribute::new(#key_lit, formatted));
            }
        })
    } else {
        Ok(quote! {
            {
                let formatted = ::fstools_describe::format_byte_size(self.#field_ident as u64);
                node.push_attribute(::fstools_describe::DescribedAttribute::new(#key_lit, formatted));
            }
        })
    }
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            return segment.ident == "Option";
        }
    }

    false
}
