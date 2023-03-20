use macro_magic_core::*;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn export_tokens(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    match export_tokens_internal(attr, tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn forward_tokens(tokens: TokenStream) -> TokenStream {
    match forward_tokens_internal(tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn import_tokens(tokens: TokenStream) -> TokenStream {
    match import_tokens_internal(tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn import_tokens_inner(tokens: TokenStream) -> TokenStream {
    match import_tokens_inner_internal(tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn forward_tokens_inner(tokens: TokenStream) -> TokenStream {
    match forward_tokens_inner_internal(tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn import_tokens_attr(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    println!("called");
    match import_tokens_attr_internal(attr, tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
