use macro_magic::export_tokens;

#[export_tokens]
fn add2(left: usize, right: usize) -> usize {
    left + right
}

#[export_tokens]
pub mod cool_types {
    pub type Foo = u8;
    pub type Bar = u16;
    pub type Fizz = u32;
    pub type Buzz = u64;
}

#[export_tokens]
fn subtraction(left: i64, right: i64) -> i64 {
    left - right
}

pub mod cool_module {
    use macro_magic::*;

    #[export_tokens]
    trait MyTrait {
        fn some_behavior() -> String;
        type SomeType;
    }
}
