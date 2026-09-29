#!/usr/bin/env bash
# Local equivalent of CI.
#
#   scripts/check.sh          core only (fmt, clippy, test)
#   scripts/check.sh --gui    core + GUI crate (fmt, clippy, build)
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

echo "== core: fmt =="
cargo fmt --all -- --check
echo "== core: clippy =="
cargo clippy --all-targets -- -D warnings
echo "== core: test =="
cargo test --all-targets

if [[ "${1:-}" == "--gui" ]]; then
  echo "== gui: fmt =="
  cargo fmt --manifest-path gui/Cargo.toml -- --check
  echo "== gui: clippy =="
  cargo clippy --manifest-path gui/Cargo.toml --all-targets -- -D warnings
  echo "== gui: build =="
  CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target-gui}" \
    cargo build --manifest-path gui/Cargo.toml
fi
