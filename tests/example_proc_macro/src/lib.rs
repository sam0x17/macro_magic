extern crate proc_macro;
use proc_macro::TokenStream;

use macro_magic::import_tokens;

#[proc_macro]
pub fn example_macro(_tokens: TokenStream) -> TokenStream {
    import_tokens!(example_crate::add2).into()
}
