extern crate proc_macro;
use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Error, Item};

#[proc_macro_attribute]
pub fn export_tokens(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return Error::new(
            Span::call_site().into(),
            "#[export_tokens] does not take any arguments",
        )
        .to_compile_error()
        .into();
    }
    let tmp = tokens.clone();
    let item: Item = parse_macro_input!(tmp as Item);
    let source_code = tokens.to_string();
    quote! {
        #item
        const SOURCE_CODE: &'static str = #source_code;
    }
    .into()
}
