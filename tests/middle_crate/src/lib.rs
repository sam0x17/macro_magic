pub mod export_mod {
    pub mod sub_mod {
        pub use macro_magic;
    }
}

#[macro_magic::export_tokens]
struct ForeignItem {}

#[macro_magic::use_attr]
pub use test_macros::distant_re_export_attr;

pub use macro_magic::use_attr;
