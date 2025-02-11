#!/bin/sh
set -e

srcdir="$(realpath "$0" | xargs dirname)"
srcdir="$srcdir/.."

cd "$srcdir"
echo Running clippy...
cargo clippy -- --deny warnings
cargo clippy --tests -- --deny warnings
echo Running std tests...
cargo test
echo Running std/fastfloat tests...
cargo test --features fastfloat
echo Running no_std tests...
cargo test --tests --no-default-features
echo Running no_std/fastfloat tests...
cargo test --tests --no-default-features --features fastfloat
