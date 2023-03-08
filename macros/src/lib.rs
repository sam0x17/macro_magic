extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, TypePath};

use macro_magic_core::*;

/// This attribute can be attached to any [`syn::Item`]-compatible source code item, with the
/// exception of [`ForeignMod`](`syn::ItemForeignMod`), [`Impl`](`syn::ItemImpl`),
/// [`Macro`](`syn::ItemMacro`), [`Use`](`syn::ItemUse`), and
/// [`Verbatim`](`syn::Item::Verbatim`). Attaching to an item will "export" that item so that
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
/// You can also specify a path as an argument to [`#[export_tokens]`](`macro@export_tokens`)
/// as follows:
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
/// ```ignore
/// #[export_tokens]
/// fn foo_bar(a: u32) -> u32 {
///     a * 2
/// }
/// ```
///
/// expands to:
///
/// ```ignore
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
    let (item, const_decl) = match export_tokens_internal(tokens, attr, "#[export_tokens]") {
        Ok((item, const_decl)) => (item, const_decl),
        Err(e) => return e.to_compile_error().into(),
    };
    quote! {
        #[allow(dead_code)]
        #item
        #[doc(hidden)]
        #[allow(dead_code)]
        #const_decl
    }
    .into()
}

/// This macro is the primary way to bring exported tokens into scope in your proc macros
/// (though it can also be used in non-proc-macro contexts, and is based on
/// [`TokenStream2`](syn::__private::TokenStream2) for this purpose).
///
/// This approach is called a "direct import" and requires the source and target to be in the
/// same crate, or requires that the source crate is a dependency of the target crate. For a
/// less restrictive approach, see [`import_tokens_indirect!`].
///
/// Suppose you have exported tokens using the [`#[export_tokens]`](`macro@export_tokens`)
/// attribute macro as follows:
///
/// ```ignore
/// pub mod my_module {
///     use macro_magic::*;
///
///     #[export_tokens]
///     struct MyCoolStruct {
///         foo: u32,
///         bar: usize,
///         fizz: bool,
///     }
/// }
/// ```
///
/// You can now call `import_tokens!(MyCoolStruct)` anywhere that `my_module::*` has been
/// brought into scope (via a `use`), and you can call
/// `import_tokens!(my_module::MyCoolStruct)` anywhere that `my_module` is accessible,
/// including elsewhere in the same crate, or in crates that have the crate containing
/// `my_module` as a dependency.
///
/// In some other file in the same crate:
/// ```ignore
/// let tokens = import_tokens!(my_module::MyCoolStruct);
/// ```
///
/// In a proc macro crate that has the `my_module` crate as a dependency:
/// ```ignore
/// #[proc_macro]
/// pub fn my_macro(_tokens: TokenStream) -> TokenStream {
///     let struct_tokens = import_tokens!(some_crate::my_module::MyCoolStruct);
/// }
/// ```
///
/// With a `use` statement:
/// ```ignore
/// use some_crate::my_module::*;
///
/// #[proc_macro]
/// pub fn my_macro(_tokens: TokenStream) -> TokenStream {
///     let struct_tokens = import_tokens!(MyCoolStruct);
/// }
/// ```
///
/// Note that you must be able to import the module that contains the item where you called
/// `#[export_tokens]` for this to work properly.
///
/// ## Expansion
///
/// An invocation like this:
/// ```ignore
/// let tokens = import_tokens!(my_module::MyCoolStruct);
/// ```
/// would expand to this:
/// ```ignore
/// let tokens: TokenStream2 = my_module::MyCoolStruct::__EXPORT_TOKENS__MYCOOLSTRUCT
///     .parse::<::macro_magic::__private::TokenStream2>()
///     .unwrap();
/// ```
///
/// The `.unwrap()` will never fail because for [`#[export_tokens]`](`macro@export_tokens`) to
/// compile, the item it is attached to must be a valid [`syn::Item`], so syntax errors cannot
/// make it into the `__EXPORT_TOKENS__MYCOOLSTRUCT` const.
///
/// Because the expansion of [`import_tokens!()`](`macro@import_tokens`) calls the non-const
/// function `.parse()`, you cannot use [`import_tokens!`](`macro@import_tokens`) in a const
/// context.
///
/// Note that the type of `__EXPORT_TOKENS__MYCOOLSTRUCT` is `&'static str`. The naming of
/// these constants is consistent and is defined by the `get_const_name` function. You should
/// never need to call this directly so it is not exported anywhere.
#[proc_macro]
pub fn import_tokens(tokens: TokenStream) -> TokenStream {
    let path = parse_macro_input!(tokens as TypePath);
    let path = match get_const_path(&path) {
        Ok(path) => path,
        Err(e) => return e.to_compile_error().into(),
    };
    quote!(#path.parse::<::macro_magic::__private::TokenStream2>().unwrap()).into()
}

/// This macro allows you to import tokens across crate boundaries
/// without strict dependency requirements and to use advanced features such as
/// [`namespacing`](`read_namespace!').
///
/// Calling `import_tokens_indirect!` is slightly different from calling [`import_tokens!`] in
/// that indirect imports will work even when the item whose tokens you are importing is
/// contained in a crate that is not a dependency of the current crate, so long as the
/// following requirements are met:
///
/// 1. The "indirect-read" feature must be enabled for `macro_magic`, otherwise the
///    `import_tokens_indirect!` macro will not be available. This is automatically enabled by
///    the "indirect" feature.
/// 2. The source crate and the target crate must be in the same
///    [cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html). This is a
///    non-negotiable hard requirement when using indirect imports, however direct imports will
///    work fine across workspace boundaries (they just have other stricter requirements that
///    can be cumbersome).
/// 3. The source crate and the target crate must both use the same version of `macro_magic`
///    (this is not a hard requirement, but undefined behavior could occur with mixed
///    versions).
/// 4. Both the source crate and target crate must have their builds somehow triggered by the
///    compilation target of the current workspace such that they are both compiled. Unlike
///    with direct imports, where you explictily `use` the source crate as a dependency of the
///    target crate, there needs to be some reason to compile the source crate, or its exported
///    tokens will be unavailable.
/// 5. The export path declared by the source crate must exactly match the path you try to
///    import in the target crate. If you don't manually specify an export path, then your
///    import path should be the name of the item that
///    [`#[export_tokens]`](`macro@export_tokens`) was attached to (i.e. the `Ident`), however
///    this approach is not recommended since you can run into collisions if you are not
///    explicit about naming. For highly uniquely named items, however, this is fine. In other
///    words, if you don't specify a namespace, and you have an item named `foo` in two
///    different files, when you export these two items, they will collide.
/// 6. The target crate _must_ be a proc macro crate. If this requirement is violated, then the
///    build-order guarantees exploited by the indirect approach no longer hold true and you
///    may experience undefined behavior in the form of compile errors.
///
/// The vast majority of common use cases for `macro_magic` meet these criteria, but if you run
/// into any issues where exported tokens can't be found, make sure your source crate is
/// included as part of the compilation target and that it is in the current workspace.
/// Likewise watch out for collisions as these are easy to encounter if you don't
/// [`namespace`](`read_namespace!') your items.
///
/// Keep in mind that you can use the optional attribute, `#[export_tokens(my::path::Here)]` to
/// specify a disambiguation path for the tokens you are exporting. Otherwise the name of the item
/// the macro is attached to will be used, potentially causing collisions if you export items by
/// the same name from different contexts.
///
/// This situation will eventually be resolved when the machinery behind
/// [caller_modpath](https://crates.io/crates/caller_modpath) is stabilized, which will allow
/// `macro_magic` to automatically detect the path of the
/// [`#[export_tokens]`](`macro@export_tokens`) caller.
///
/// A peculiar aspect of how [`#[export_tokens(some_path)]`](`macro@export_tokens`) works is
/// the path you enter doesn't need to be a real path. You could do
/// `#[export_tokens(completely::made_up::path::MyItem)]` in one context and then
/// `import_tokens!(completely::made_up::path::MyItem)` in another context, and it will still
/// work as long as these two paths are the same. They need not actually exist, they are just
/// used for disambiguation so we can tell the difference between these tokens and other
/// potential exports of an item called `MyItem`. The last segment _does_ need to match the
/// name of the item you are exporting, however.
#[proc_macro]
pub fn import_tokens_indirect(tokens: TokenStream) -> TokenStream {
    match import_tokens_indirect_internal(tokens) {
        Ok(res) => res.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// This macro allows you to group a number of [`#[export_tokens]`](`macro@export_tokens`)
/// calls and collect them into a `Result<Vec<(String, TokenStream2)>>`.
///
/// The first component of the tuple corresponds with the name of the item and the second
/// component contains the tokens for that item. The `Result` is a [`std::io::Result`] and any
/// `Err` variants that come back would indicate an internal error (i.e. something tampered
/// with the `target` directory at an unexpected time) or (more likely) that the specified
/// namespace does not exist.
///
/// The [`macro@export_tokens`] attribute automatically defines namespaces when you call it
/// with an argument. Namespaces function like directories, so if you define item A with the
/// path `foo::bar::fizz` and item B with path `foo::bar::buzz`, and you will get back both
/// items if you read the namespace `foo::bar` i.e.:
///
/// ```ignore
/// let namespace_items = read_namespace!(foo::bar).unwrap();
/// let (name, tokens) = namespace_items.first().unwrap();
/// // name = "buzz"
/// // tokens = tokens for `foo::bar::buzz` item
/// ```
///
/// Note that `read_namespace!` always returns results sorted by name, so you can rely on the
/// order to be consistent.
#[proc_macro]
pub fn read_namespace(tokens: TokenStream) -> TokenStream {
    #[allow(unused)]
    let type_path = parse_macro_input!(tokens as TypePath);
    #[cfg(not(feature = "indirect-read"))]
    return syn::Error::new(
        quote::__private::Span::call_site().into(),
        "The `read_namespace!` macro can only be used when the \"indirect\" feature is enabled",
    )
    .to_compile_error()
    .into();
    #[cfg(feature = "indirect-read")]
    {
        let ref_path = get_ref_path(&type_path).to_str().unwrap().to_string();
        quote! {
            {
                use ::macro_magic::__private::TokenStream2;
                let closure = || -> std::io::Result<Vec<(String, TokenStream2)>> {
                    let namespace_path = #ref_path;
                    let mut results: Vec<(String, TokenStream2)> = Vec::new();
                    for entry in std::fs::read_dir(&namespace_path)? {
                        let entry = entry?;
                        if entry.path().is_dir() {
                            continue;
                        }
                        let source = std::fs::read_to_string(entry.path())?;
                        let tokens2 = source.parse::<TokenStream2>().unwrap();
                        let name = entry
                        .path()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_owned()
                        .to_string()
                        .replace("-", "::")
                        .replace("_LT_", "<")
                        .replace("_GT_", ">");
                        results.push((name, tokens2));
                    }
                    results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                    Ok(results)
                };
                closure()
            }
        }
        .into()
    }
}

/// This convenient macro can be used to publicly re-export an item that has been exported via
/// [`#[export_tokens]`](`macro@export_tokens`) when doing direct imports.
///
/// See the documentation for [`#[export_tokens]`](`macro@export_tokens`) and
/// [`import_tokens!`] for more information.
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
/// pub use mm_example_crate::cool_module::__EXPORT_TOKENS__MYTRAIT;
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
