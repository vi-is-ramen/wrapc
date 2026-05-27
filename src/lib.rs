//! # wrapc
//!
//! A zero-fuss parser for rustc arguments, designed for RUSTC_WRAPPER tools.
//! Provides comprehensive and type-safe coverage of `rustc`'s CLI surface area.
//! 
//! ## Usage Guide: Building Bulletproof `RUSTC_WRAPPER` Tools
//! 
//! ### 1. The Core Concept
//! 
//! When Cargo invokes a `RUSTC_WRAPPER`, it doesn't just pass rustc arguments.
//! It uses a specific protocol: `<wrapper_binary> - <actual_rustc_path> <rustc_args...>`
//! 
//! `wrapc` automatically detects this protocol, extracts the real `rustc` path, and
//! parses the remaining arguments into a strongly-typed `Info` struct.
//! 
//! > **Experimental:**
//! > If invoked directly (without Cargo's `-` separator), it gracefully falls back to standard parsing.
//! 
//! ### 2. Basic Setup & Execution
//! 
//! Add wrapc to your wrapper tool's dependencies:
//! ```shell
//! cargo add wrapc
//! ```
//! 
//! Here is the canonical skeleton for a `RUSTC_WRAPPER` executable:
//! 
//! ```rust
//! use std::process::Command;
//! use wrapc::fetch;
//! 
//! fn main() {
//!     // `fetch()` reads `std::env::args()` and parses them.
//!     let info = fetch().expect("Failed to parse rustc arguments");
//!     
//!     // 1. Handle passthrough commands (help/version)
//!     if info.help || info.version {
//!         // Just execute rustc and exit
//!     }
//! 
//!     // 2. Inspect the compilation context
//!     println!("Wrapper: Compiling crate {:?}", info.crate_name);
//!     println!("Wrapper: Target is {:?}", info.target);
//! 
//!     // 3. Determine the real rustc path
//!     // If invoked via Cargo, `info.rustc` contains the path. 
//!     // Otherwise, fallback to "rustc" in PATH.
//!     let rustc_path = info.rustc.unwrap_or_else(|| "rustc".to_string());
//! 
//!     // 4. Reconstruct the arguments perfectly
//!     let args = info.to_args();
//! 
//!     // 5. Execute the actual compiler
//!     let status = Command::new(rustc_path)
//!         .args(&args)
//!         .status()
//!         .expect("Failed to spawn rustc");
//! 
//!     std::process::exit(status.code().unwrap_or(1));
//! }
//! ```
//! 
//! ### 3. Inspecting Parsed Data
//! 
//! `wrapc` categorizes `rustc` flags into logical, type-safe collections.
//! 
//! #### Configurations and Crate Metadata
//! 
//! ```rust
//! // --cfg and --check-cfg
//! for cfg in &info.configs {
//!     if cfg == "test" {
//!         println!("Compiling in test mode!");
//!     }
//! }
//! 
//! // --crate-type (e.g., bin, rlib, dylib)
//! if info.crate_types.contains(&"dylib".to_string()) {
//!     println!("Building a dynamic library.");
//! }
//! ```
//! 
//! #### Complex Linking (`-l` and `-L`)
//! 
//! `rustc`'s linking flags are highly structured. `wrapc` parses modifiers, kinds, and renames automatically.
//! 
//! ```rust
//! use wrapc::{LibrarySearchPathKind, LSPK, LinkLibKind};
//! 
//! // Inspecting -L (Library Search Paths)
//! for lib_path in &info.libpaths {
//!     match lib_path.kind {
//!         LibrarySearchPathKind::Native => println!("Native search path: {:?}", lib_path.path),
//!         // LSPK is an alias to LibrarySearchPathKind
//!         LSPK::Framework => println!("macOS Framework path: {:?}", lib_path.path),
//!         _ => println!("Generic search path: {:?}", lib_path.path),
//!     }
//! }
//! 
//! // Inspecting -l (Link Libraries)
//! // Example parsed: `-lstatic:+bundle,+whole-archive=mylib:renamed_lib`
//! for link in &info.links {
//!     println!("Linking: {}", link.name);
//!     if let Some(kind) = &link.kind {
//!         println!("  Kind: {:?}", kind); // e.g., LinkLibKind::Static
//!     }
//!     if !link.modifiers.is_empty() {
//!         println!("  Modifiers: {:?}", link.modifiers); // e.g., ["+bundle", "+whole-archive"]
//!     }
//!     if let Some(rename) = &link.rename {
//!         println!("  Renamed to: {}", rename);
//!     }
//! }
//! ```
//! 
//! ### 4. Mutating Arguments (The Wrapper Superpower)
//! 
//! The most common use case for a wrapper is to inject or modify flags before passing
//! them to `rustc`. Because [`Info`] is just a standard Rust struct, you can mutate it freely.
//! [`Info::to_args()`] will flawlessly reconstruct the CLI string.
//! 
//! ```rust
//! let mut info = fetch().unwrap();
//! 
//! // Inject a custom linker argument
//! info.codegen_opts.push("link-arg=-Wl,-rpath,/opt/my-tool/lib".to_string());
//! 
//! // Force a specific optimization level for a specific crate
//! if info.crate_name.as_deref() == Some("legacy_crate") {
//!     // Remove existing opt-levels
//!     info.codegen_opts.retain(|opt| !opt.starts_with("opt-level="));
//!     // Inject our own
//!     info.codegen_opts.push("opt-level=1".to_string());
//! }
//! 
//! // Add a custom configuration flag
//! info.configs.push("my_wrapper_active".to_string());
//! 
//! // Pass to rustc
//! let args = info.to_args();
//! ```
//! 
//! ### 5. Handling Edge Cases & "Unknown" Flags
//! 
//! `rustc` has hundreds of flags, including unstable nightly flags (`-Z`) and internal
//! compiler queries. `wrapc` captures everything it explicitly knows, and safely
//! buckets the rest into info.unknown.
//! 
//! #### The `--` Separator
//! 
//! In `rustc`, anything after `--` is usually ignored or passed to specific internal tools.
//! `wrapc` stops parsing at `--` and dumps the rest into [`Info::unknown`] to prevent
//! accidental misinterpretation.
//! 
//! ```rust
//! // If rustc is called with: `rustc --crate-name foo -- --some-internal-flag`
//! assert_eq!(info.crate_name, Some("foo".to_string()));
//! assert_eq!(info.unknown, vec!["--", "--some-internal-flag"]);
//! ```
//! 
//! #### Unrecognized / Nightly Flags
//! 
//! If a user passes a brand new nightly flag that `wrapc` hasn't explicitly modeled
//! yet, it won't crash. It goes to [`Info::unknown`] and is perfectly
//! preserved by [`Info::to_args()`].
//! 
//! ```rust
//! for unknown_arg in &info.unknown {
//!     // Safe to pass through to rustc
//!     println!("Passing through: {}", unknown_arg);
//! }
//! ```
//! 
//! #### The `stdin` Edge Case (`-`)
//! 
//! Tools like `cargo` or IDEs sometimes pass `-` as the input file
//! to tell `rustc` to read source code from stdin. 
//! Because Cargo's wrapper protocol also uses `-` as a separator
//! (`<wrapper> - <rustc>`), naive parsers break here.
//! `wrapc` handles this contextually:
//! 
//! ```rust
//! // Invocation: `my_wrapper - rustc -`
//! let info = fetch().unwrap();
//! 
//! assert_eq!(info.rustc, Some("rustc".to_string()));
//! assert_eq!(info.inputs, vec![std::path::PathBuf::from("-")]); // Correctly identified as input!
//! ```
//! 
//! ### 6. Testing Your Wrapper
//! 
//! To test your wrapper locally without publishing it, use the `RUSTC_WRAPPER` environment variable:
//! 
//! ```bash
//! # Build your wrapper
//! cargo build --release
//! 
//! # Point Cargo to your wrapper
//! export RUSTC_WRAPPER="/path/to/your/project/target/release/my_wrapper"
//! 
//! # Run a standard cargo command
//! cargo build
//! ```
//! 
//! Your wrapper will intercept every `rustc` invocation, allowing you to log,
//! mutate, or cache the compilation process using the type-safe
//! data provided by `wrapc`.
//! 

mod error;
pub use error::*;

mod info;
pub use info::*;

mod parser;
pub use parser::*;
