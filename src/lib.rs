//! # Macro Magic 🪄
//!
//! ![Build Status](https://img.shields.io/github/actions/workflow/status/sam0x17/macro_magic/ci.yaml)
//! ![GitHub](https://img.shields.io/github/license/sam0x17/macro_magic)
//! ![Crates.io](https://img.shields.io/crates/d/macro_magic)
//! ![docs.rs](https://img.shields.io/docsrs/macro_magic?label=docs)
//!
//! This crate provides an [`#[export_tokens]`](`export_tokens`) attribute macro, and a number
//! of companion macros, most prominently [`#[import_tokens_proc]`](`import_tokens_proc`) and
//! [`#[import_tokens_attr]`](`import_tokens_attr`), which, when used in tandem with
//! [`#[export_tokens]`](`export_tokens`), allow you to create regular and attribute proc
//! macros in which you can import and make use of the tokens of external/foreign items marked
//! with [`#[export_tokens]`](`export_tokens`) in other modules, files, and even in other
//! crates merely by referring to them by name/path.
//!
//! Among other things, the patterns introduced by `macro_magic` can be used to implement safe
//! and efficient exportation and importation of item tokens within the same file, and even
//! across file and crate boundaries.
//!
//! `macro_magic` is designed to work with stable Rust, and is fully `no_std` compatible (in
//! fact, there is a unit test to ensure everything is `no_std` safe).
//!
//! One thing that `macro_magic` _doesn't_ provide is the ability to build up state information
//! across multiple macro invocations, however this problem can be tackled effectively using
//! the [outer macro pattern](https://www.youtube.com/watch?v=aEWbZxNCH0A). There is also my
//! (deprecated but functional) [macro_state](https://crates.io/crates/macro_state) crate,
//! which relies on some incidental features of the rust compiler that could be removed in the
//! future.

/// Contains the internal code behind the `macro_magic` macros in a re-usable form, in case you
/// need to design new macros that utilize some of the internal functionality of `macro_magic`.
pub mod core {
    pub use macro_magic_core::*;
}

pub use macro_magic_macros::{
    export_tokens, export_tokens_alias, forward_tokens, import_tokens, import_tokens_attr,
    import_tokens_proc,
};

/// Contains re-exports required at compile-time by the macro_magic macros and support
/// functions.
#[doc(hidden)]
pub mod __private {
    pub use macro_magic_macros::*;
    pub use quote;
    pub use syn;
    pub use syn::__private::TokenStream2;
}
