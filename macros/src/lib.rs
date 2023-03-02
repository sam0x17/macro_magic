extern crate proc_macro;
use proc_macro::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::{fs::read_to_string, fs::OpenOptions, io::Write};
use syn::{parse_macro_input, spanned::Spanned, Error, Ident, Item, Path, TypePath};

const REFS_DIR: &'static str = env!("REFS_DIR");

fn write_file<T: Into<String>>(path: &std::path::Path, source: T) -> std::io::Result<()> {
    let mut f = OpenOptions::new().write(true).create(true).open(path)?;
    f.write_all(source.into().as_bytes())?;
    f.flush()?;
    Ok(())
}

fn get_const_name(name: String) -> String {
    format!("__EXPORT_TOKENS__{}", name.replace(" ", "").to_uppercase())
}

fn get_const_path(path: &TypePath) -> Result<Path, Error> {
    let mut path = path.path.clone();
    let Some(mut last) = path.segments.last_mut() else {
        return Err(Error::new(path.span(), "Empty paths cannot be expanded!"))
    };
    last.ident = Ident::new(
        get_const_name(last.to_token_stream().to_string()).as_str(),
        Span::call_site().into(),
    );
    Ok(path)
}

/// The `#[export_tokens]` attribute can be attached to any [`syn::Item`]-compatible source
/// code item, with the exception of [`Item::ForeignMod`], [`Item::Impl`], [`Item::Macro`],
/// [`Item::Use`], and [`Item::Verbatim`]. Attaching to an item will "export" that item so that
/// it can be imported elsewhere by name via the [`import_tokens!`] macro.
///
/// For example, this would export a function named `foo`:
/// ```ignore
/// use macro_magic::export_tokens;
///
/// #[export_tokens]
/// fn foo(a: i64, b: i64) -> i64 {
///     a * a + b * b
/// }
/// ```
///
/// and this would export a module named `bar`:
/// ```ignore
/// use macro_magic::export_tokens;
///
/// #[export_tokens]
/// mod bar {
///     // ...
/// }
/// ```
///
/// You can also specify a path as an argument to `#[export_tokens]` as follows:
/// ```ignore
/// use macro_magic::export_tokens;
///
/// #[export_tokens(foo::bar::fizz_buzz)]
/// fn fizz_buzz() {
///     // ...
/// }
/// ```
///
/// This path will be used to disambiguate `fizz_buzz` from other `fizz_buzz` items that may
/// have been exported from different paths when you use [`import_tokens!`] to perform an
/// indirect import. Direct imports only make use of the last segment of this name, if it is
/// specified, while indirect imports will use the whole path.
///
/// Also note that direct imports are subject to visibility restrictions (i.e. they won't work
/// if you aren't in a public module), whereas indirect imports completely bypass visibility
/// restrictions because of how they are implemented internally.
///
/// ## Expansion
///
/// ```rust
/// #[export_tokens]
/// fn foo_bar(a: u32) -> u32 {
///     a * 2
/// }
/// ```
///
/// expands to:
///
/// ```rust
/// #[allow(dead_code)]
/// fn foo_bar(a: u32) -> u32 {
///     a * 2
/// }
/// #[allow(dead_code)]
/// #[doc(hidden)]
/// pub const __EXPORT_TOKENS__FOO_BAR: &'static str = "fn foo_bar(a : u32) -> u32 { a * 2 }";
/// ```
///
/// See the documentation for [`import_tokens!`] for more information and a full example of
/// exporting and importing tokens. README.md also contains valuable information.
#[proc_macro_attribute]
pub fn export_tokens(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let tmp = tokens.clone();
    let item: Item = parse_macro_input!(tmp as Item);
    let ident = match item.clone() {
        Item::Const(item_const) => item_const.ident,
        Item::Enum(item_enum) => item_enum.ident,
        Item::ExternCrate(item_extern_crate) => item_extern_crate.ident,
        Item::Fn(item_fn) => item_fn.sig.ident,
        Item::ForeignMod(item_foreign_mod) => {
            return Error::new(
                item_foreign_mod.span(),
                "#[export_tokens] cannot be applied to a foreign module",
            )
            .to_compile_error()
            .into()
        }
        Item::Impl(item_impl) => {
            return Error::new(
                item_impl.span(),
                "#[export_tokens] cannot be applied to an impl",
            )
            .to_compile_error()
            .into()
        }
        Item::Macro(item_macro) => match item_macro.ident {
            Some(ident) => ident,
            None => {
                return Error::new(
                    item_macro.span(),
                    "#[export_tokens] cannot be applied to unnamed decl macros",
                )
                .to_compile_error()
                .into()
            }
        },
        Item::Macro2(item_macro2) => item_macro2.ident,
        Item::Mod(item_mod) => item_mod.ident,
        Item::Static(item_static) => item_static.ident,
        Item::Struct(item_struct) => item_struct.ident,
        Item::Trait(item_trait) => item_trait.ident,
        Item::TraitAlias(item_trait_alias) => item_trait_alias.ident,
        Item::Type(item_type) => item_type.ident,
        Item::Union(item_union) => item_union.ident,
        Item::Use(item_use) => {
            return Error::new(
                item_use.span(),
                "#[export_tokens] cannot be applied to a use declaration",
            )
            .to_compile_error()
            .into()
        }
        _ => {
            return Error::new(
                item.span(),
                "#[export_tokens] cannot be applied to this item",
            )
            .to_compile_error()
            .into()
        }
    };
    let const_name = get_const_name(ident.to_string());
    let const_ident = Ident::new(const_name.as_str(), Span::call_site().into());
    let source_code = tokens.to_string();

    use std::path::Path;
    let refs_dir = Path::new(REFS_DIR);
    assert!(refs_dir.exists());

    if !attr.is_empty() {
        let export_path = parse_macro_input!(attr as TypePath);
        let fname = export_path
            .path
            .to_token_stream()
            .to_string()
            .replace("::", "-")
            .replace(" ", "");
        write_file(&refs_dir.join(fname), &source_code).unwrap();
        // do error handling
    }
    quote! {
        #[allow(dead_code)]
        #item
        #[doc(hidden)]
        #[allow(dead_code)]
        pub const #const_ident: &'static str = #source_code;
    }
    .into()
}

// Expands to a `TokenStream2` representing the tokens of the item at the specified path
#[proc_macro]
pub fn import_tokens(tokens: TokenStream) -> TokenStream {
    let path = parse_macro_input!(tokens as TypePath);
    let path = match get_const_path(&path) {
        Ok(path) => path,
        Err(e) => return e.to_compile_error().into(),
    };
    quote!(#path.parse::<::macro_magic::__private::TokenStream2>().unwrap()).into()
}

#[proc_macro]
pub fn import_tokens_indirect(tokens: TokenStream) -> TokenStream {
    let path = parse_macro_input!(tokens as TypePath);
    let fname = path
        .to_token_stream()
        .to_string()
        .replace("::", "-")
        .replace(" ", "");
    let fpath = std::path::Path::new(REFS_DIR).join(fname);
    if let Ok(source) = read_to_string(fpath) {
        quote!(#source.parse::<::macro_magic::__private::TokenStream2>().unwrap()).into()
    } else {
        Error::new(
            path.span(),
            "Indirectly importing the specified item failed. Make \
             sure the path is correct and the crate the item appears \
             in is being compiled as part of this workspace.",
        )
        .to_compile_error()
        .into()
    }
}

/// This convenient macro can be used to publicly re-export an item that has been exported via
/// [`macro@export_tokens`] when doing direct imports. See the documentation for
/// [`macro@export_tokens`] and [`import_tokens!`] for more information.
///
/// For example, assume in the module `my::cool_module` you have the following code:
/// ```ignore
/// pub mod cool_module {
///     use macro_magic::*;
///
///     #[export_tokens]
///     trait MyTrait {
///         fn some_behavior() -> String;
///         type SomeType;
///     }
/// }
/// ```
///
/// In another module or crate that has `my::cool_module` as a dependency, you could then do
/// something like this:
/// ```ignore
/// use macro_magic::re_export_tokens_const;
///
/// re_export_tokens_const!(my::cool_module::MyTrait);
/// ```
///
/// Now the `MyTrait` tokens will be available from the context where you called
/// `re_export_tokens_const` and can be accessed using direct imports via [`import_tokens!`],
/// for example if the re-export module/crate was called `other_crate::re_exports`, you could
/// then add that as a dependency to another crate and import the `my_item` tokens from that
/// crate like so:
///
/// ```ignore
/// use macro_magic::import_tokens;
/// use other_crate::re_exports::*; // this brings the re-exports into scope
/// use proc_macro2::TokenStream as TokenStream2;
///
/// #[proc_macro]
/// pub fn my_proc_macro(_tokens: TokenStream) -> TokenStream {
///     // ...
///
///     let my_trait_tokens: TokenStream2 = import_tokens!(MyTrait);
///
///     // can also do it this way if we wanted to avoid the `use` statement above:
///     let my_trait_tokens: TokenStream2 = import_tokens!(other_crate::re_exports::MyTrait);
///
///     // ...
///
///     my_item_tokens.into()
/// }
/// ```
///
/// So in other words, this is just a way of cleanly re-exporting the internal token constants
/// that are created by [`macro@export_tokens`] to make them accessible elsewhere.
///
/// ## Expansion
///
/// This code:
/// ```ignore
/// use macro_magic::re_export_tokens_const;
///
/// re_export_tokens_const!(my::cool_module::MyTrait);
/// ```
///
/// Would expand to the following:
/// ```ignore
/// use macro_magic::re_export_tokens_const;
/// #[doc(hidden)]
/// pub use example_crate::cool_module::__EXPORT_TOKENS__MYTRAIT;
/// ```
///
/// Notice that the actual item under the hood is a `const`. &'static str`.
#[proc_macro]
pub fn re_export_tokens_const(tokens: TokenStream) -> TokenStream {
    let path = match get_const_path(&parse_macro_input!(tokens as TypePath)) {
        Ok(path) => path,
        Err(e) => return e.to_compile_error().into(),
    };
    quote! {
        #[doc(hidden)]
        pub use #path;
    }
    .into()
}
