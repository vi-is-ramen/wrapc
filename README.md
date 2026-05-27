# wrapc

[![Crates.io](https://img.shields.io/crates/v/wrapc.svg)](https://crates.io/crates/wrapc)
[![Documentation](https://docs.rs/wrapc/badge.svg)](https://docs.rs/wrapc)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**A zero-fuss, type-safe parser for `rustc` arguments, designed specifically for `RUSTC_WRAPPER` tools.**

---

## Overview

Building tools that intercept Rust compilation (such as compilation caches, custom profilers, static analyzers, or linker-flag injectors) requires parsing `rustc`'s command-line arguments. 

However, `rustc`'s CLI is notoriously complex. It mixes `=` and space-separated values, features intricate sub-syntaxes for linking (`-l kind:+modifiers=name:rename`), and constantly evolves with new nightly flags. Standard CLI parsers like `clap` are too rigid and bloated for this specific use case.

`wrapc` solves this by providing a comprehensive, strongly-typed, and easily extensible parser that perfectly understands the `RUSTC_WRAPPER` protocol, guaranteeing **flawless round-trip reconstruction** of arguments.

## Features

- **Wrapper Protocol**: handles Cargo's `<wrapper> - <rustc> <args...>` invocation format.
- **Type-Safe & Comprehensive**: Parses complex flags like `--emit`, `--extern`, `-L`, and `-l` into structured Rust types instead of raw strings.
- **Flawless Round-Tripping**: The `Info::to_args()` method perfectly reconstructs the parsed arguments, ensuring zero data loss when passing them down to the actual `rustc`.
- **Lightweight**: No heavy macro dependencies or `clap`. Just fast, predictable, and extensible string parsing.
- **Panic-Free**: Gracefully handles missing values, unrecognized/nightly flags, and the `--` separator without crashing your build pipeline.

## Quick Start

Add `wrapc` to your wrapper tool's dependencies:

```toml
cargo add wrapc
```

### The Canonical Wrapper Skeleton

```rust
use std::process::Command;
use wrapc::fetch;

fn main() {
    // `fetch()` reads `std::env::args()` and parses them.
    let mut info = fetch().expect("Failed to parse rustc arguments");

    // 1. Handle passthrough commands (help/version)
    if info.help || info.version {
        // Just execute rustc and exit
    }

    // 2. Inspect or mutate the compilation context
    if info.crate_name.as_deref() == Some("legacy_crate") {
        info.codegen_opts.push("opt-level=1".to_string());
    }

    // 3. Determine the real rustc path
    let rustc_path = info.rustc.unwrap_or_else(|| "rustc".to_string());

    // 4. Reconstruct the arguments perfectly
    let args = info.to_args();

    // 5. Execute the actual compiler
    let status = Command::new(rustc_path)
        .args(&args)
        .status()
        .expect("Failed to spawn rustc");

    std::process::exit(status.code().unwrap_or(1));
}
```

## Deep Dive: Complex Flags

`rustc`'s linking flags are highly structured. `wrapc` parses modifiers, kinds, and renames automatically so you don't have to write regexes.

```rust
use wrapc::{LibrarySearchPathKind, LinkLibKind};

// Example invocation: 
// rustc -L native=/opt/lib -l static:+bundle,+whole-archive=mylib:renamed

for lib_path in &info.libpaths {
    if lib_path.kind == LibrarySearchPathKind::Native {
        println!("Native search path: {:?}", lib_path.path);
    }
}

for link in &info.links {
    println!("Linking: {}", link.name);
    if let Some(LinkLibKind::Static) = link.kind {
        println!("  Modifiers: {:?}", link.modifiers); // ["+bundle", "+whole-archive"]
        println!("  Renamed to: {:?}", link.rename);   // Some("renamed")
    }
}
```

## The `RUSTC_WRAPPER` Protocol & Edge Cases

When Cargo invokes a wrapper, it uses a specific protocol:
`<wrapper_binary> - <actual_rustc_path> <rustc_args...>`

`wrapc` automatically strips the first three arguments and stores the real compiler path in `info.rustc`. 

### The `stdin` Edge Case
Tools like `cargo` or IDEs sometimes pass `-` as the input file to tell `rustc` to read source code from `stdin`. Because Cargo's wrapper protocol *also* uses `-` as a separator, naive parsers break here. `wrapc` handles this contextually:

```rust
// Invocation: `my_wrapper - rustc -`
let info = wrapc::fetch().unwrap();

assert_eq!(info.rustc, Some("rustc".to_string()));
assert_eq!(info.inputs, vec![std::path::PathBuf::from("-")]); // Correctly identified!
```

### Unrecognized / Nightly Flags
If a user passes a brand-new nightly flag that `wrapc` hasn't explicitly modeled yet, it won't crash. It safely buckets it into `info.unknown` and perfectly preserves it during `to_args()` reconstruction.

## Testing Your Wrapper

To test your wrapper locally without publishing it, use the `RUSTC_WRAPPER` environment variable:

```bash
# Build your wrapper
cargo build --release

# Point Cargo to your wrapper
export RUSTC_WRAPPER="$(pwd)/target/release/my_wrapper"

# Run a standard cargo command
cargo build
```

## Contributing

Contributions, issues, and feature requests are welcome!  
If you notice a missing `rustc` flag that should be explicitly modeled rather than falling back to `unknown`, please open an Issue or Merge Request on [GitLab](https://gitlab.com/viraven/wrapc).

## License

This project is licensed under the [MIT License](https://opensource.org/licenses/MIT).

---

*Made with ❤️ for Rust community.*
