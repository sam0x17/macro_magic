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

/// Creates an attribute proc macro that is an alias for
/// [`#[export_tokens]`](`macro@export_tokens`).
///
/// Simply pass an ident to this proc macro, and an alias for
/// [`#[export_tokens]`](`macro@export_tokens`) will be created with the specified name.
///
/// Can only be used within a proc macro crate.
#[proc_macro]
pub fn export_tokens_alias(tokens: TokenStream) -> TokenStream {
    match export_tokens_alias_internal(tokens) {
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
/// Note that the referenced item _must_ have the [`#[export_tokens]`][`macro@export_tokens`]
/// attribute attached to it, or this will not work.
///
/// This macro can be used in item contexts, and is also safe in expr contexts as long as both
/// arguments passed are idents rather than paths (can't contain `::`). This is an unfortunate
/// side effect of how decl macros are implemented in Rust
///
/// There is also an optional third argument called "extra" which allows you to forward string
/// literal data to the target macro. This is used by
/// [`#[import_tokens_attr]`](`macro@import_tokens_proc`) to pass the tokens for the attached
/// item in addition to the tokens for the external item.
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

/// Allows you to import the tokens of an external item marked with
/// [`#[export_tokens]`][`macro@export_tokens`] whose path is already known at compile-time
/// without having to do any additional parsing.
///
/// The macro lets you define as its argument a let variable declaration that will expand to
/// that variable being set to the tokens of the specified external item at compile-time.
///
/// For example:
///
/// ```ignore
/// import_tokens!(let tokens = external_crate::SomeItem);
/// ```
///
/// will expand such that a `tokens` variable will be created containing the tokens for the
/// `SomeItem` item that exists in an external crate. For this to work,
/// `external_crate::SomeItem` must be the path of an item that has
/// [`#[export_tokens]`][`macro@export_tokens`] attached to it. The imported tokens wil be of
/// type `TokenStream2`.
///
/// Unfortunately this macro isn't very useful, because it is quite rare that you already know
/// the path of the item you want to import _inside_ your proc macro. Note that having the
/// _tokens_ for the path you want isn't the same as having those tokens already expanded in
/// the current context.
///
/// That said, this can be quite useful for scenarios where for whatever reason you have an
/// item with a set-in-stone path whose tokens you need to access at compile time.
///
/// For more powerful importing capabilities, see [`macro@import_tokens_proc`] and
/// [`macro@import_tokens_attr`], which are capable of importing items based on a path that has
/// been pased to a regular proc macro or as the argument to an attribute proc macro.
#[proc_macro]
pub fn import_tokens(tokens: TokenStream) -> TokenStream {
    match import_tokens_internal(tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// An attribute macro that can be attached to a proc macro function definition that will cause
/// it to receive the tokens of the external item referred to by its argument as input to your
/// proc macro.
///
/// For example:
///
/// ```ignore
/// #[import_tokens_proc]
/// #[proc_macro]
/// pub fn my_macro(tokens: TokenStream) -> TokenStream {
///     // `tokens` will contain the tokens of
///     let item = parse_macro_input!(tokens as Item);
///     // you can now do stuff with `item`
///     // ...
/// }
/// ```
///
/// Which you could use like this:
///
/// ```ignore
/// my_macro!(some_crate::some_item);
/// ```
///
/// In this case the `tokens` variable will contain the tokens for the `some_crate::some_item`
/// item, as long as it has been marked with [`#[export_tokens]`][`macro@export_tokens`].
///
/// Can only be used within a proc macro crate.
///
/// Note that you can provide a module path as an optional argument to this attribute macro and
/// that path will be used as the override for [`MACRO_MAGIC_ROOT`] within the context of code
/// generated by this attribute.
#[proc_macro_attribute]
pub fn import_tokens_proc(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    match import_tokens_proc_internal(attr, tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Can be attached to an attribute proc macro function, causing it to receive the tokens for
/// the external item referred to by the path provided as the `attr` / first argument to the
/// attribute macro.
///
/// For this to work, the item whose path is provided as the `attr` / first argument _must_
/// have the [`#[export_tokens]`][`macro@export_tokens`] attribute attached to it, or this will
/// not work.
///
/// For example:
///
/// ```ignore
/// #[import_tokens_attr]
/// #[proc_macro_attribute]
/// pub fn my_attribute(attr: TokenStream, tokens: TokenStream) -> TokenStream {
///     let external_item = parse_macro_input!(attr as Item);
///     let attached_item = parse_macro_input!(tokens as Item);
///     // ...
/// }
/// ```
///
/// Which could then be used like:
///
/// ```ignore
/// #[my_attribute(path::to::AnItem)]
/// mod my_mod {
///     // ...
/// }
/// ```
///
/// This would result in the `external_item` variable having the parsed tokens of the external
/// `path::to::AnItem` item, and the `attached_item` variable having the parsed tokens of the
/// item the attribute is attached to (`my_mod`) as usual.
///
/// This allows to to create extremely powerful attribute macros that take in an export tokens
/// path as their `attr` and internally receive the tokens for that external item. For example
/// you could write an attribute macro that combines two modules or two structs together, among
/// many other things.
///
/// See `tests.rs` for more examples.
///
/// Can only be used within a proc macro crate.
///
/// Note that you can provide a module path as an optional argument to this attribute macro and
/// that path will be used as the override for [`MACRO_MAGIC_ROOT`] within the context of code
/// generated by this attribute.
#[proc_macro_attribute]
pub fn import_tokens_attr(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    match import_tokens_attr_internal(attr, tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// A helper macro used by [`macro@import_tokens`]. Hidden from docs.
#[doc(hidden)]
#[proc_macro]
pub fn import_tokens_inner(tokens: TokenStream) -> TokenStream {
    match import_tokens_inner_internal(tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// A helper macro used by [`macro@forward_tokens`]. Hidden from docs.
#[doc(hidden)]
#[proc_macro]
pub fn forward_tokens_inner(tokens: TokenStream) -> TokenStream {
    match forward_tokens_inner_internal(tokens) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
