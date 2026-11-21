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
    #!/usr/bin/env bash
    set -uo pipefail
    cargo doc --all-features --no-deps
    out=$(cargo deadlinks --dir target/doc 2>&1)
    # shortcut: tolerates tracing's inlined `dispatcher` link and rustdoc's help/settings index.html (neither is tagid's); drop when rustdoc stops emitting them.
    bad=$(grep "Linked file" <<<"$out" | grep -v -e "/dispatcher does not exist" -e "path index.html does not exist" || true)
    if [ -n "$bad" ]; then echo "$out"; exit 1; fi

# Build with all features and for all targets
build:
    cargo build --all-features

# Run tests with all features
test:
    cargo test --all-features --no-fail-fast

# Check optional validation dependencies without starting services
check-validator-features:
    python3 scripts/check_validator_features.py

# Run all checks (formatting, Clippy, doc links, build, and test)
check: fmt check-validator-features clippy doc-check build test

# Publish to crates.io (dry run by default)
publish_dryrun:
    cargo publish --dry-run

# Publish to crates.io (actual publish)
publish:
    cargo publish
