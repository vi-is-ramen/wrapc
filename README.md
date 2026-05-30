# wrapc

> A zero-fuss, type-safe parser for `rustc` arguments, designed for `RUSTC_WRAPPER` tools.

[![Crates.io](https://img.shields.io/crates/v/wrapc.svg)](https://crates.io/crates/wrapc)
[![Documentation](https://img.shields.io/badge/docs-book-blue)](https://vi-is-ramen.github.io/book/en/wrapc)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

`wrapc` parses `rustc`'s notoriously complex CLI into strongly-typed Rust structs — with flawless round-trip reconstruction via `Info::to_args()`.

## Why not `clap`?

`rustc`'s argument surface is unique:
- Mixed `=` and space-separated values (`--edition=2021` vs `-L native=/path`)
- Complex sub-syntaxes (`-l static:+bundle,+whole-archive=foo:bar`)
- Nightly flags that appear/disappear without notice
- The `RUSTC_WRAPPER` protocol: `<wrapper> - <rustc> <args...>`

`clap` is too rigid for this. `wrapc` is purpose-built.

## Quick Start

Add to your dependencies:

```shell
cargo add wrapc
```

Canonical wrapper skeleton:

```rust
use std::process::Command;
use wrapc::fetch;

fn main() {
    let info = fetch().expect("Failed to parse rustc arguments");
    
    // Inspect or mutate
    if info.crate_name.as_deref() == Some("legacy") {
        // inject flags, log, cache, etc.
    }
    
    // Reconstruct and execute
    let rustc = info.rustc.unwrap_or_else(|| "rustc".into());
    let status = Command::new(rustc).args(info.to_args()).status()?;
    std::process::exit(status.code().unwrap_or(1));
}
```

## Documentation

Full API reference, protocol details, and advanced usage patterns are available in the [book](https://vi-is-ramen.github.io/book/en/wrapc).

## License

MIT OR Apache-2.0
