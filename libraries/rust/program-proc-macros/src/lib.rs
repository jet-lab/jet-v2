use proc_macro::TokenStream;
use quote::quote;
use syn::{self, DeriveInput};

mod mem;

/// Use the "aligns" or "size" options to ensure memory and storage safety with state structs or enums.
///
/// *aligns*: Ensure struct can be included in a parent struct that is packed (e.g. anchor's zero_copy)
///           without messing up the parent's alignment
///           *Important*: This does not guarantee alignment within this struct!
///
/// *size: usize*: Enforces that the struct is a specific size
///
/// For example, decorate a struct with any of these attributes:
/// #[assert_size(128, aligns)
/// #[assert_size(128)
/// #[assert_size(aligns)
/// #[assert_size(aligns, 128)
#[proc_macro_attribute]
pub fn assert_size(args: TokenStream, input_struct: TokenStream) -> TokenStream {
    mem::handler(args.into(), input_struct.into()).into()
}

/// Implements MarketTokenManager by implementing MarketProvider and
/// TokenProgramProvider.
///
/// By default, this expects fields named token_program and market with
/// the appropriate types. If either field is missing, then the data must be
/// nested. Annotate the nesting field with attribute #[token_manager] if that
/// field contains a token_manager. If further nested, and the field contains a
/// sub-field called "subfield" which contains the token_manager, mark the field
/// with #[token_manager(subfield)]. If token_manager is another level deep
/// within "subsubfield", use #[token_manager(subfield::subsubfield)]. There is
/// no limit to the accessible depth.
///
/// Examples
///
/// ```ignore
/// #[derive(MarketTokenManager)]
/// struct BaseCase<'info> {
///     market: AccountLoader<'info, Market>,
///     token_program: Program<'info, Token>,
/// }
/// ```
///
/// ```ignore
/// #[derive(MarketTokenManager)]
/// struct Top<'info> {
///     #[market]
///     nested: Bottom<'info>,
///     token_program: Program<'info, Token>,
/// }
///
/// struct Bottom<'info> {
///     market: AccountLoader<'info, Market>,
/// }
/// ```
///
/// ```ignore
/// #[derive(MarketTokenManager)]
/// struct Top<'info> {
///     #[market(mid_two)]
///     #[token_program(mid_two::bottom)]
///     mid_one: MiddleOne<'info>,
/// }
///
/// struct MiddleOne<'info> {
///     mid_two: MiddleTwo<'info>,
/// }
///
/// struct MiddleTwo<'info> {
///     market: AccountLoader<'info, Market>,
///     bottom: Bottom<'info>,
/// }
///
/// struct Bottom<'info> {
///     token_program: Program<'info, Token>,
/// }
/// ```
#[proc_macro_derive(MarketTokenManager, attributes(market, token_program))]
pub fn market_token_manager_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let mgr = impl_market_provider(&ast);
    let tkn = impl_token_program_provider(&ast);
    quote! { #mgr #tkn }.into()
}

fn impl_market_provider(ast: &DeriveInput) -> quote::__private::TokenStream {
    let name = &ast.ident;
    let lt = &ast.generics.lifetimes().next();
    let accessor = find_attr_path_as_accessor(ast, "market").unwrap_or_default();
    quote! {
        impl<#lt> crate::market_token_manager::MarketProvider<#lt> for #name<#lt> {
            fn market(&self) -> anchor_lang::prelude::AccountLoader<#lt, crate::control::state::Market> {
                self #accessor.market.clone()
            }
        }
    }
}

fn impl_token_program_provider(ast: &DeriveInput) -> quote::__private::TokenStream {
    let name = &ast.ident;
    let lt = &ast.generics.lifetimes().next();
    let accessor = find_attr_path_as_accessor(ast, "token_program").unwrap_or_default();
    quote! {
        impl<#lt> crate::market_token_manager::TokenProgramProvider<#lt> for #name<#lt> {
            fn token_program(&self) -> anchor_lang::prelude::Program<#lt, anchor_spl::token::Token> {
                self #accessor.token_program.clone()
            }
        }
    }
}

/// If `ast` is a struct, this will find the first field with an attribute named
/// `attr_name`, and return a TokenStream representing the code that you would
/// use to access a field with the name `attr_name` within the annotated field
/// of this struct. It will also include any path passed as an argument as
/// intermediary fields.
///
/// ```ignore
/// /// Searching this struct for "thing" returns ".bar.thing"
/// struct Foo {
///     #[thing]
///     bar: Bar,
/// }
/// ```
///
/// ```ignore
/// /// Searching this struct for "thing" returns ".bar.some.nesting.thing"
/// struct Foo {
///     #[thing(some::nesting)]
///     bar: Bar,
/// }
/// ```
///
/// Returns None if not a struct or there is no field with the attribute.
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
