//! # Macro Magic 🪄
//!
//! ![Build Status](https://img.shields.io/github/actions/workflow/status/sam0x17/macro_magic/ci.yaml)
//! ![GitHub](https://img.shields.io/github/license/sam0x17/macro_magic)
//! ![Crates.io](https://img.shields.io/crates/d/macro_magic)
//! ![docs.rs](https://img.shields.io/docsrs/macro_magic?label=docs)
//!
//! This crate provides two powerful proc macros, [`#[export_tokens]`](`macro@export_tokens`)
//! and [`import_tokens!`]. When used in tandem, these two macros allow you to mark items in
//! other files (and even in other crates, as long as you can modify the source code) for
//! export. The tokens of these items can then be imported by the [`import_tokens!`] macro using
//! the path to an item you have exported.
//!
//! Two advanced macros, [`import_tokens_indirect!`] and [`read_namespace!`] are also provided
//! when the "indirect" feature is enabled. These macros are capable of going across crate
//! boundaries without complicating your dependencies and can return collections of tokens
//! based on a shared common prefix.
//!
//! Among other things, the patterns introduced by Macro Magic, and in particular by the
//! "indirect" feature be used to implement safe and efficient coordination and communication
//! between macro invocations in the same file, and even across different files and different
//! crates. This crate officially supercedes my previous effort at achieving this,
//! [macro_state](https://crates.io/crates/macro_state), which was designed to allow for
//! building up and making use of state information across multiple macro invocations. All of
//! the things you can do with `macro_state` you can also achieve with this crate, albeit with
//! slightly different patterns.
//!
//! `macro_magic` is designed to work with stable Rust.

/// Contains the internal code behind the `macro_magic` macros in a re-usable and extensible
/// form, including the ability to make custom macros that behave like `#[export_tokens]` and
/// `import_tokens_indirect!`. This module obeys the "indirect" / "indirect-read"
/// "indirect-write" feature conventions so make sure the proper features are enabled if you
/// are trying to access anything involving indirect exports/imports.
pub mod core {
    pub use macro_magic_core::*;
}

pub use macro_magic_macros::{export_tokens, forward_tokens, import_tokens, import_tokens_attr};

#[macro_export]
macro_rules! expand_item_safe {
    ($($tokens:tt)*) => {
        $($tokens)*
    };
}

/// Contains re-exports required at compile-time by the macro_magic macros and support
/// functions. This includes a re-export of [`import_tokens_inner`] and some [`syn`]-related
/// types include [`TokenStream2`].
#[doc(hidden)]
pub mod __private {
    pub use macro_magic_macros::*;
    pub use quote;
    pub use syn;
    pub use syn::__private::TokenStream2;
}
