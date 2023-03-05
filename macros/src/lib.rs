extern crate proc_macro;
use proc_macro::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, Error, Ident, Item, Path, TypePath};

#[cfg(feature = "indirect")]
use std::{
    fs::{create_dir_all, OpenOptions},
    io::Write,
    iter,
    path::PathBuf,
};

#[cfg(feature = "indirect")]
use atomicwrites::{AllowOverwrite, AtomicFile};

#[cfg(feature = "indirect")]
const REFS_DIR: &'static str = env!("REFS_DIR");

#[cfg(feature = "indirect")]
fn write_file<T: Into<String>>(path: &std::path::Path, source: T) -> std::io::Result<()> {
    let parent = path.parent().unwrap();
    if !parent.exists() {
        #[cfg(feature = "verbose")]
        println!("directory {} doesn't exist, creating...", parent.display());
        create_dir_all(parent)?;
    }
    #[cfg(feature = "verbose")]
    println!("writing {}...", path.display());
    let data: String = source.into();
    let af = AtomicFile::new(path, AllowOverwrite);
    af.write_with_options(
        |f| f.write_all(data.as_bytes()),
        OpenOptions::new().write(true).create(true).clone(),
    )?;
    #[cfg(feature = "verbose")]
    println!("wrote {}.", path.display());
    Ok(())
}

#[cfg(feature = "indirect")]
fn get_ref_path(type_path: &TypePath) -> PathBuf {
    PathBuf::from_iter(
        iter::once(String::from(REFS_DIR)).chain(
            type_path
                .path
                .segments
                .iter()
                .map(|seg| sanitize_name(seg.to_token_stream().to_string())),
        ),
    )
}

#[cfg(feature = "indirect")]
fn sanitize_name(name: String) -> String {
    name.replace("::", "-")
        .replace("<", "_LT_")
        .replace(">", "_GT_")
        .replace(" ", "")
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

/// This attribute can be attached to any [`syn::Item`]-compatible source code item, with the
/// exception of [`ForeignMod'](`syn::Item::ForeignMod`), [`Impl`](`syn::Item::Impl`),
/// [`Macro`](`syn::Item::Macro`), [`Use`](`syn::Item::Use`), and
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

    if !attr.is_empty() {
        let export_path = parse_macro_input!(attr as TypePath);
        #[cfg(feature = "indirect")]
        {
            use std::path::Path;
            let refs_dir = Path::new(REFS_DIR);
            assert!(refs_dir.exists());
            let fpath = get_ref_path(&export_path);
            let Ok(_) = write_file(&fpath, &source_code) else {
                return Error::new(
                    export_path.path.segments.last().span(),
                    "Failed to write to the specified namespace, is it already occupied?",
                )
                .to_compile_error()
                .into()
            };
        }
        #[cfg(not(feature = "indirect"))]
        {
            return Error::new(
                export_path.span(),
                "Arguments for #[export_tokens] are only supported when the \"indirect\" feature is enabled"
            )
            .to_compile_error()
            .into();
        }
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

/// This macro is the primary way to bring exported tokens into scope in your proc macros
/// (though it can also be used in non-proc-macro contexts, and is based on `TokenStream2` for
/// this purpose).
///
/// This approach is called a "direct import" and requires the source and target to be in the
/// same crate, or requires that the source crate is a dependency of the target crate. For a
/// less restrictive approach, see [`import_tokens_indirect!`].
///
/// Suppose you have exported tokens using the [`macro@export_tokens`] attribute macro as follows:
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
/// The `.unwrap()` will never fail because for `#[export_tokens]` to compile, the item it is
/// attached to must be a valid `syn::Item`, so syntax errors cannot make it into the
/// `__EXPORT_TOKENS__MYCOOLSTRUCT` const.
///
/// Because the expansion of `import_tokens!()` calls the non-const function `.parse()`, you
/// cannot use `import_tokens!()` in a const context.
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
/// 1. The "indirect" feature must be enabled for `macro_magic`, otherwise the
///    `import_tokens_indirect!` macro will not be available.
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
///    import path should be the name of the item that `#[export_tokens]` was attached to (i.e.
///    the `Ident`), however this approach is not recommended since you can run into collisions
///    if you are not explicit about naming. For highly uniquely named items, however, this is
///    fine. In other words, if you don't specify a namespace, and you have an item named `foo`
///    in two different files, when you export these two items, they will collide.
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
/// `macro_magic` to automatically detect the path of the `#[export_tokens]` caller.
///
/// A peculiar aspect of how `#[export_tokens(some_path)]` works is the path you enter doesn't need
/// to be a real path. You could do `#[export_tokens(completely::made_up::path::MyItem)]` in one
/// context and then `import_tokens!(completely::made_up::path::MyItem)` in another context, and it
/// will still work as long as these two paths are the same. They need not actually exist, they are
/// just used for disambiguation so we can tell the difference between these tokens and other
/// potential exports of an item called `MyItem`. The last segment _does_ need to match the name of
/// the item you are exporting, however.
#[proc_macro]
pub fn import_tokens_indirect(tokens: TokenStream) -> TokenStream {
    #[allow(unused)]
    let path = parse_macro_input!(tokens as TypePath);
    #[cfg(not(feature = "indirect"))]
    return Error::new(
        Span::call_site().into(),
        "The `import_tokens_indirect!` macro can only be used when the \"indirect\" feature is enabled",
    )
    .to_compile_error()
    .into();
    #[cfg(feature = "indirect")]
    {
        let fpath = get_ref_path(&path).to_str().unwrap().to_string();
        let src_qt = quote! {
            std::fs::read_to_string(#fpath)
            .expect(
                "Indirectly importing the specified item failed. Make \
                 sure the path is correct and the crate the item appears \
                 in is being compiled as part of this workspace.",
            )
            .parse::<::macro_magic::__private::TokenStream2>()
            .unwrap()
        };
        if cfg!(feature = "verbose") {
            return quote! {
                {
                    println!("reading {}...", #fpath);
                    let source = #src_qt;
                    println!("read {}.", #fpath);
                    source
                }
            }
            .into();
        } else {
            return quote!(#src_qt).into();
        }
    }
}

/// The `read_namespace` allows you to group a number of `#[export_tokens]` calls and collect
/// them into a [`Result<Vec<(String, TokenStream2)>>`].
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
    #[cfg(not(feature = "indirect"))]
    return Error::new(
        Span::call_site().into(),
        "The `read_namespace!` macro can only be used when the \"indirect\" feature is enabled",
    )
    .to_compile_error()
    .into();
    #[cfg(feature = "indirect")]
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
