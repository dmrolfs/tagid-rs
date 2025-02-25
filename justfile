# Justfile for the `tagid` crate

# Set default shell behavior
set shell := ["bash", "-ueo", "pipefail", "-c"]

# Format the code
fmt:
    cargo fmt --all --check

# Run Clippy with all features and deny warnings
clippy:
    cargo clippy --all-features --all-targets

# Validate documentation links
doc-check:
    cargo deadlinks --dir target/doc

# Build with all features and for all targets
build:
    cargo build --all-features

# Run tests with all features
test:
    cargo test --all-features --no-fail-fast

# Run all checks (formatting, Clippy, doc links, build, and test)
check: fmt clippy doc-check build test

# Publish to crates.io (dry run by default)
publish_dryrun:
    cargo publish --dry-run

# Publish to crates.io (actual publish)
publish:
    cargo publish
