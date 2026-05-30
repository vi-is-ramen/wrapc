//! A zero-fuss, type-safe parser for `rustc` arguments, designed for `RUSTC_WRAPPER` tools.
//!
//! Part of the [`Inherit`](https://crates.io/crates/cargo-inherit) ecosystem.
//! For detailed documentation, protocol specification, and advanced usage examples, see the
//! [Inherit Book — wrapc chapter](https://vi-is-ramen.github.io/book/en/wrapc).

mod error;
pub use error::*;

mod info;
pub use info::*;

mod parser;
pub use parser::*;
