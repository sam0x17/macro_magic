pub use macros::export_tokens;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_tokens() {
        #[export_tokens]
        fn _add(a: usize, b: usize) -> usize {
            a + b
        }
        assert_eq!(
            SOURCE_CODE,
            "fn _add(a : usize, b : usize) -> usize { a + b }"
        );
    }
}
