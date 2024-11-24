use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Environment, attributes(parenv))]
pub fn derive_environment(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = input.ident;
    let Data::Struct(struct_data) = input.data else {
        panic!("environment parser can only be derived on structs");
    };
    let Fields::Named(fields) = struct_data.fields else {
        panic!("environment parser can only be derived on structs whose fields have names");
    };

    let (prefix, suffix) = parenv_mata_values(&input.attrs);

    let field_descs: Vec<_> = fields
        .named
        .iter()
        .map(|field| {
            let is_option = subty_if_name(&field.ty, "Option").is_some();

            let ident = field.ident.clone().unwrap();
            let ident_uppercase = ident.to_string().to_uppercase();
            let ident_uppercase = format!("{prefix}{ident_uppercase}{suffix}");

            let doc_comment = extract_doc_comment(field);

            if is_option {
                quote! {
                    [#ident_uppercase.bold().to_string(), #doc_comment.bright_magenta().to_string(), "[optional]".dimmed().to_string()]
                }
            } else {
                quote! {
                    [#ident_uppercase.bold().to_string(), #doc_comment.bright_magenta().to_string(), "".to_string()]
                }
            }
        })
        .collect();

    let field_names: Vec<_> = fields
        .named
        .iter()
        .map(|field| {
            let is_option = subty_if_name(&field.ty, "Option").is_some();
            let ident = field.ident.clone().unwrap();
            if is_option {
                quote! {
                    #ident: #ident
                }
            } else {
                quote! {
                    #ident: #ident.unwrap()
                }
            }
        })
        .collect();

    let parse_fields: Vec<_> = fields
        .named
        .into_iter()
        .enumerate()
        .map(|(i, field)| {
            let ident = field.ident.unwrap();
            let is_option = subty_if_name(&field.ty, "Option");

            let ident_uppercase = ident.to_string().to_uppercase();
            let ident_uppercase = format!("{prefix}{ident_uppercase}{suffix}");
            let parse_ident = format_ident!("parse_{ident}");

            if let Some(inner_typ) = is_option {
                quote! {
                    fn #parse_ident() -> ::std::option::Option<::parenv::miette::Result<#inner_typ>> {
                        ::std::env::var(#ident_uppercase)
                            .ok()
                            .map(|f| {
                                f.parse::<#inner_typ>()
                                    .into_diagnostic()
                                    .wrap_err_with(||
                                        format!(
                                            "I couldn't parse the value '{}' provided by the environment variable {}.",
                                            f.red().bold(),
                                            #ident_uppercase.red().bold()
                                        )
                                    )
                            })
                    }

                    let #ident = match #parse_ident() {
                        ::std::option::Option::Some(res) => {
                            match res {
                                ::std::result::Result::Err(err) => {
                                    errors[#i] = ::std::option::Option::Some(err);
                                    ::std::option::Option::None
                                },
                                ::std::result::Result::Ok(val) => ::std::option::Option::Some(val),
                            }
                        },
                        ::std::option::Option::None => ::std::option::Option::None
                    };
                }
            } else {
                let ident_typ = field.ty;

                quote! {
                    fn #parse_ident() -> ::parenv::miette::Result<#ident_typ> {
                        let #ident = ::std::env::var(#ident_uppercase)
                            .into_diagnostic()
                            .wrap_err_with(||
                                format!(
                                    "I couldn't find the environment variable {}.",
                                    #ident_uppercase.red().bold()
                                )
                            )?;
                        let #ident: #ident_typ = #ident.parse()
                            .into_diagnostic()
                            .wrap_err_with(||
                                format!(
                                    "I couldn't parse the value '{}' provided by the environment variable {}.",
                                    #ident.red().bold(),
                                    #ident_uppercase.red().bold()
                                )
                            )?;
                        Ok(#ident)
                    }

                    let #ident = match #parse_ident() {
                        ::std::result::Result::Err(err) => {
                            errors[#i] = Some(err);
                            ::std::option::Option::None
                        },
                        ::std::result::Result::Ok(val) => ::std::option::Option::Some(val),
                    };
                }
            }
        })
        .collect();

    let fields_len = field_names.len();
    let nones: Vec<_> = std::iter::repeat_n(quote! { None }, fields_len).collect();

    let expanded = quote! {
        impl #ident {
            fn parse() -> Self {
                use ::parenv::miette::{IntoDiagnostic, WrapErr};
                use ::parenv::owo_colors::OwoColorize;

                let mut errors: [
                    ::std::option::Option<::parenv::miette::Report>;
                    #fields_len
                ] = [#(#nones),*];

                #(#parse_fields)*

                let there_is_a_some = errors.iter().any(|e| e.is_some());
                if there_is_a_some {
                    let crate_name = ::std::env!("CARGO_PKG_NAME").green();

                    ::std::println!("I, {crate_name}, expect the following environment variables.\n");

                    let items: [[::std::string::String; 3]; #fields_len] = [ #(#field_descs),* ];

                    let mut max_widths: [usize; 3] = [0; 3];
                    for col in 0..3 {
                        max_widths[col] = items
                            .iter()
                            .map(|row| row[col].len())
                            .max()
                            .unwrap_or(0);
                    }

                    for row in items {
                        ::std::print!("    ");
                        for (value, width) in row.iter().zip(&max_widths) {
                            ::std::print!("{:<width$}    ", value, width = width);
                        }
                        ::std::println!();
                    }

                    ::std::println!();

                    ::std::println!("I faced an error parsing the following environment variables.\n");

                    for err in errors {
                        if let ::std::option::Option::Some(e) = err {
                            ::std::println!("{:?}", e);
                        }
                    }

                    ::std::process::exit(1);
                }

                Self {
                    #(#field_names),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn parenv_mata_values(attrs: &[syn::Attribute]) -> (String, String) {
    attrs
        .iter()
        .find_map(|attr| match attr.meta {
            syn::Meta::List(ref val) if val.path.is_ident("parenv") => {
                let mut tokens = val.tokens.clone().into_iter();

                let mut prefix = None;
                let mut suffix = None;

                for _ in 0..2 {
                    let ident = tokens.next().and_then(|t| match t {
                        TokenTree::Ident(ident) => Some(ident.to_string()),
                        _ => None,
                    })?;

                    if ident == "prefix" {
                        prefix = get_parenv_value(&mut tokens);
                    }

                    if ident == "suffix" {
                        suffix = get_parenv_value(&mut tokens);
                    }

                    let _ = tokens.next().and_then(|t| match t {
                        TokenTree::Punct(punct) if punct.as_char() == ',' => Some(()),
                        _ => None,
                    });
                }

                Some((prefix.unwrap_or_default(), suffix.unwrap_or_default()))
            }
            _ => None,
        })
        .unwrap_or((String::default(), String::default()))
}

/// Parses this structure
/// = value
fn get_parenv_value(tokens: &mut proc_macro2::token_stream::IntoIter) -> Option<String> {
    tokens.next().and_then(|t| match t {
        TokenTree::Punct(punct) if punct.as_char() == '=' => Some(()),
        _ => None,
    })?;
    let value = tokens.next().and_then(|t| match t {
        TokenTree::Literal(lit) => Some(lit.to_string()),
        _ => None,
    })?;

    Some(if value.is_empty() {
        value
    } else {
        value[1..(value.len() - 1)].to_owned()
    })
}

fn extract_doc_comment(field: &syn::Field) -> String {
    field
        .attrs
        .iter()
        .find_map(|attr| match attr.meta.clone() {
            syn::Meta::NameValue(val) if val.path.is_ident("doc") => Some(val.value),
            _ => None,
        })
        .and_then(|expr| match expr {
            syn::Expr::Lit(lit) => Some(lit.lit),
            _ => None,
        })
        .and_then(|lit| match lit {
            syn::Lit::Str(lit_str) => Some(lit_str.value()),
            _ => None,
        })
        .map(|doc| doc.trim().to_owned())
        .unwrap_or_default()
}

pub(crate) fn subty_if_name<'a>(ty: &'a syn::Type, name: &str) -> Option<&'a syn::Type> {
    subty_if(ty, |seg| seg.ident == name)
}

fn subty_if<F>(ty: &syn::Type, f: F) -> Option<&syn::Type>
where
    F: FnOnce(&syn::PathSegment) -> bool,
{
    use syn::{GenericArgument, PathArguments::AngleBracketed};

    only_last_segment(ty)
        .filter(|segment| f(segment))
        .and_then(|segment| {
            if let AngleBracketed(args) = &segment.arguments {
                only_one(args.args.iter()).and_then(|genneric| {
                    if let GenericArgument::Type(ty) = genneric {
                        Some(ty)
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        })
}

fn only_last_segment(mut ty: &syn::Type) -> Option<&syn::PathSegment> {
    use syn::{Path, Type, TypePath};

    while let Type::Group(syn::TypeGroup { elem, .. }) = ty {
        ty = elem;
    }
    match ty {
        Type::Path(TypePath {
            qself: None,
            path:
                Path {
                    leading_colon: None,
                    segments,
                },
        }) => only_one(segments.iter()),

        _ => None,
    }
}

fn only_one<I, T>(mut iter: I) -> Option<T>
where
    I: Iterator<Item = T>,
{
    iter.next().filter(|_| iter.next().is_none())
}
