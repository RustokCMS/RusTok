.PHONY: docs-sync-loco docs-check-loco docs-sync-server-libs docs-check-server-libs
.PHONY: test test-unit test-integration ci-check

# Refresh metadata for the local Loco upstream docs snapshot.
docs-sync-loco:
    python3 scripts/loco_upstream_snapshot.py sync

# Validate that the upstream snapshot metadata exists and is fresh enough.
docs-check-loco:
    python3 scripts/loco_upstream_snapshot.py check

# Download fresh upstream docs snapshots for core server libraries.
docs-sync-server-libs:
    python3 scripts/server_library_docs_sync.py sync

# Validate local upstream docs snapshot for core server libraries.
docs-check-server-libs:
    python3 scripts/server_library_docs_sync.py check

# Run all tests (unit + integration)
test:
    cargo test --workspace --all-features

# Run only unit tests
test-unit:
    cargo test --workspace --lib

# Run integration tests (requires test server)
test-integration:
    cargo test --package rustok-server --test '*' -- --ignored

# Run all CI checks locally
ci-check: fmt-check clippy test

# Format check
fmt-check:
    cargo fmt --all -- --check

# Clippy check
clippy:
    cargo clippy --workspace --all-targets -- -D warnings
