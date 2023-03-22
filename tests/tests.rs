use macro_magic::*;
use macro_magic_macros::export_tokens_alt;
use test_macros::*;

#[export_tokens]
struct SomeStruct {
    field1: u32,
    field2: bool,
}

#[export_tokens(charlie)]
struct Struct2 {
    field1: i64,
    field2: usize,
}

mod some_module {
    use macro_magic::*;

    #[export_tokens]
    fn plus_plus<T: Into<i64>>(n: T) -> i64 {
        n.into() + 1
    }

    #[export_tokens(MinusMinus)]
    fn minus_minus<T: Into<i32>>(n: T) -> i32 {
        n.into() - 1
    }
}

#[include_impl(SomeStruct)]
mod some_mod {}

#[export_tokens]
struct AnotherStruct {
    field1: u32,
}

#[test_tokens_attr1(AnotherStruct)]
pub mod hunter {
    pub fn stuff() {
        println!("things");
    }
}

#[test_tokens_attr2(external_crate::AnExternalTraitImpl)]
struct LocalItemStruct {}

#[test_tokens_attr_direct_import(external_crate::an_external_function)]
fn cute_little_fn() {
    println!("hey!");
}

#[export_tokens]
struct LionStruct {}

#[export_tokens_alt]
struct TigerStruct {}

#[test]
fn test_import_tokens_proc_statement_position() {
    example_tokens_proc!(LionStruct);
}

#[test]
fn test_import_tokens_proc_expr_position() {
    let something = example_tokens_proc!(TigerStruct);
    assert_eq!(something.to_string(), "struct TigerStruct {}");
}

#[test]
fn attr_direct_import() {
    assert_eq!(an_external_function(4), 37);
}

#[test]
fn import_tokens_same_mod_no_ident() {
    some_macro!(SomeStruct);
    import_tokens!(let tokens = SomeStruct);
    assert!(tokens.to_string().contains("field1"));
}

#[test]
fn import_tokens_same_mod_ident() {
    import_tokens!(let tokens = charlie);
    assert!(tokens.to_string().contains("field2 : usize"));
}

#[test]
fn import_tokens_different_mod_no_ident() {
    import_tokens!(let tokens = PlusPlus);
    assert_eq!(
        tokens.to_string(),
        "fn plus_plus < T : Into < i64 > > (n : T) -> i64 { n . into () + 1 }"
    );
}

#[test]
fn import_tokens_different_mod_ident() {
    import_tokens!(let tokens = MinusMinus);
    assert_eq!(
        tokens.to_string(),
        "fn minus_minus < T : Into < i32 > > (n : T) -> i32 { n . into () - 1 }"
    );
}
