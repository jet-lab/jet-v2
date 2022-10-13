use proc_macro::TokenStream;
use quote::quote;
use syn::{self, DeriveInput};

#[proc_macro_derive(BondTokenManager, attributes(bond_manager, token_program))]
pub fn bond_token_manager_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let mgr = impl_bond_manager_provider(&ast);
    let tkn = impl_token_program_provider(&ast);
    quote! { #mgr #tkn }.into()
}

fn impl_bond_manager_provider(ast: &DeriveInput) -> quote::__private::TokenStream {
    let name = &ast.ident;
    let lt = &ast.generics.lifetimes().next();
    let accessor = find_attr_path_as_accessor(&ast, "bond_manager").unwrap_or_default();
    quote! {
        impl<#lt> crate::utils::BondManagerProvider<#lt> for #name<#lt> {
            fn bond_manager(&self) -> anchor_lang::prelude::AccountLoader<#lt, crate::control::state::BondManager> {
                self #accessor.bond_manager.clone()
            }
        }
    }
}

fn impl_token_program_provider(ast: &DeriveInput) -> quote::__private::TokenStream {
    let name = &ast.ident;
    let lt = &ast.generics.lifetimes().next();
    let accessor = find_attr_path_as_accessor(&ast, "token_program").unwrap_or_default();
    quote! {
        impl<#lt> crate::utils::TokenProgramProvider<#lt> for #name<#lt> {
            fn token_program(&self) -> anchor_lang::prelude::Program<#lt, anchor_spl::token::Token> {
                self #accessor.token_program.clone()
            }
        }
    }
}

fn find_attr_path_as_accessor(
    ast: &DeriveInput,
    attr_name: &str,
) -> Option<quote::__private::TokenStream> {
    match &ast.data {
        syn::Data::Struct(data) => {
            for field in &data.fields {
                let field_name = field.ident.as_ref().unwrap().clone();
                for attr in &field.attrs {
                    if attr.path.segments[0].ident == attr_name {
                        let args: syn::Result<syn::Path> = attr.parse_args();
                        return Some(match args {
                            Ok(args) => {
                                let accessor = args.segments.into_iter();
                                quote! { .#field_name.#(#accessor).* }
                            }
                            Err(_) => quote! { .#field_name },
                        });
                    }
                }
            }
        }
        _ => return None,
    }

    None
}
