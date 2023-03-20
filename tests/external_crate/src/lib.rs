mod some_submodule {
    struct FooBarStruct {}

    trait FooBarTrait {
        fn foo(n: u32) -> u32;
        fn bar(n: i32) -> i32;
        fn fizz(v: bool) -> bool;
        fn buzz(st: String) -> String;
    }

    #[macro_magic::export_tokens(AnExternalTraitImpl)]
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

        fn buzz(st: String) -> String {
            format!("{}buzzz", st)
        }
    }
}

#[macro_magic::export_tokens]
fn an_external_function(my_string: String) -> String {
    format!("{}_bizzz!", my_string)
}
