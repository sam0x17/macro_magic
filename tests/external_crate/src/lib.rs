#![no_std] // test that `#[export_tokens]` works with `no_std`

use macro_magic::*;

pub mod some_submodule {
    use macro_magic::*;

    struct FooBarStruct {}

    trait FooBarTrait {
        fn foo(n: u32) -> u32;
        fn bar(n: i32) -> i32;
        fn fizz(v: bool) -> bool;
    }

    #[export_tokens(AnExternalTraitImpl)]
    impl FooBarTrait for FooBarStruct {
        fn foo(n: u32) -> u32 {
            n + 1
        }

        fn bar(n: i32) -> i32 {
            n - 1
        }

        fn fizz(v: bool) -> bool {
            !v
        }
    }
}

#[export_tokens]
fn an_external_function(my_num: u32) -> u32 {
    my_num + 33
}

#[export_tokens]
mod an_external_module {
    fn my_cool_function() -> u32 {
        567
    }
}

macro_rules! another_macro {
    () => {
        let a = 2;
    };
}

#[export_tokens]
fn external_fn_with_local_macro_calls() -> u32 {
    another_macro!();
    1337
}
