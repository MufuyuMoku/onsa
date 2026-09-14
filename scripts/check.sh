#!/usr/bin/env sh
# Runs the same checks as CI (.github/workflows/ci.yml) on this machine.
# Usage: scripts/check.sh   (from anywhere inside the repository)
set -eu

cd "$(dirname "$0")/.."

step() {
	printf '\n==> %s\n' "$1"
}

if [ ! -d ui/node_modules ]; then
	step "npm ci (ui)"
	(cd ui && npm ci)
fi

step "svelte-check (ui)"
(cd ui && npm run check)

step "interface tests (ui)"
(cd ui && npm test)

# The Tauri context embeds the built interface, so the Rust checks need it.
step "build interface (ui)"
(cd ui && npm run build)

step "cargo fmt --check"
cargo fmt --all -- --check

step "cargo clippy"
cargo clippy --workspace --all-targets -- -D warnings

step "cargo test"
cargo test --workspace

printf '\nAll checks passed.\n'
