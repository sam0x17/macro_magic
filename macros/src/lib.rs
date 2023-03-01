extern crate proc_macro;
use proc_macro::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::hash::Hash;
use std::{fs, io::Write};
use syn::{parse_macro_input, spanned::Spanned, Error, Ident, Item, Path, TypePath};

const REFS_DIR: &'static str = env!("REFS_DIR");
#[allow(unused)]
const WATCHER_CRATE_PATH: &'static str = "../token_watcher";

#[allow(unused)]
fn write_file<T: Into<String>>(path: &std::path::Path, source: T) -> std::io::Result<()> {
    println!("opening {} for writing...", path.to_str().unwrap());
    let mut f = OpenOptions::new().write(true).create(true).open(path)?;
    println!("writing data...");
    f.write_all(source.into().as_bytes())?;
    println!("wrote data. Flushing..");
    f.flush()?;
    Ok(())
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct CrateReference {
    name: String,                      // "my-crate"
    access_name: String,               // "my_crate"
    path: String,                      // "/home/sam/workspace/my-crate"
    referenced_items: HashSet<String>, // HashSet::from([String::from("my_crate::some::item"), String::from("my_crate::some_other::item")])
}

impl Hash for CrateReference {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.access_name.hash(state);
        self.path.hash(state);
        // note: skipping referenced_items
    }
}

#[allow(unused)]
fn generate_crate(refs: &HashSet<CrateReference>) -> std::io::Result<()> {
    use std::path::Path;

    let crate_dir = Path::new(&WATCHER_CRATE_PATH);
    if crate_dir.exists() {
        fs::remove_dir_all(crate_dir)?;
        fs::create_dir(crate_dir)?;
    } else {
        fs::create_dir_all(crate_dir)?;
    }
    let src_dir = crate_dir.join(Path::new("src"));
    fs::create_dir(src_dir.clone())?;
    let cargo_toml_path = crate_dir.join(Path::new("Cargo.toml"));
    write_file(
        &cargo_toml_path,
        format!(
            "[package]\n\
             name = \"token_watcher\"\n\
             version = \"0.1.0\"\n\
             edition = \"2021\"\n\
             publish = false\n\
             \n\
             [dependencies]\n\
             macro_magic = {{ version = \"*\", path = \"../\" }}\n\
             {}\n\
            ",
            refs.iter()
                .map(|r| format!("{} = {{ version = \"*\", path = \"{}\" }}", r.name, r.path))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    )?;
    let lib_rs_path = src_dir.join(Path::new("lib.rs"));
    write_file(
        &lib_rs_path,
        format!(
            "// auto-generated -- do not edit manually\nuse macro_magic::pub_use_src_const;\n{}\n",
            refs.iter()
                .flat_map(|r| r.referenced_items.iter().cloned())
                .map(|item| format!("pub_use_src_const!({});", item))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    )?;
    let rustfmt_toml_path = crate_dir.join(Path::new("rustfmt.toml"));
    write_file(&rustfmt_toml_path, "ignore = [\"src/lib.rs\"]");
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

#[proc_macro_attribute]
pub fn export_tokens(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return Error::new(
            Span::call_site().into(),
            "#[export_tokens] does not take any arguments",
        )
        .to_compile_error()
        .into();
    }
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
    //write_file(&refs_dir.join(Path::new(&format!(""))), &source_code).unwrap(); // do error handling
    quote! {
        #[allow(dead_code)]
        #item
        #[doc(hidden)]
        #[allow(dead_code)]
        pub const #const_ident: &'static str = #source_code;
    }
    .into()
}

// Imports a `TokenStream2` representing the item at the specified path
#[proc_macro]
pub fn import_tokens(tokens: TokenStream) -> TokenStream {
    let path = match get_const_path(&parse_macro_input!(tokens as TypePath)) {
        Ok(path) => path,
        Err(e) => return e.to_compile_error().into(),
    };
    quote!(#path.parse::<::macro_magic::__private::TokenStream2>().unwrap()).into()
}

#[doc(hidden)]
#[proc_macro]
pub fn pub_use_src_const(tokens: TokenStream) -> TokenStream {
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

/// Verbatim imports the item located at the specified path, similar to `require` in Ruby. This
/// is different from a standard `use` statement because this expands to the code for whatever
/// foreign item is referenced, whereas Rust's implementation of use functions differently.
#[proc_macro]
pub fn import(tokens: TokenStream) -> TokenStream {
    let path = parse_macro_input!(tokens as TypePath);
    quote!(import_tokens!(#path)).into()
}

#[test]
fn test_generate_crate() {
    //let reference = CrateReference { name: "example_crate", access_name: , path: (), referenced_items: () };
    generate_crate(&HashSet::from([CrateReference {
        name: String::from("example_crate2"),
        access_name: String::from("example_crate2"),
        path: String::from("../tests/example_crate2"),
        referenced_items: HashSet::from([
            String::from("example_crate2::mult"),
            String::from("example_crate2::div"),
        ]),
    }]))
    .unwrap();
}
