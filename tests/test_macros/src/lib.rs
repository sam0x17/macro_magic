use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemMod, Path};

/// An example proc macro built on top of `import_tokens_internal`.
///
/// ## Example
///
/// ```ignore
/// #[include_impl(path::to::exported::impl_block)]
/// mod my_module {
///     // ...
/// }
/// ```
/// would result in all of the items within the specified `impl` getting reproduced/imported
/// within the module the attribute is attached to, possibly resulting in a compiler error if
/// the specified `impl` block has generics.
#[proc_macro_attribute]
pub fn include_impl(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let external_path = parse_macro_input!(attr as Path);
    let _item_mod = parse_macro_input!(tokens as ItemMod);
    quote! {
        ::macro_magic::forward_tokens!(#external_path, include_impl_inner);
    }
    .into()
}

#[proc_macro]
pub fn include_impl_inner(tokens: TokenStream) -> TokenStream {
    println!("GOT TOKENS: {}", tokens.to_string());
    quote!().into()
}

#[proc_macro]
pub fn some_macro(tokens: TokenStream) -> TokenStream {
    let source_path = parse_macro_input!(tokens as Path);
    let fin = quote! {
        forward_tokens!(#source_path, test_macros::some_other_macro);
    };
    //println!("final: {}", fin.to_string());
    fin.into()
}

#[proc_macro]
pub fn some_other_macro(_tokens: TokenStream) -> TokenStream {
    //println!("tokens: {}", tokens.to_string());
    quote!().into()
}
