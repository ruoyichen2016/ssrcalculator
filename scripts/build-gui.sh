#!/usr/bin/env bash
# Build the optional egui GUI into its own target directory.
#
# eframe pulls in ~130 crates; keeping them in a dedicated target dir means the
# core `target/` never has to hold (or rebuild) the GUI dependency tree, and the
# GUI cache can be deleted independently:
#
#   rm -rf target-gui
#
# Usage: scripts/build-gui.sh [cargo build args...]
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-stable}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target-gui}"
exec cargo build --manifest-path gui/Cargo.toml "$@"
