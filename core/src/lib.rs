use derive_syn_parse::Parse;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::Nothing,
    parse_macro_input,
    token::{Brace, Comma},
    FnArg, Ident, Item, ItemFn, Pat, Path, Token, Visibility,
};

pub fn export_tokens_macro_ident(ident: &Ident) -> Ident {
    let ident_string = format!("__export_tokens_tt_{}", ident.to_token_stream().to_string());
    Ident::new(ident_string.as_str(), Span::call_site())
}
