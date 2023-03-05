use ::macro_magic::{__private::TokenStream2, *};
use ::mm_example_proc_macro::*;
use syn::{ItemConst, ItemMod, __private::ToTokens};

mod other_file;
mod re_exports;

#[test]
fn export_tokens_basic() {
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
fn import_tokens_basic() {
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
fn import_tokens_generics() {
    #[export_tokens]
    struct MyGenericStruct<T: Into<String>> {
        something: T,
    }
    assert_eq!(
        import_tokens!(MyGenericStruct).to_string(),
        "struct MyGenericStruct < T : Into < String >> { something : T , }"
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

#[cfg(feature = "indirect")]
#[test]
fn external_crate_direct_import_fn() {
    example_macro!();
    // add function imported via import_tokens! call in proc macro
    assert_eq!(add2(2, 3), 5);
}

example_macro2!();

#[test]
fn external_crate_direct_import_trait() {
    let _a: cool_types::Bar = 3;
}

#[cfg(feature = "indirect")]
#[test]
fn external_crate_indirect_import_fn() {
    example_macro3!();
    assert_eq!(mult(4, 5), 20);
}

#[test]
fn re_exports() {
    let tokens = import_tokens!(re_exports::MyTrait);
    assert!(!tokens.is_empty());
    assert!(tokens.to_string().contains("trait MyTrait"));
}

#[cfg(feature = "indirect")]
#[test]
fn external_crate_indirect_generics_in_name() {
    example_macro4!();
}

#[cfg(feature = "indirect")]
#[test]
fn read_namespace_red() {
    read_namespace_test_red!();
}

#[cfg(feature = "indirect")]
#[test]
fn read_namespace_green() {
    read_namespace_test_green!();
}

#[cfg(feature = "indirect")]
#[test]
fn read_namespace_foo_bar() {
    read_namespace_test_foo_bar!();
}
