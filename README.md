# Macro Magic 🪄

[![Build Status](https://img.shields.io/github/actions/workflow/status/sam0x17/macro_magic/ci.yaml)](https://github.com/sam0x17/macro_magic/actions/workflows/ci.yaml?query=branch%3Amain)
[![MIT License](https://img.shields.io/github/license/sam0x17/macro_magic)](https://github.com/sam0x17/macro_magic/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/d/macro_magic)](https://crates.io/crates/macro_magic)
[![docs.rs](https://img.shields.io/docsrs/macro_magic?label=docs)](https://docs.rs/macro_magic/latest/macro_magic/)

This crate provides an `#[export_tokens]` attribute macro, and a number of companion macros
which, when used in tandem, allow you to create regular and attribute proc macros in which you
can import and make use of the tokens of foreign items marked with `#[export_tokens]` in other
modules, files, and even in other crates merely by referring to them by name/path.

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

For example, you could write an attribute macro to "inject" the fields of one struct into
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

Among other things, the patterns introduced by `macro_magic` can be used to implement safe and
efficient coordination and communication between macro invocations in the same file, and even
across different files and different crates. This crate officially supercedes my previous
effort at achieving this, [macro_state](https://crates.io/crates/macro_state), which was
designed to allow for building up and making use of state information across multiple macro
invocations. All of the things you can do with `macro_state` you can also achieve with this
crate, albeit with slightly different patterns.

`macro_magic` is designed to work with stable Rust, and is fully `no_std` compatible (in fact,
there is a unit test to ensure everything is `no_std` safe).
