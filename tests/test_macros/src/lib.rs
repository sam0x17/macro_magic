use macro_magic::*;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Item, ItemMod, Path};

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
        ::macro_magic::forward_tokens! { #external_path, include_impl_inner }
    }
    .into()
}

#[proc_macro]
pub fn include_impl_inner(_tokens: TokenStream) -> TokenStream {
    // println!("GOT TOKENS: {}", tokens.to_string());
    quote!().into()
}

#[proc_macro]
pub fn some_macro(tokens: TokenStream) -> TokenStream {
    let source_path = parse_macro_input!(tokens as Path);
    quote! {
        forward_tokens!(#source_path, test_macros::some_other_macro);
    }
    .into()
}

#[proc_macro]
pub fn some_other_macro(tokens: TokenStream) -> TokenStream {
    // println!("tokens: {}", tokens.to_string());
    let item = parse_macro_input!(tokens as Item);
    assert_eq!(
        item.to_token_stream().to_string(),
        "struct SomeStruct { field1 : u32, field2 : bool, }"
    );
    quote! {
        #[allow(unused)]
        #item
    }
    .into()
}

#[proc_macro_attribute]
pub fn test_attr(_attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let _item = parse_macro_input!(tokens as Item);
    quote! {
        macro_magic::expand_item_safe! {
            fn item() {}
        }
    }
    .into()
}

#[import_tokens_attr]
#[proc_macro_attribute]
pub fn test_tokens_attr1(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let imported_item = parse_macro_input!(attr as Item);
    let attached_item = parse_macro_input!(tokens as Item);
    let imported_item_str = imported_item.to_token_stream().to_string();
    let attached_item_str = attached_item.to_token_stream().to_string();
    assert_eq!(imported_item_str, "struct AnotherStruct { field1 : u32, }");
    assert_eq!(
        attached_item_str,
        "pub mod hunter { pub fn stuff() { println! (\"things\") ; } }"
    );
    quote! {
        #attached_item
    }
    .into()
}

#[import_tokens_attr]
#[proc_macro_attribute]
pub fn test_tokens_attr2(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let imported_item = parse_macro_input!(attr as Item);
    let attached_item = parse_macro_input!(tokens as Item);
    let imported_item_str = imported_item.to_token_stream().to_string();
    let attached_item_str = attached_item.to_token_stream().to_string();
    assert_eq!(
        imported_item_str,
        "impl FooBarTrait for FooBarStruct\n{\n    fn foo(n : u32) -> u32 { n + 1 } \
        fn bar(n : i32) -> i32 { n - 1 } fn\n    fizz(v : bool) -> bool { ! v }\n}"
    );
    assert_eq!(attached_item_str, "struct LocalItemStruct {}");
    quote! {
        #attached_item
    }
    .into()
}

#[proc_macro_attribute]
#[import_tokens_attr]
pub fn test_tokens_attr_direct_import(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let imported_item = parse_macro_input!(attr as Item);
    let attached_item = parse_macro_input!(tokens as Item);
    quote! {
        #imported_item
        #attached_item
    }
    .into()
}

#[import_tokens_proc]
#[proc_macro]
pub fn example_tokens_proc(tokens: TokenStream) -> TokenStream {
    let imported_item = parse_macro_input!(tokens as Item);
    let item_as_string = imported_item.to_token_stream().to_string();
    quote!(#item_as_string).into()
}
