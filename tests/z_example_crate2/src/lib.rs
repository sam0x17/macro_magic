use macro_magic::export_tokens;

#[export_tokens(example_crate2::mult)]
fn mult(a: i32, b: i32) -> i32 {
    a * b
}

#[export_tokens]
fn div(a: i64, b: i64) -> i64 {
    a / b
}

#[export_tokens(BadBad<T>)]
struct Bad {}
