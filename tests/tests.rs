use ::example_proc_macro::*;
use ::macro_magic::{__private::TokenStream2, *};
use syn::{ItemConst, ItemMod, __private::ToTokens};

mod other_file;

#[test]
fn export_tokens() {
    #[export_tokens]
    fn add_stuff(a: usize, b: usize) -> usize {
        a + b
    }
    assert_eq!(
        __EXPORT_TOKENS__ADD_STUFF,
        "fn add_stuff(a : usize, b : usize) -> usize { a + b }"
    );
}

#[test]
fn import_tokens() {
    #[export_tokens]
    fn add_stuff(c: usize, d: usize) -> usize {
        c + d
    }
    assert_eq!(
        import_tokens!(add_stuff).to_string(),
        "fn add_stuff (c : usize , d : usize) -> usize { c + d }"
    );
}

#[test]
fn external_file_parsing() {
    let tokens: TokenStream2 = import_tokens!(other_file::some_module::foo);
    let item_mod: ItemMod = syn::parse2(tokens).unwrap();
    assert_eq!(item_mod.ident.to_string(), "foo");
    let item_const: ItemConst = syn::parse2(
        item_mod
            .content
            .unwrap()
            .1
            .first()
            .unwrap()
            .to_token_stream(),
    )
    .unwrap();
    assert_eq!(item_const.ident.to_string(), "SOMETHING");
}

#[test]
fn external_crate_proc_macro_a() {
    example_macro!();
    // add function imported via import_tokens! call in proc macro
    assert_eq!(add2(2, 3), 5);
}

example_macro2!();

#[test]
fn external_crate_proc_macro_b() {
    let _a: cool_types::Bar = 3;
}

#[test]
fn verbatim_import() {
    println!("{}", import!(example_crate::subtraction));
    //assert_eq!(subtraction(10, 3), 7);
}

#[test]
fn magic_crate_dir_macro() {
    assert!(magic_crate_dir!().ends_with("/__magic_crate"));
}
