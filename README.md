# Macro Magic 🪄

This crate provides two powerful proc macros, `#[export_tokens]` and `import_tokens!`. When
used in tandem, these two macros allow you to mark items in other files (and even in other
crates, as long as you can modify the source code) for export. The tokens of these items can
then be imported by the `import_tokens!` macro using the path to an item you have exported.

## Example

Let's say you have some module that defines a bunch of type aliases like this:

```rust
// src/bar/baz.rs

pub mod foo {
    type Foo = u32;
    type Bar = usize;
    type Fizz = String;
    type Buzz = bool;
}
```

And let's say you are writing some proc macro somewhere else, and you realize you really need
to know what types have and have not been defined in the `bar::baz::foo` module shown above,
perhaps so you can provide default values for these type aliases if they are not present.

```rust
#[proc_macro]
pub fn my_macro(tokens: TokenStream) -> TokenStream {
    // ...
    let foo_tokens: TokenStream2 = ???
    // ...
}
```

We need the tokens from some item that hasn't been passed to our macro here. How can we get
them?

Well, you can attach the `#[export_tokens]` attribute macro to the `foo` module as follows:

```rust
// src/bar/baz.rs

use macro_magic::export_tokens;

#[export_tokens]
pub mod foo {
    type Foo = u32;
    // ...
}
```

Now you can import the tokens for the entire `foo` module inside of `my_macro` even though
they are in different crates. The only caveat is that you have to import the `foo` module to
the context where you are writing your macro, like so:

```rust
use bar::baz::foo;

use macro_magic::import_tokens;

#[proc_macro]
pub fn my_macro(tokens: TokenStream) -> TokenStream {
    let foo_tokens: TokenStream = import_tokens!(foo).into(); // type is TokenStream2
    let parsed_mod: ItemMod = parse_macro_input!(foo_tokens as ItemMod);
    // ...
}
```

## `#[export_tokens]`

You can apply the `#[export_tokens]` macro to any
[Item](https://docs.rs/syn/latest/syn/enum.Item.html), with the exception of foreign modules,
impls, unnamed decl macros, and use declarations.

When you apply `#[export_tokens]` to an item, a `const` variable is generated immediately after
the item and set to a `&'static str` containing the source code of the item. The `const`
variable is hidden from docs and its name consists of the upcased item name (i.e. the ident),
prefixed with `__EXPORT_TOKENS__`, to avoid any collisions with any legitimate constants that
may have been defined.

This allows the tokens for the item to be imported using the `import_tokens!` macro.

Optionally, you may specify a disambiguation path for the item as an argument to the macro,
such as:

```rust
#[export_tokens(my::cool::ItemName)]
fn my_item () {
    // ...
}
```

Any valid `syn::TypePath`-compatible item is acceptable as input for `#[export_tokens]` and
this input is optional.

### Expansion

```rust
#[export_tokens]
fn foo_bar(a: u32) -> u32 {
    a * 2
}
```

expands to:

```rust
#[allow(dead_code)]
fn foo_bar(a: u32) -> u32 {
    a * 2
}
#[allow(dead_code)]
#[doc(hidden)]
pub const __EXPORT_TOKENS__FOO_BAR: &'static str = "fn foo_bar(a : u32) -> u32 { a * 2 }";
```

NOTE: items marked with `#[export_tokens]` do not need to be public, however they do need to be
in a module that is accessible from wherever you intend to call `import_tokens!`.

## `import_tokens!`

You can pass the path of any item that has had the `#[export_tokens]` attribute applied to it
directly to the `import_tokens!` macro to get a
[TokenStream2](https://docs.rs/proc-macro2/latest/proc_macro2/struct.TokenStream.html) of the
foreign item.

For example, suppose the `foo_bar` function mentioned above is located in another crate and can
be accessed via `really::cool::path::foo_bar`. As long as that path is accessible from the
current context (i.e. could be loaded via a `use` statement if you wanted to), `import_tokens!`
will expand to a `TokenStream2` of the item, e.g.:

```rust
let tokens = import_tokens!(cool::path::foo_bar);
```

### Expansion

The example above would roughly expand to:

```rust
let tokens = cool::path::__EXPORT_TOKENS__FOO_BAR.parse::<TokenStream2>().unwrap();
```

## Overhead

Because the automatically generated constants created by `#[export_tokens]` are only used in a
proc-macro context, these constants do not add any bloat to the final binary because they will
be optimized out in contexts where they are not used. Thus these constants are a zero-overhead
abstraction once proc-macro expansion completes.
