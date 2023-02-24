# Macro Magic 🪄

This crate provides two powerful proc macros, `#[export_tokens]` and `import_tokens!`.
When used in tandem, these two macros allow you to mark items in other files (and even
in other crates, as long as you can modify the source code) for export. The tokens of these
items can then be imported by the `import_tokens!` macro by passing in the path to an item you
have exported.

For example, let's say you have some module that defines a bunch of type aliases like this:

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
to know what types have and have not been defined in our `bar::baz::foo` module shown above,
perhaps so you can provide default values for these type aliases if they are not present.

```rust
#[proc_macro]
pub fn my_macro(tokens: TokenStream) -> TokenStream {
    let external_tokens: TokenStream = ???
}
```

We need the tokens from some item that hasn't been passed to our macro here. How can we get
them?

Well normally you'd be out of luck, but suppose you attach the `#[export_tokens]` attribute
macro to the `bar::baz::foo` module as follows:

```rust
// src/bar/baz.rs

use macro_magic::*;

#[export_tokens]
pub mod foo {
    type Foo = u32;
    // ...
}
```
