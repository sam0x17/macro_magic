pub use macros::*;

#[doc(hidden)]
pub mod __private {
    pub use syn::__private::TokenStream2;
}

pub mod macro_magic {
    pub use super::*;
}
