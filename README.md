# Macro Magic 🪄

[![Build Status](https://img.shields.io/github/actions/workflow/status/sam0x17/macro_magic/ci.yaml)](https://github.com/sam0x17/macro_magic/actions/workflows/ci.yaml?query=branch%3Amain)
[![MIT License](https://img.shields.io/github/license/sam0x17/macro_magic)](https://github.com/sam0x17/macro_magic/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/d/macro_magic)](https://crates.io/crates/macro_magic)
[![docs.rs](https://img.shields.io/docsrs/macro_magic?label=docs)](https://docs.rs/macro_magic/latest/macro_magic/)

This crate provides the `#[export_tokens]` macro and a number of companion macros, including
the `#[import_tokens_proc]` and `#[import_tokens_attr]` macros. When used in tandem with
`#[export_tokens]`, these macros allow you to create regular and attribute proc macros in which
you can import and make use of the tokens of external/foreign items marked with
`#[export_tokens]` in other modules, files, and even in other crates merely by referring to
them by name/path.

Among other things, the patterns introduced by `macro_magic` can be used to implement safe and
efficient exportation and importation of item tokens within the same file, and even across file
and crate boundaries.

`macro_magic` is designed to work with stable Rust, and is fully `no_std` compatible (in fact,
there is a unit test to ensure everything is `no_std` safe). The main crate and all sub-crates
are `no_std`.

## General Syntax

You can use `macro_magic` to build regular and attribute proc macros that look like this:

```rust
#[my_attribute(path::to::MyItem)]
trait SomeTrait {
    // ..
}
```

this:

```rust
do_something!(path::to::MyItem);
```

or even this:

```rust
let foreign_tokens = my_macro!(path::to::MyItem);
assert_eq!(foreign_tokens.to_string(), "struct MyItem {...}");
```

where `path::to::MyItem` is the path to an item that has been marked with `#[export_tokens]`.

All of this behavior is accomplished under the hood using proc macros that create
`macro_rules`-based callbacks, but as a programmer this complexity is completely hidden from
you via simple attribute macros you can apply to your proc macros to imbue them with the power
of importing the tokens for external items based on their path.

## Attribute Example

You could write an attribute macro to "inject" the fields of one struct into
another as follows:

```rust
#[import_tokens_attr]
#[proc_macro_attribute]
pub fn combine_structs(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let foreign_struct = parse_macro_input!(attr as ItemStruct);
    let local_struct = parse_macro_input!(tokens as ItemStruct);
    let Fields::Named(local_fields) = local_struct.fields else {
        return Error::new(
            local_struct.fields.span(),
            "unnamed fields are not supported"
        ).to_compile_error().into()
    };
    let Fields::Named(foreign_fields) = foreign_struct.fields else {
        return Error::new(
            foreign_struct.fields.span(),
            "unnamed fields are not supported"
        ).to_compile_error().into()
    };
    let local_fields = local_fields.named.iter();
    let foreign_fields = foreign_fields.named.iter();
    let attrs = local_struct.attrs;
    let generics = local_struct.generics;
    let ident = local_struct.ident;
    let vis = local_struct.vis;
    quote! {
        #(#attrs)
        *
        #vis struct #ident<#generics> {
            #(#local_fields),
            *
            ,
            #(#foreign_fields),
            *
        }
    }
    .into()
}
```

And then you could use the `#[combine_structs]` attribute as follows:

```rust
#[export_tokens]
struct ExternalStruct {
    foo: u32,
    bar: u64,
    fizz: i64,
}

#[combine_structs(ExternalStruct)]
struct LocalStruct {
    biz: bool,
    baz: i32,
}
```

Which would result in the following expanded output for `LocalStruct`:

```rust
struct LocalStruct {
    foo: u32,
    bar: u64,
    fizz: i64,
    biz: bool,
    baz: i32,
}
```

Note that the `attr` variable on the `combine_structs` proc macro, thanks to the powers of
`#[import_tokens_attr]`, will receive the actual tokens for the `ExternalStruct` item, rather
than merely receiving the tokens for the path `ExternalStruct`.

This gives you the ability to write attribute macros that receive tokens for two items, one
specified by path via the first argument `attr`, as well as the tokens for the item the
attribute is attached to via the 2nd argument `tokens`. The only requirement is that the item
specified by `attr` has been marked with `#[export_tokens]`.

## Proc Macro Example

You could write a PHP/ruby/crystal-style verbatim import / `require` macro which blindly
imports the tokens of the specified external module into the current context (with all the good
and bad implications that would imply), like this:

```rust
#[import_tokens_proc]
#[proc_macro]
pub fn require(tokens: TokenStream) -> TokenStream {
    let external_mod = parse_macro_input!(tokens as ItemMod);
    let Some((_, stmts)) = external_mod.content else {
        return Error::new(
            external_mod.span(),
            "cannot import tokens from a file-based module since custom file-level \
            attributes are not yet supported by Rust"
        ).to_compile_error().into()
    };
    quote! {
        #(#stmts)
        *
    }
    .into()
}
```

You could then use the `require!` macro like this:

```rust
// in some external crate
#[export_tokens]
mod an_external_module {
    fn my_cool_function() -> u32 {
        567
    }

    fn my_other_function() -> u32 {
        116
    }
}
```

```rust
// in another crate where we will use the `require!` macro
mod my_module {
    use my_macros::require;

    fn existing_stuff() {
        println!("foo!");
    }

    require!(external_crate::an_external_module);
}
```

which would result in this expansion of `require!` within `my_module`:

```rust
mod my_module {
    use my_macros::require;

    fn existing_stuff() {
        println!("foo!");
    }

    fn my_cool_function() -> u32 {
        567
    }

    fn my_other_function() -> u32 {
        116
    }
}
```

Notice that this hypothetical `require!` macro is dangerous for two reasons:

- Any types you may have brought into scope with `use` statements in the foreign module may or
  may not be available in their new context without additional use statements.
- If existing items in the module or context where you use the `require!` macro conflict with
  something you are importing, you will get a compiler error (this is good, though).

These are just _some_ of the capabilities of `macro_magic` 🪄

## Features

### pretty_print

The `pretty_print` feature, when enabled, adds a `pretty_print` function to
`macro_magic::mm_core` which is capable of printing anything compatible with
`Into<TokenStream2>` and is highly useful for debugging. This feature is not enabled by default
since it relies on some things that can be problematic in `no_std` environments.

## Limitations

One thing that `macro_magic` _doesn't_ provide is the ability to build up state information
across multiple macro invocations, however this problem can be tackled effectively using the
[outer macro pattern](https://www.youtube.com/watch?v=aEWbZxNCH0A). There is also my
(deprecated but functional) [macro_state](https://crates.io/crates/macro_state) crate, which
relies on some incidental features of the rust compiler that could be removed in the future.

Note that the transition from 0.1.7 to 0.2.0 of `macro_magic` removed and/or re-wrote a number
of features that relied on a non-future-proof behavior of writing/reading files from the
`OUT_DIR`. Versions of `macro_magic` >= 0.2.0 are completely future-proof and safe, however
features that provided the ability to enumerate all the `#[export_tokens]` calls in a namespace
have been removed. The proper way to do this is with the outer macro pattern, mentioned above.
