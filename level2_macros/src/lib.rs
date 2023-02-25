use proc_macro::TokenStream;

use level1_macros::import_tokens;
use syn::{parse_macro_input, TypePath};

#[proc_macro]
pub fn import(tokens: TokenStream) -> TokenStream {
    //let path: TypePath = parse_macro_input!(tokens as TypePath);
    //let tokens = import_tokens!()
    tokens
}
