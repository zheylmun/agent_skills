#!/bin/sh
# Re-runs the sampling tests whenever a source file changes.
# Needs cargo-watch: cargo install cargo-watch
set -eu
cd "$(dirname "$0")/.."
exec cargo watch -w src -w tests -x 'test --test sampling'
