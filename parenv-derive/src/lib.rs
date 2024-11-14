use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Environment, attributes(prompt))]
pub fn derive_environment(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = input.ident;
    let Data::Struct(struct_data) = input.data else {
        panic!("environment parser can only be derived on structs");
    };
    let Fields::Named(fields) = struct_data.fields else {
        panic!("environment parser can only be derived on structs whose fields have names");
    };

    let field_descs: Vec<_> = fields
        .named
        .iter()
        .map(|a| {
            let ident = a.ident.clone().unwrap();
            let ident_uppercase = ident.to_string().to_uppercase();

            let doc_comment = a
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
                .unwrap_or_else(|| "An expected environment variable".to_string());
            let doc_comment = doc_comment.trim();

            quote! {
                ::std::println!("    {}: {}", #ident_uppercase.bold(), #doc_comment.bright_magenta());
            }
        })
        .collect();

    let field_names: Vec<_> = fields
        .named
        .iter()
        .map(|a| {
            let ident = a.ident.clone().unwrap();
            quote! {
                #ident: #ident.unwrap()
            }
        })
        .collect();

    let parse_fields: Vec<_> = fields
        .named
        .into_iter()
        .enumerate()
        .map(|(i, a)| {
            let ident = a.ident.unwrap();
            let ident_typ = a.ty;
            let ident_uppercase = ident.to_string().to_uppercase();
            let parse_ident = format_ident!("parse_{ident}");

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
                    Err(err) => {
                        errors[#i] = Some(err);
                        None
                    },
                    Ok(val) => Some(val),
                };
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
                    let all_errors_are_some = errors.iter().all(|e| e.is_some());

                    let crate_name = ::std::env!("CARGO_PKG_NAME").green();

                    if all_errors_are_some {
                        ::std::println!("I, {crate_name}, expect the following environment variables.\n");
                        #(#field_descs)*
                        ::std::println!();
                    }

                    ::std::println!("I faced an error parsing the following environment variables.\n");

                    for err in errors {
                        if let Some(e) = err {
                            println!("{:?}", e);
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
