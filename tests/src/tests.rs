#![cfg(test)]

use ::macro_magic::*;

#[test]
fn export_tokens() {
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
fn import_tokens() {
    #[export_tokens]
    fn add_stuff(c: usize, d: usize) -> usize {
        c + d
    }
    assert_eq!(
        import_tokens!(add_stuff).to_string(),
        "fn add_stuff (c : usize , d : usize) -> usize { c + d }"
    );
}
