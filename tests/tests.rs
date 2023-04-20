use macro_magic::*;

use macro_magic_macros::export_tokens_no_emit;
use test_macros::{custom_export_tokens, include_impl, include_impl_inner};

#[cfg(feature = "proc_support")]
use test_macros::some_macro;

#[use_attr]
use test_macros::combine_structs;
#[use_attr]
use test_macros::emit_foreign_path;
#[use_proc]
use test_macros::example_tokens_proc;
use test_macros::fake_pallet_section;
#[use_proc]
use test_macros::item_level_proc;
#[use_proc]
use test_macros::require;
#[use_attr]
use test_macros::test_tokens_attr1;
#[use_attr]
use test_macros::test_tokens_attr2;
#[use_attr]
use test_macros::verbatim_emit_both;

/// Some doc comment
#[use_attr]
pub use test_macros::test_tokens_attr_direct_import;

pub mod example_export {
    pub mod subpath {
        pub use ::macro_magic::*;
    }
}

mod external_file;

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
fn test_forward_tokens() {
    #[macro_export]
    macro_rules! receiver {
        ($tokens:item) => {
            stringify!($tokens)
        };
    }

    let result = forward_tokens!(LionStruct, receiver);
    assert_eq!(result, "struct LionStruct {}");
}

#[cfg(feature = "proc_support")]
#[test]
fn import_tokens_same_mod_no_ident() {
    some_macro!(SomeStruct);
    import_tokens!(let tokens = SomeStruct);
    assert!(tokens.to_string().contains("field1"));
}

#[cfg(feature = "proc_support")]
#[test]
fn import_tokens_same_mod_ident() {
    import_tokens!(let tokens = charlie);
    assert!(tokens.to_string().contains("field2 : usize"));
}

#[cfg(feature = "proc_support")]
#[test]
fn import_tokens_different_mod_no_ident() {
    import_tokens!(let tokens = PlusPlus);
    assert_eq!(
        tokens.to_string(),
        "fn plus_plus < T : Into < i64 > > (n : T) -> i64 { n . into () + 1 }"
    );
}

#[cfg(feature = "proc_support")]
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
fn println_inside_fn_external_file() {
    let tokens = example_tokens_proc!(external_fn_with_println);
    assert_eq!(
        tokens.to_string(),
        "fn external_fn_with_println() { println! (\"testing\") ; }"
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

#[custom_export_tokens]
struct Wombat {
    field1: u32,
    field2: u64,
}

#[test]
fn test_export_tokens_alias() {
    let tokens = example_tokens_proc!(Wombat);
    assert_eq!(
        tokens.to_string(),
        "struct Wombat { field1 : u32, field2 : u64, }"
    );
}

#[emit_foreign_path(external_crate::an_external_function)]
struct YetAnotherStruct {}

#[test]
fn test_foreign_path_emission() {
    assert_eq!(emitted_path, "external_crate :: an_external_function");
    assert_eq!(
        foreign_item_str,
        "fn an_external_function(my_num : u32) -> u32 { my_num + 33 }"
    );
}

#[export_tokens_no_emit]
fn non_compiling_fn() {
    compile_error!("this should not compile ");
}

// should not collide with above function since above function does not emit tokens locally and
// so it does not exist locally
fn non_compiling_fn() -> usize {
    3
}

#[test]
fn test_export_tokens_no_emit_exportation() {
    import_tokens!(let tokens = non_compiling_fn);
    assert_eq!(
        tokens.to_string(),
        "fn non_compiling_fn () { compile_error ! (\"this should not compile \") ; }"
    );
    assert_eq!(non_compiling_fn(), 3);
}

#[fake_pallet_section]
mod pallet {

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T, I = ()>(_);
}

// #[verbatim_emit_both(external_crate::ItemWithError)]
// struct SomethingLocal {
//     field: u32,
// }
