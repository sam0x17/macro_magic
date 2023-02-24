pub mod some_module {
    use ::macro_magic::*;

    #[export_tokens]
    pub mod foo {
        const SOMETHING: &'static str = "cool";

        fn subtract<T: Into<i128> + From<i128>>(a: T, b: T) -> T {
            (a.into() - b.into()).into()
        }

        type MyType = u32;
    }
}
