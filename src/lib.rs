pub use macros::*;

#[doc(hidden)]
pub mod __private {
    pub use syn::__private::TokenStream2;
}

pub mod macro_magic {
    pub use super::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_tokens() {
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
    fn test_import_tokens() {
        #[export_tokens]
        fn add_stuff(a: usize, b: usize) -> usize {
            a + b
        }
        assert_eq!(
            import_tokens!(add_stuff).to_string(),
            "fn add_stuff (a : usize , b : usize) -> usize { a + b }"
        );
    }
}
