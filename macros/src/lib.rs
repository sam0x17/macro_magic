use macro_magic_core::*;
use proc_macro::TokenStream;

/// Can be applied to any [`syn::Item`]-compatible item. Doing so will make the tokens for this
/// item available for import by the other macros in this crate.
///
/// An optional argument can be provided specifying an override export name to use instead of
/// the regular name of the item, such as `#[export_tokens(MyCoolName)]` or
/// `#[export_tokens(some_name)]`. Syntactically this name is parsed as a [`struct@syn::Ident`]
/// and is then normalized by converting to snake_case. Note that because of this, `MyCoolName`
/// would collide with `my_cool_name`, resulting in a compiler error if these items are being
/// exported from the same crate.
///
/// The reason this is true of items in the same _crate_ rather than just the same _module_ is
/// because internally `#[export_tokens]` creates a `macro_rules!` / decl macro and utilizes
/// callbacks to communicate the underlying tokens of the foreign item to whatever external
/// macros might request this information, and decl macro names collide on a crate-wide basis.
/// This is why when _importing_ tokens, specifying the full path other than
/// `my_crate::my_item` is optional, since all exported tokens can be accessed directly from
/// the crate root.
///
/// A convenient further implication of this design decision is that the visibility of the
/// module containing the item you are exporting does not interfere with the accessibility of
/// that item's tokens. You can export the tokens of items in completely private modules
/// without worrying about visibility.
///
/// Note also that some types of items, namely [`syn::ItemForeignMod`], [`syn::ItemUse`],
/// [`syn::ItemImpl`], and [`syn::Item::Verbatim`], do not have an inherent concept of a naming
/// ident, and so for these items specifying an override name is required or you will get a
/// compiler error. This also applies to `macro_rules!` definitions that do not specify a name.
///
/// It is also possible to export tokens inside normally inaccessible scopes, such as inside a
/// function definition, because `#[macro_export]` works even in these scenarios (and this use
/// case is tested as part of the test suite).
///
/// ## Examples
///
/// Applied to a regular function definition:
/// ```ignore
/// #[export_tokens]
/// fn my_function() {
///     println!("hey");
/// }
/// ```
///
/// Applied to a module:
/// ```ignore
/// #[export_tokens]
/// mod my_module() {
///     fn some_fn() {
///         stuff();
///     }
/// }
/// ```
///
/// Applied to an `impl` requiring an override name:
/// ```ignore
/// #[export_tokens(impl_my_trait_for_my_item)]
/// impl MyTrait for MyItem {
///     fn something() {
///         do_stuff();
///     }
/// }
/// ```
///
/// Applied to a struct, but specifying an override name:
/// ```ignore
/// #[export_tokens(SomeOtherName)]
/// struct MyStruct {
///     field: u32,
/// }
/// ```
#[proc_macro_attribute]
pub fn export_tokens(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    match export_tokens_internal(attr, tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// "Forwards" the tokens of the specified exported item (specified by path as the first arg)
/// to the specified proc or `macro_rules!` macro (specified by path as the second arg).
///
/// This is used internally as the basis for many of the other macros in this crate, but can
/// also be useful in its own right in certain situations.
///
/// This macro can be used in item contexts, and is also safe in expr contexts as long as both
/// arguments passed are idents rather than paths (can't contain `::`). This is an unfortunate
/// side effect of how decl macros are implemented in Rust
///
/// There is also an optional third argument called "extra" which allows you to forward string
/// literal data to the target macro. This is used by
/// [`#[import_tokens_attr]`](`import_tokens_proc`) to pass the tokens for the attached item
/// in addition to the tokens for the external item.
///
/// ## Example
///
/// ```ignore
/// #[macro_export]
/// macro_rules! receiver {
///     ($tokens:item) => {
///         stringify!($tokens)
///     };
/// }
///
/// let result = forward_tokens!(LionStruct, receiver);
/// assert_eq!(result, "struct LionStruct {}");
/// ```
#[proc_macro]
pub fn forward_tokens(tokens: TokenStream) -> TokenStream {
    match forward_tokens_internal(tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn import_tokens(tokens: TokenStream) -> TokenStream {
    match import_tokens_internal(tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn import_tokens_inner(tokens: TokenStream) -> TokenStream {
    match import_tokens_inner_internal(tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn forward_tokens_inner(tokens: TokenStream) -> TokenStream {
    match forward_tokens_inner_internal(tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn import_tokens_proc(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    match import_tokens_proc_internal(attr, tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn import_tokens_attr(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    match import_tokens_attr_internal(attr, tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
