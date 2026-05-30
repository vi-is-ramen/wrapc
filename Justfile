# === Base checks (used everywhere) ===

_check:
    @cargo fmt --all -- --check
    @cargo clippy --workspace --all-targets --all-features -- -Dwarnings
    @cargo test --workspace

_check-wrapc:
    @cd wrapc && cargo fmt -- --check
    @cd wrapc && cargo clippy --all-targets --all-features -- -Dwarnings
    @cd wrapc && cargo test

# === Local development ===

default:
    @just --list | grep -v default || true

fmt:
    @cargo fmt --all

fmt-fix:
    @cargo fmt --all

clippy: _check
clippy-fix:
    @cargo clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged

test:
    @cargo test --workspace

check: _check

# wrapc-specific shortcuts
wrapc-fmt:
    @cd wrapc && cargo fmt

wrapc-clippy: _check-wrapc
wrapc-test:
    @cd wrapc && cargo test

wrapc-check: _check-wrapc

# === Pre-commit hook (lightweight, fast feedback) ===

pre:
    @echo "> Running pre-commit checks..."
    @just fmt
    @just _check-wrapc
    @echo "+ Pre-commit passed"

# === CI (full workspace validation) ===

ci:
    @echo "> Running CI checks..."
    @just _check
    @echo "+ CI passed"

# === Publish (tag-based, crates.io) ===

pub:
    @cd wrapc && cargo publish --token "${CRATES_IO_TOKEN}"
    @echo "+ Published $version"

# === Helpers ===

clean:
    @cargo clean

doc:
    @cargo doc --workspace --no-deps --open

doc-wrapc:
    @cd wrapc && cargo doc --no-deps --open
