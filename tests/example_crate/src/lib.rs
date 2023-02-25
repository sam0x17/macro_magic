use macro_magic::export_tokens;

#[export_tokens]
fn add2(left: usize, right: usize) -> usize {
    left + right
}
