extern crate proc_macro;
use proc_macro::TokenStream;

use macro_magic::*;

#[proc_macro]
pub fn example_macro(_tokens: TokenStream) -> TokenStream {
    import_tokens!(example_crate::add2).into()
}

#[proc_macro]
pub fn example_macro2(_tokens: TokenStream) -> TokenStream {
    import_tokens!(example_crate::cool_types).into()
}

#[cfg(feature = "indirect")]
#[proc_macro]
pub fn example_macro3(_tokens: TokenStream) -> TokenStream {
    import_tokens_indirect!(example_crate2::mult).into()
}

#[cfg(feature = "indirect")]
#[proc_macro]
pub fn example_macro4(_tokens: TokenStream) -> TokenStream {
    import_tokens_indirect!(BadBad<T>).into()
}

#[cfg(feature = "indirect")]
#[proc_macro]
pub fn read_namespace_test_red(tokens: TokenStream) -> TokenStream {
    let items = read_namespace!(foo_bar::red).unwrap();
    assert_eq!(items.len(), 3);
    tokens
}

#[cfg(feature = "indirect")]
#[proc_macro]
pub fn read_namespace_test_green(tokens: TokenStream) -> TokenStream {
    let items = read_namespace!(foo_bar::red::green).unwrap();
    assert_eq!(items.first().unwrap().0, "max_f64");
    assert_eq!(items.len(), 4);
    tokens.into()
}

#[cfg(feature = "indirect")]
#[proc_macro]
pub fn read_namespace_test_foo_bar(tokens: TokenStream) -> TokenStream {
    let items = read_namespace!(foo_bar).unwrap();
    assert_eq!(items.len(), 0);
    tokens.into()
}
