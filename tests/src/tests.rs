#![cfg(test)]

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
