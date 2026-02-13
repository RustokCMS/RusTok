.PHONY: docs-sync-loco docs-check-loco docs-sync-server-libs docs-check-server-libs

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
