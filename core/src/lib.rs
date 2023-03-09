use std::fmt::Display;

use quote::{quote, ToTokens, __private::Span};
use syn::__private::TokenStream2;
use syn::parse2;
use syn::{spanned::Spanned, Error, Ident, Item, Path, Result, TypePath};

#[cfg(any(feature = "indirect-write", feature = "indirect-read"))]
use std::{iter, path::PathBuf};

#[cfg(feature = "indirect-write")]
use std::{
    fs::{create_dir_all, OpenOptions},
    io::Write,
};

#[cfg(feature = "indirect-write")]
use atomicwrites::{AllowOverwrite, AtomicFile};

#[cfg(any(feature = "indirect-write", feature = "indirect-read"))]
const REFS_DIR: &'static str = env!("REFS_DIR");

#[cfg(feature = "indirect-write")]
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

/// Helper method used to get the path to the data file corresponding with an indrect [`TypePath`]
#[cfg(any(feature = "indirect-write", feature = "indirect-read"))]
pub fn get_ref_path(type_path: &TypePath) -> PathBuf {
    PathBuf::from_iter(
        iter::once(String::from(REFS_DIR)).chain(
            type_path
                .path
                .segments
                .iter()
                .map(|seg| sanitize_name_file(seg.to_token_stream().to_string())),
        ),
    )
}

/// Helper method used to sanitize a path name for use as a file name
#[cfg(any(feature = "indirect-write", feature = "indirect-read"))]
fn sanitize_name_file(name: String) -> String {
    name.replace("::", "-")
        .replace("<", "_LT_")
        .replace(">", "_GT_")
        .replace(" ", "")
}

/// Helper method used to sanitize a path name for use as a constant name
fn sanitize_name_constant(name: String) -> String {
    name.replace("::", "__")
        .replace("<", "_LT_")
        .replace(">", "_GT_")
        .replace(" ", "")
}

/// Helper method used to generate the name of the const used by direct imports to store the
/// raw source code of an item before it is converted to a [`TokenStream2`] by the
/// `import_tokens!` macro.
pub fn get_const_name(name: String) -> String {
    format!(
        "__EXPORT_TOKENS__{}",
        sanitize_name_constant(name).to_uppercase()
    )
}

/// Helper method used to generate the full path of a direct import const.
pub fn get_const_path(path: &TypePath) -> core::result::Result<Path, Error> {
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

/// The full internal implementation behind `#[export_tokens]`. This can be used to make custom
/// `#[export_tokens]` macros. The `feature_name` attribute determines the name that will be
/// displayed in error messages. When using the `#[export_tokens]` macro directly,
/// `feature_name` is set to `"#[export_tokens]"`.
///
/// Returns a `Result<Item, TokenStream2>` where the first item is `tokens` parsed as an
/// [`Item`] and the second item is the const declaration that would be generated for a
/// _direct_ import as a [`TokenStream2`]. Calling this function will write to the appropriate
/// item storage if the "indirect-write" or "indirect" features are enabled.
pub fn export_tokens_internal<T: Into<TokenStream2>, E: Into<TokenStream2>, I: Display>(
    tokens: T,
    attr: E,
    feature_name: I,
) -> Result<(Item, TokenStream2)> {
    let attr = attr.into();
    let item: Item = parse2(tokens.into())?;
    let ident = match item.clone() {
        Item::Const(item_const) => Some(item_const.ident),
        Item::Enum(item_enum) => Some(item_enum.ident),
        Item::ExternCrate(item_extern_crate) => Some(item_extern_crate.ident),
        Item::Fn(item_fn) => Some(item_fn.sig.ident),
        Item::Macro(item_macro) => item_macro.ident, // note this one might not have an Ident as well
        Item::Macro2(item_macro2) => Some(item_macro2.ident),
        Item::Mod(item_mod) => Some(item_mod.ident),
        Item::Static(item_static) => Some(item_static.ident),
        Item::Struct(item_struct) => Some(item_struct.ident),
        Item::Trait(item_trait) => Some(item_trait.ident),
        Item::TraitAlias(item_trait_alias) => Some(item_trait_alias.ident),
        Item::Type(item_type) => Some(item_type.ident),
        Item::Union(item_union) => Some(item_union.ident),
        // Item::ForeignMod(item_foreign_mod) => None,
        // Item::Use(item_use) => None,
        // Item::Impl(item_impl) => None,
        _ => None,
    };
    let source_code = item.to_token_stream().to_string();
    let export_path: Option<TypePath> = match attr.is_empty() {
        true => None,
        false => {
            let export_path: TypePath = parse2(attr)?;
            #[cfg(feature = "indirect-write")]
            {
                use std::path::Path;
                let refs_dir = Path::new(REFS_DIR);
                assert!(refs_dir.exists());
                let fpath = get_ref_path(&export_path);
                let Ok(_) = write_file(&fpath, &source_code) else {
                    return Err(Error::new(
                        export_path.path.segments.last().span(),
                        "Failed to write to the specified namespace, is it already occupied?",
                    ))
                };
            }
            Some(export_path)
        }
    };
    let const_name = get_const_name(match export_path {
        Some(export_path) => {
            #[cfg(not(feature = "indirect-write"))]
            if export_path.path.segments.len() > 1 {
                return Err(Error::new(
                    export_path.span(),
                    "Complex paths (i.e. containing `::`) are not allowed when the \"indirect-write]\" feature is enabled",
                ));
            }
            export_path.to_token_stream().to_string()
        }
        None => match ident {
            Some(ident) => ident.to_string(),
            None => {
                return Err(Error::new(
                    item.span(),
                    format!(
                        "Automatic name detection is not supported for this item. You must provide {} \
                        with an argument specifying a disambiguation name",
                        feature_name
                    ),
                ));
            }
        },
    });
    let const_ident = Ident::new(const_name.as_str(), Span::call_site());
    Ok((
        item,
        quote!(pub const #const_ident: &'static str = #source_code;),
    ))
}

/// The full internal implementation behind the `import_tokens_indirect!` macro. Can be used to
/// make custom/re-branded macros that behave like `import_tokens_indirect!`.
pub fn import_tokens_indirect_internal<T: Into<TokenStream2>>(tokens: T) -> Result<TokenStream2> {
    #[allow(unused)]
    let path: TypePath = parse2(tokens.into())?;
    #[cfg(not(feature = "indirect-read"))]
    return Err(Error::new(
        Span::call_site().into(),
        "The `import_tokens_indirect!` macro can only be used when the \"indirect-read\" feature is enabled",
    ));
    #[cfg(feature = "indirect-read")]
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
            return Ok(quote! {
                {
                    println!("reading {}...", #fpath);
                    let source = #src_qt;
                    println!("read {}.", #fpath);
                    source
                }
            });
        } else {
            return Ok(quote!(#src_qt));
        }
    }
}

/// The full internal implementation behind the `read_namespace_internal!` macro. Can be used to
/// make custom/re-branded macros that behave like `read_namespace_internal!`.
///
/// Note that the returned [`TokenStream2`] consists of the tokens of runtime code that, when
/// run, results in a `Result<Vec<TokenStream2>>` of all the tokens in the specified namespace.
/// This function does not directly retrieve the tokens (if you need that, just call
/// `import_tokens_indirect` directly!).
pub fn read_namespace_internal<T: Into<TokenStream2>>(tokens: T) -> Result<TokenStream2> {
    #[allow(unused)]
    let type_path: TypePath = parse2(tokens.into())?;
    #[cfg(not(feature = "indirect-read"))]
    return Err(Error::new(
        Span::call_site().into(),
        "The `read_namespace!` macro can only be used when the \"indirect\" feature is enabled",
    ));
    #[cfg(feature = "indirect-read")]
    {
        let ref_path = get_ref_path(&type_path).to_str().unwrap().to_string();
        Ok(quote! {
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
        })
    }
}
