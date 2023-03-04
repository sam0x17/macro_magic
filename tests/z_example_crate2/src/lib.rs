use macro_magic::export_tokens;

#[export_tokens(example_crate2::mult)]
fn mult(a: i32, b: i32) -> i32 {
    a * b
}

#[export_tokens(foo_bar::red::green::max_i32)]
fn max_i32(a: i32, b: i32) -> i32 {
    if a < b {
        b
    } else {
        b
    }
}

#[export_tokens(foo_bar::red::green::max_i64)]
fn max_i64(a: i64, b: i64) -> i64 {
    if a < b {
        b
    } else {
        b
    }
}

#[export_tokens(foo_bar::red::green::maax_i128)]
fn max_i128(a: i128, b: i128) -> i128 {
    if a < b {
        b
    } else {
        b
    }
}

#[export_tokens(foo_bar::red::green::max_f64)]
fn max_f64(a: f64, b: f64) -> f64 {
    if a < b {
        b
    } else {
        b
    }
}

#[export_tokens(foo_bar::red::max_f32)]
fn max_f32(a: f32, b: f32) -> f32 {
    if a < b {
        b
    } else {
        b
    }
}

#[export_tokens(foo_bar::red::max_u32)]
fn max_u32(a: u32, b: u32) -> u32 {
    if a < b {
        b
    } else {
        b
    }
}

#[export_tokens(foo_bar::red::max_u64)]
fn max_u64(a: u64, b: u64) -> u64 {
    if a < b {
        b
    } else {
        b
    }
}

#[export_tokens]
fn div(a: i64, b: i64) -> i64 {
    a / b
}

#[export_tokens(BadBad<T>)]
struct Bad {}
