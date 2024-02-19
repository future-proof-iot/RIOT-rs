# Coding Conventions

## Item Order in Rust Modules

Items SHOULD appear in Rust modules in the following order, based on [the one used by rust-analyzer](https://rust-analyzer.github.io/manual.html#auto-import):

1. Inner doc comment
1. [Inner attributes](https://doc.rust-lang.org/reference/attributes.html)
1. Unconditional—i.e., not feature-gated—bodyless [modules](https://doc.rust-lang.org/reference/items/modules.html)
1. Conditional—i.e., feature-gated—bodyless modules
1. Unconditional imports from the following:
    1. `std`/`alloc`/`core`
    1. External crates (including crates from the same workspace)
    1. Current crate, paths prefixed by `crate`
    1. Current module, paths prefixed by `self`
    1. Super module, paths prefixed by `super`
    1. Re-exports—i.e., `pub` imports not used in the module
1. Conditional imports from the same
1. [Const items](https://doc.rust-lang.org/reference/items/constant-items.html)
1. [Static items](https://doc.rust-lang.org/reference/items/static-items.html)
1. Other items

TODO: type aliases before other items?

TODO: how to organize type definitions w.r.t. related logic?

## Imports

Imports from the same crate with the same visibility MUST be [merged into a single use statement](https://rust-analyzer.github.io/manual.html#auto-import).

## Comments

### Doc Comments

All public items listed in the documentation—i.e., not marked with [`#[doc(hidden)]`](https://doc.rust-lang.org/rustdoc/write-documentation/the-doc-attribute.html#hidden)—SHOULD be documented.

Doc comments MUST use the [line comment style](https://doc.rust-lang.org/reference/comments.html#doc-comments), not the block style.

Doc comments MUST be written in third person present, not in the imperative mood, as recommended by [RFC 1574](https://github.com/rust-lang/rfcs/blob/master/text/1574-more-api-documentation-conventions.md#summary-sentence).
Each sentence in doc comments—including the first one, before an empty line—SHOULD end with a period.
For instance, instead of:

```rust
/// Get the underlying value
```

write:

```rust
/// Returns the underlying value.
```

More generally, use the [`std` docs](https://doc.rust-lang.org/stable/std/) as inspiration.

## `unsafe` Code

When `unsafe` is used, a `SAFETY` comment MUST be added, in the style [supported by Clippy](https://rust-lang.github.io/rust-clippy/master/index.html#/undocumented_unsafe_blocks).

TODO: enforce it in CI

## Naming Conventions

Names SHOULD adhere to the [official API guidelines](https://rust-lang.github.io/api-guidelines/naming.html).

TODO: how to name error types/error enum variants (`CannotDoSth` vs `DoingSth`)?

## Dependencies

If the same dependency is used in multiples crates within the workspace, that dependency SHOULD be specified in the *workspace*'s `Cargo.toml` file and workspace crates should import them from there.
