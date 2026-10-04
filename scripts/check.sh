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
  # Set once for the whole GUI section: every `--manifest-path gui/...` call
  # made from the repo root bypasses `gui/.cargo/config.toml` (Cargo discovers
  # config from the cwd, not the manifest), so without this the clippy step
  # alone would create a second ~950 MB `gui/target/` cache.
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target-gui}"
  echo "== gui: fmt =="
  cargo fmt --manifest-path gui/Cargo.toml -- --check
  echo "== gui: clippy =="
  cargo clippy --manifest-path gui/Cargo.toml --all-targets -- -D warnings
  echo "== gui: build =="
  cargo build --manifest-path gui/Cargo.toml
fi

# Tripwire: `/gui/target` is gitignored, so a stray copy of the eframe cache is
# invisible in `git status` and can quietly grow to ~1 GB (that is exactly how
# it went unnoticed before).  Warn instead of relying on someone running `du`.
if [[ -d "$root/gui/target" ]]; then
  echo
  echo "warning: $root/gui/target exists — that is a duplicate GUI cache." >&2
  echo "         GUI builds must go to target-gui/ (scripts/build-gui.sh)." >&2
  echo "         Remove it with: rm -rf gui/target" >&2
fi
