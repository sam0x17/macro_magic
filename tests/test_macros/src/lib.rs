use derive_syn_parse::Parse;
use macro_magic::core::*;
use macro_magic::*;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemMod, Path, Stmt};

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
    let import_line = match import_tokens_internal(quote!(let imported_tokens = #external_path)) {
        Ok(line) => line,
        Err(err) => return err.to_compile_error().into(),
    };
    let _item_mod = parse_macro_input!(tokens as ItemMod);
    quote! {
        ::macro_magic::core::execute_callback! {
            ::test_macros::include_impl_inner,
            {
                #import_line
                panic!("hey");
            }
        }
    }
    .into()
}

#[proc_macro]
pub fn include_impl_inner(_tokens: TokenStream) -> TokenStream {
    // let mut st = tokens.to_string();
    // st = st.replace("{", "").replace("}", "");
    // let tokens = st.parse().unwrap();
    //tokens
    quote!().into()
}

#[derive(Parse)]
struct IncludeImplInnerArgs {
    import_line: Stmt,
    item_mod: ItemMod,
}
