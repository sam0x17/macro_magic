//! # Macro Magic 🪄
//!
//! ![Build Status](https://img.shields.io/github/actions/workflow/status/sam0x17/macro_magic/ci.yaml)
//! ![GitHub](https://img.shields.io/github/license/sam0x17/macro_magic)
//! ![Crates.io](https://img.shields.io/crates/d/macro_magic)
//! ![docs.rs](https://img.shields.io/docsrs/macro_magic?label=docs)
//!
//! ## Overview
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
//! ## no_std
//!
//! `macro_magic` is designed to work with stable Rust, and is fully `no_std` compatible (in
//! fact, there is a unit test to ensure everything is `no_std` safe). The main crate and all
//! sub-crates are `no_std`.
//!
//! ## Features
//!
//! ### proc_support
//!
//! The `proc_support` feature _must_ be enabled in proc macro crates that make use of any
//! import tokens functionality, including [`#[import_tokens_attr]`](`import_tokens_attr`),
//! [`#[import_tokens_proc]`](`import_tokens_proc`) and [`import_tokens!`]. Otherwise these
//! macros will not function correctly and will issue compiler errors complaining about items
//! not existing under [`mm_core`]. The [`#[export_tokens]`](`export_tokens`) macro does not
//! require this feature to function correctly, so you can safely use it without enabling this
//! feature.
//!
//! The reason for this feature gating is that things like [`syn`], [`quote`], `proc_macro2`,
//! etc., are not 100% `no_std` compatible and should only be enabled in proc macro crates
//!
//! ### pretty_print
//!
//! The `pretty_print` feature, when enabled, adds a `pretty_print` function to [`mm_core`]
//! which is capable of printing anything compatible with [`Into<TokenStream2>`] and is highly
//! useful for debugging. This feature is not enabled by default since it relies on some things
//! that can be problematic in `no_std` environments.
//!
//! ## Limitations
//!
//! One thing that `macro_magic` _doesn't_ provide is the ability to build up state information
//! across multiple macro invocations, however this problem can be tackled effectively using
//! the [outer macro pattern](https://www.youtube.com/watch?v=aEWbZxNCH0A). There is also my
//! (deprecated but functional) [macro_state](https://crates.io/crates/macro_state) crate,
//! which relies on some incidental features of the rust compiler that could be removed in the
//! future.
//!
//! Note that the transition from 0.1.7 to 0.2.0 of `macro_magic` removed and/or re-wrote a
//! number of features that relied on a non-future-proof behavior of writing/reading files from
//! the `OUT_DIR`. Versions of `macro_magic` >= 0.2.0 are completely future-proof and safe,
//! however features that provided the ability to enumerate all the `#[export_tokens]` calls in
//! a namespace have been removed. The proper way to do this is with the outer macro pattern,
//! mentioned above.

#![no_std]

/// Contains the internal code behind the `macro_magic` macros in a re-usable form, in case you
/// need to design new macros that utilize some of the internal functionality of `macro_magic`.
pub mod mm_core {
    #[cfg(feature = "proc_support")]
    pub use macro_magic_core::*;

    #[cfg(feature = "pretty_print")]
    pub use macro_magic_core::pretty_print;
}

pub use macro_magic_macros::{
    export_tokens, export_tokens_alias, forward_tokens, use_attr, use_proc,
};

#[cfg(feature = "proc_support")]
pub use macro_magic_macros::{import_tokens, import_tokens_attr, import_tokens_proc};

/// Contains re-exports required at compile-time by the macro_magic macros and support
/// functions.
#[doc(hidden)]
pub mod __private {
    pub use macro_magic_macros::*;

    #[cfg(feature = "proc_support")]
    pub use quote;

    #[cfg(feature = "proc_support")]
    pub use syn;

    #[cfg(feature = "proc_support")]
    pub use syn::__private::TokenStream2;
}
