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

trait MyCoolTrait<T, E> {
    fn foo(input: T) -> T;
    fn bar(input: E) -> E;
}

struct FooBar<I> {
    _data: I,
}

pub struct DooDad<I> {
    _data: I,
}

#[export_tokens(ImplMyCoolTraitForFooBar)]
impl<T, E, I> MyCoolTrait<T, E> for FooBar<I> {
    fn foo(input: T) -> T {
        input
    }

    fn bar(input: E) -> E {
        input
    }
}

#[cfg(feature = "indirect-write")]
#[export_tokens(some::path::ImplMyCoolTraitForDooDad)]
impl<T, E, I> MyCoolTrait<T, E> for DooDad<I> {
    fn foo(input: T) -> T {
        input
    }

    fn bar(input: E) -> E {
        input
    }
}
