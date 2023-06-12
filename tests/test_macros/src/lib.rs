use derive_syn_parse::Parse;
use macro_magic::{mm_core::ForeignPath, *};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, spanned::Spanned, Error, Fields, Ident, Item, ItemMod, ItemStruct, Path,
};

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
    let mm_path = macro_magic::mm_core::macro_magic_root();
    quote! {
        #mm_path::forward_tokens! { #external_path, include_impl_inner }
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

#[derive(Parse)]
struct SomeOtherMacroArgs {
    forwarded_ident: Ident,
    item: Item,
}

#[proc_macro]
pub fn some_other_macro(tokens: TokenStream) -> TokenStream {
    let args = parse_macro_input!(tokens as SomeOtherMacroArgs);
    assert_eq!(
        args.forwarded_ident.to_token_stream().to_string(),
        "__private_macro_magic_tokens_forwarded",
    );
    assert_eq!(
        args.item.to_token_stream().to_string(),
        "struct SomeStruct { field1 : u32, field2 : bool, }"
    );
    let item = args.item;
    quote! {
        #[allow(unused)]
        #item
    }
    .into()
}

// as demonstrated here, `import_tokens_attr` can take a path or an expression that evaluates
// to something compatible with `Into<String>`
#[import_tokens_attr(format!("{}::export_mod::sub_mod::macro_magic", "middle_crate"))]
#[proc_macro_attribute]
pub fn distant_re_export_attr(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let imported_item = parse_macro_input!(attr as Item);
    let attached_item = parse_macro_input!(tokens as Item);
    let imported_item_str = imported_item.to_token_stream().to_string();
    let attached_item_str = attached_item.to_token_stream().to_string();
    quote! {
        const DISTANT_ATTR_ATTACHED_ITEM: &'static str = #attached_item_str;
        const DISTANT_ATTR_IMPORTED_ITEM: &'static str = #imported_item_str;
        #attached_item
    }
    .into()
}

#[import_tokens_proc(middle_crate::export_mod::sub_mod::macro_magic)]
#[proc_macro]
pub fn distant_re_export_proc(tokens: TokenStream) -> TokenStream {
    let imported_item = parse_macro_input!(tokens as Item);
    let imported_item_str = imported_item.to_token_stream().to_string();
    quote!(#imported_item_str).into()
}

#[import_tokens_attr(example_export::subpath)]
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

#[derive(Parse)]
struct CustomParsingA {
    foreign_path: syn::Path,
    _comma: syn::token::Comma,
    custom_path: syn::Path,
}

impl ToTokens for CustomParsingA {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.foreign_path.to_token_stream());
        tokens.extend(self._comma.to_token_stream());
        tokens.extend(self.custom_path.to_token_stream());
    }
}

impl ForeignPath for CustomParsingA {
    fn foreign_path(&self) -> &syn::Path {
        &self.foreign_path
    }
}

#[with_custom_parsing(CustomParsingA)]
#[import_tokens_attr]
#[proc_macro_attribute]
pub fn import_tokens_attr_with_custom_parsing_a(
    attr: TokenStream,
    tokens: TokenStream,
) -> TokenStream {
    let imported_item = parse_macro_input!(attr as Item);
    let attached_item = parse_macro_input!(tokens as Item);
    let imported_item_str = imported_item.to_token_stream().to_string();
    let attached_item_str = attached_item.to_token_stream().to_string();
    let custom_path_str = __custom_tokens.to_string();
    assert_eq!(
        imported_item_str,
        "struct CustomParsingStructForeign { field : bool, }"
    );
    assert_eq!(
        custom_path_str,
        "CustomParsingStructForeign, some :: cool :: path"
    );
    assert_eq!(
        attached_item_str,
        "struct CustomParsingStructLocal { field : u32, }"
    );
    quote! {
        #attached_item
    }
    .into()
}

/// we do this one to check that both orderings work
#[import_tokens_attr]
#[with_custom_parsing(CustomParsingA)]
#[proc_macro_attribute]
pub fn import_tokens_attr_with_custom_parsing_b(
    attr: TokenStream,
    tokens: TokenStream,
) -> TokenStream {
    let imported_item = parse_macro_input!(attr as Item);
    let attached_item = parse_macro_input!(tokens as Item);
    let imported_item_str = imported_item.to_token_stream().to_string();
    let attached_item_str = attached_item.to_token_stream().to_string();
    let custom_path_str = __custom_tokens.to_string();
    assert_eq!(
        imported_item_str,
        "struct CustomParsingStructForeign { field : bool, }"
    );
    assert_eq!(
        custom_path_str,
        "CustomParsingStructForeign, some :: cool :: path"
    );
    assert_eq!(
        attached_item_str,
        "struct CustomParsingStructLocal2 { field : u32, }"
    );
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

#[import_tokens_proc(example_export::subpath)]
#[proc_macro]
pub fn item_level_proc(tokens: TokenStream) -> TokenStream {
    let _imported_item = parse_macro_input!(tokens as Item);
    quote!(
        struct SomeInjectedStruct {}
    )
    .into()
}

#[import_tokens_attr]
#[proc_macro_attribute]
pub fn emit_foreign_path(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let path = __source_path.to_string();
    let foreign_item_str = attr.to_string();
    let item = parse_macro_input!(tokens as Item);
    quote! {
        const foreign_item_str: &'static str = #foreign_item_str;
        const emitted_path: &'static str = #path;
        #item
    }
    .into()
}

#[import_tokens_attr]
#[proc_macro_attribute]
pub fn combine_structs(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let foreign_struct = parse_macro_input!(attr as ItemStruct);
    let local_struct = parse_macro_input!(tokens as ItemStruct);
    let Fields::Named(local_fields) = local_struct.fields else {
        return Error::new(
            local_struct.fields.span(),
            "unnamed fields are not supported"
        ).to_compile_error().into()
    };
    let Fields::Named(foreign_fields) = foreign_struct.fields else {
        return Error::new(
            foreign_struct.fields.span(),
            "unnamed fields are not supported"
        ).to_compile_error().into()
    };
    let local_fields = local_fields.named.iter();
    let foreign_fields = foreign_fields.named.iter();
    let attrs = local_struct.attrs;
    let generics = local_struct.generics;
    let ident = local_struct.ident;
    let vis = local_struct.vis;
    quote! {
        #(#attrs)
        *
        #vis struct #ident<#generics> {
            #(#local_fields),
            *
            ,
            #(#foreign_fields),
            *
        }
    }
    .into()
}

#[import_tokens_proc]
#[proc_macro]
pub fn require(tokens: TokenStream) -> TokenStream {
    let external_mod = parse_macro_input!(tokens as ItemMod);
    let Some((_, stmts)) = external_mod.content else {
        return Error::new(
            external_mod.span(),
            "cannot import tokens from a file-based module since custom file-level \
            attributes are not yet supported by Rust"
        ).to_compile_error().into()
    };
    quote! {
        #(#stmts)
        *
    }
    .into()
}

export_tokens_alias!(custom_export_tokens);
