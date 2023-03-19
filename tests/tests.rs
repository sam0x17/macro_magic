use macro_magic::*;
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

// #[include_impl(SomeStruct)]
// mod some_mod {}

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
