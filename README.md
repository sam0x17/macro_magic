# Macro Magic 🪄

[![Build Status](https://img.shields.io/github/actions/workflow/status/sam0x17/macro_magic/ci.yaml)](https://github.com/sam0x17/macro_magic/actions/workflows/ci.yaml?query=branch%3Amain)
[![MIT License](https://img.shields.io/github/license/sam0x17/macro_magic)](https://github.com/sam0x17/macro_magic/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/d/macro_magic)](https://crates.io/crates/macro_magic)
[![docs.rs](https://img.shields.io/docsrs/macro_magic?label=docs)](https://docs.rs/macro_magic/latest/macro_magic/)

This crate provides two powerful proc macros, `#[export_tokens]` and `import_tokens!`. When
used in tandem, these two macros allow you to mark items in other files (and even in other
crates, as long as you can modify the source code) for export. The tokens of these items can
then be imported by the `import_tokens!` macro using the path to an item you have exported.

Two advanced macros, `import_tokens_indirect!` and `read_namespace!` are also provided when the
"indirect" feature is enabled. These macro are capable of going across crate boundaries without
complicating your dependencies and can return collections of tokens based on a shared common
prefix.

Among other things, the patterns introduced by Macro Magic can be used to implement safe and
efficient coordination and communication between macro invocations in the same file, and even
across different files and different crates. This crate officially supercedes my previous
effort at achieving this, [macro_state](https://crates.io/crates/macro_state), which was
designed to allow for building up and making use of state information across multiple macro
invocations. All of the things you can do with `macro_state` you can also achieve with this
crate, albeit with slightly different patterns.

`macro_magic` is designed to work with stable Rust.

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

Even this caveat can be removed if you make use of indirect imports (explained below), which
are capable of working without requiring the token source to be a dependency of the target.

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
this input is optional. Furthermore the path need not exist -- you just have to use the same
path when you import.

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

This style of importing is called a direct import because we directly include the code we are
exporting into the context where the tokens are being used (usually a proc macro crate).

### Expansion

The example above would roughly expand to:

```rust
let tokens = cool::path::__EXPORT_TOKENS__FOO_BAR.parse::<TokenStream2>().unwrap();
```

## import_tokens_indirect!

While direct imports are useful, there are situations where it would be impractical or
extremely cumbersome to have the crate where your tokens are exported from (i.e. the "source"
crate) be a dependency of your proc macro crate where those tokens are used (i.e. the "target
crate"). This is especially true in scenarios where your proc macro crate is consumed by
arbitrary downstream users who cannot modify your proc macro crate in any way without forking
it. We provide a workaround via what we call "indirect imports". Another use-case for indirect
imports is scenarios where the item in question is hidden behind a private module, as indirect
imports can work around this scenario.

Calling `import_tokens_indirect!` is slightly different from calling `import_tokens!` in that
indirect imports will work even when the item whose tokens you are importing is contained in a
crate that is not a dependency of the current crate, so long as the following requirements are
met:

1. The "indirect" feature must be enabled for `macro_magic`, otherwise the
   `import_tokens_indirect!` macro will not be available.
2. The source crate and the target crate must be in the same
   [cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html). This is a
   non-negotiable hard requirement when using indirect imports, however direct imports will
   work fine across workspace boundaries (they just have other stricter requirements that can
   be cumbersome).
3. The source crate and the target crate must both use the same version of `macro_magic` (this
   is not a hard requirement, but undefined behavior could occur with mixed versions).
4. Both the source crate and target crate must be included in the compilation target of the
   current workspace such that they are both compiled. Unlike with direct imports, where you
   explictily `use` the source crate as a dependency of the target crate, there needs to be
   some reason to compile the source crate, or its exported tokens will be unavailable.
5. The export path declared by the source crate must exactly match the path you try to import
   in the target crate. If you don't manually specify an export path, then your import path
   should be the name of the item that `#[export_tokens]` was attached to (i.e. the `Ident`),
   however this approach is not recommended since you can run into collisions if you are not
   explicit about naming. For highly uniquely named items, however, this is fine.
6. The target crate _must_ be a proc macro crate.

The vast majority of common use cases for `macro_magic` meet these criteria, but if you run
into any issues where exported tokens can't be found, make sure your source crate is included
as part of the compilation target and that it is in the current workspace.

Keep in mind that you can use the optional attribute, `#[export_tokens(my::path::Here)]` to
specify a disambiguation path for the tokens you are exporting. Otherwise the name of the item
the macro is attached to will be used, potentially causing collisions if you export items by
the same name from different contexts.

This situation will eventually be resolved when the machinery behind
[caller_modpath](https://crates.io/crates/caller_modpath) is stabilized, which will allow
`macro_magic` to automatically detect the path of the `#[export_tokens]` caller.

A peculiar aspect of how `#[export_tokens(some_path)]` works is the path you enter doesn't need
to be a real path. You could do `#[export_tokens(completely::made_up::path::MyItem)]` in one
context and then `import_tokens!(completely::made_up::path::MyItem)` in another context, and it
will still work as long as these two paths are the same. They need not actually exist, they are
just used for disambiguation so we can tell the difference between these tokens and other
potential exports of an item called `MyItem`. The last segment _does_ need to match the name of
the item you are exporting, however.

## read_namespace!

Namespaces support (included as part of the "indirect" feature) allows you to group a number of
`#[export_tokens]` calls and collect them into a `Result<Vec<(String, TokenStream2)>>` with the
the `read_namespace!(some::namespace)` macro, where the first component of the tuple
corresponds with the name of the item and the second component contains the tokens for that
item. The `Result` is a `std::io::Result` and any `Err` variants that come back would indicate
an internal error (i.e. something tampered with the `target` directory at an unexpected time)
or (more likely) that the specified namespace does not exist.

The `#[export_tokens]` attribute automatically defines namespaces when you call it with an
argument. Namespaces function like directories, so if you define item A with the path
`foo::bar::fizz` and item B with path `foo::bar::buzz`, and you will get back both items if you
read the namespace `foo::bar` i.e.:

```rust
let namespace_items = read_namespace!(foo::bar).unwrap();
let (name, tokens) = namespace_items.first().unwrap();
// name = "buzz"
// tokens = tokens for `foo::bar::buzz` item
```

Note that `read_namespace!` always returns results sorted by name, so you can rely on the order
to be consistent.

## re_export_tokens_const!

This macro allows you to re-export already exported tokens across modules and even crates. See
the docs for more info.

## Features

By default no features are enabled. The following features are supported:

### "verbose"

The "verbose" feature is disabled by default. If enabled, some extra debugging information will
be printed at compile-time indicating when files are written and read from the `REFS_DIR` for
the purpose of debugging `import_tokens_indirect!`.

Normal users of the crate should not need this feature, however it is quite useful if things go
wrong for some reason.

### "indirect"

The "indirect" feature is disabled by default. When this feature is disabled, only
`#[export_tokens]`, `import_tokens!` and `re_export_tokens_const!` will be available and the
`read_namespace!` and `import_tokens_indirect!` macros will not be compiled. When "indirect" is
enabled, all of these macros will be available and you will be able to do indirect imports and
read namespaces. Namespaces and indirect imports are _only_ supported when the "indirect"
feature is enabled (or more specifically, the "indirect-read" feature).

Internally this feature enables both the "indirect-write" and "indirect-read" features.

### "indirect-read"

The "indirect-read" feature will enable indirect imports via `import_tokens_indirect!` and
`read_namespace!` without also enabling indirect writes via `#[export_tokens]`.

This feature is turned on automatically if "indirect" is enabled.

### "indirect-write"

The "indirect-write" feature will enable direct-import-compatible `#[export_tokens]` support.

This feature is turned on automatically if "indirect" is enabled.

## Overhead

Because the automatically generated constants created by `#[export_tokens]` are only used in a
proc-macro context, these constants do not add any bloat to the final binary because they will
be optimized out in contexts where they are not used. Thus these constants are a zero-overhead
abstraction once proc-macro expansion completes. The same goes for the temporary files used by
the indirect imports approach. These artifacts only exist at compile time and do not make it
into the final binary.

On a micro-scale, direct imports are slightly more efficient than indirect imports because they
do not involve any extra IO activity, using only a `const` to synchronize information between
source and target.

## Safety

Direct imports via `import_tokens!` are 100% safe and don't rely on anything sketchy about
compile-order or artifacts in the `target` directory.

Indirect imports are also safe because of how the `macro_magic` build script is constructed
(unlike `macro_state`, which may stop working in the future depending on what changes are made
to the Rust language), however, under the hood indirect imports do rely on coordinating based
on files in the `target` directory for the current workspace, so mileage may vary depending on
the context where you try to use this approach.

For this reason it is recommended to stick with `import_tokens!` unless your use case requires
the extra flexibility provided by `import_tokens_indirect!`. You can disable
`import_tokens_indirect!` completely by not opting in to the "indirect" feature.

## no_std

This crate is `no_std` safe if you do not use the "indirect" feature at all, or if you only use
the "indirect-write" feature when exporting tokens. The "indirect-read" feature brings in some
std-dependent stuff that will break `no_std` compatibility. This is fine when in a proc macro,
however since `#[export_tokens]` is used outside proc macro land you will want to only enable
the "indirect-write" and not enable the "indirect-read" feature when in non-proc-macro
contexts. If you don't care about `no_std` support, you can simply use the "indirect" feature
which enables both "indirect-read" and "indirect-write".

TLDR: If you care about `no_std` support and need to use indirect imports/exports, on your
crate that needs to call `#[export_tokens]`, only enable the "indirect-write" feature, and do
not enable the "indirect-read" feature. Elsewhere you can do whatever you want.
