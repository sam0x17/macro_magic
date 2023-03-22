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

#[export_tokens]
struct TigerStruct {}

// test proc item position
item_level_proc!(external_crate::AnExternalTraitImpl);

#[test]
fn test_import_tokens_proc_item_position() {
    let _foo = SomeInjectedStruct {};
}

#[test]
fn test_import_tokens_proc_statement_position() {
    example_tokens_proc!(LionStruct);
    example_tokens_proc!(external_crate::AnExternalTraitImpl);
}

#[test]
fn test_import_tokens_proc_expr_position() {
    let something = example_tokens_proc!(TigerStruct);
    assert_eq!(something.to_string(), "struct TigerStruct {}");
    let _something_else = example_tokens_proc!(external_crate::AnExternalTraitImpl);
}

#[test]
fn test_export_tokens_inside_function() {
    let something = example_tokens_proc!(external_crate::some_sub_function);
    assert_eq!(
        something.to_string(),
        "fn some_sub_function() -> u32 { 33 }"
    );
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

#[export_tokens]
fn a_random_fn() {
    println!("hey");
}

#[test]
fn println_inside_fn_current_file() {
    let tokens = example_tokens_proc!(a_random_fn);
    assert_eq!(
        tokens.to_string(),
        "fn a_random_fn() { println! (\"hey\") ; }"
    );
}

#[test]
fn macro_calls_inside_fn_external_crate() {
    let tokens = example_tokens_proc!(external_crate::external_fn_with_local_macro_calls);
    assert_eq!(
        tokens.to_string(),
        "fn external_fn_with_local_macro_calls() -> u32 { another_macro! () ; 1337 }"
    );
}

#[export_tokens]
struct ExternalStruct {
    foo: u32,
    bar: u64,
    fizz: i64,
}

#[combine_structs(ExternalStruct)]
struct LocalStruct {
    biz: bool,
    baz: i32,
}

#[test]
fn test_combine_structs_example() {
    let _something = LocalStruct {
        foo: 42,
        bar: 19,
        fizz: -22,
        biz: true,
        baz: 87,
    };
}

#[test]
fn test_require_example() {
    require!(external_crate::an_external_module);
    assert_eq!(my_cool_function(), 567);
}
