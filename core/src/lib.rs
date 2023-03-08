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

#[cfg(any(feature = "indirect-write", feature = "indirect-read"))]
pub fn get_ref_path(type_path: &TypePath) -> PathBuf {
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

#[cfg(any(feature = "indirect-write", feature = "indirect-read"))]
fn sanitize_name(name: String) -> String {
    name.replace("::", "-")
        .replace("<", "_LT_")
        .replace(">", "_GT_")
        .replace(" ", "")
}

pub fn get_const_name(name: String) -> String {
    format!("__EXPORT_TOKENS__{}", name.replace(" ", "").to_uppercase())
}

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

pub fn export_tokens_internal<T: Into<TokenStream2>, E: Into<TokenStream2>, I: Display>(
    tokens: T,
    attr: E,
    feature_name: I,
) -> Result<(Item, TokenStream2)> {
    let item: Item = parse2(tokens.into())?;
    let ident = match item.clone() {
        Item::Const(item_const) => item_const.ident,
        Item::Enum(item_enum) => item_enum.ident,
        Item::ExternCrate(item_extern_crate) => item_extern_crate.ident,
        Item::Fn(item_fn) => item_fn.sig.ident,
        Item::ForeignMod(item_foreign_mod) => {
            return Err(Error::new(
                item_foreign_mod.span(),
                format!("{} cannot be applied to a foreign module", feature_name),
            ))
        }
        Item::Impl(item_impl) => {
            return Err(Error::new(
                item_impl.span(),
                format!("{} cannot be applied to an impl", feature_name),
            ))
        }
        Item::Macro(item_macro) => match item_macro.ident {
            Some(ident) => ident,
            None => {
                return Err(Error::new(
                    item_macro.span(),
                    format!("{} cannot be applied to unnamed decl macros", feature_name),
                ))
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
            return Err(Error::new(
                item_use.span(),
                format!("{} cannot be applied to a use declaration", feature_name),
            ))
        }
        _ => {
            return Err(Error::new(
                item.span(),
                format!("{} cannot be applied to this item", feature_name),
            ))
        }
    };
    let const_name = get_const_name(ident.to_string());
    let const_ident = Ident::new(const_name.as_str(), Span::call_site());
    let source_code = item.to_token_stream().to_string();

    let attr = attr.into();
    if !attr.is_empty() {
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
        #[cfg(not(feature = "indirect-write"))]
        {
            return Err(Error::new(
                export_path.span(),
                format!("Arguments for {} are only supported when the \"indirect-write\" feature is enabled", feature_name)
            ));
        }
    }
    Ok((
        item,
        quote!(pub const #const_ident: &'static str = #source_code;),
    ))
}
