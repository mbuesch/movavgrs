#!/bin/sh
set -e

srcdir="$(dirname "$0")"
[ "$(echo "$srcdir" | cut -c1)" = '/' ] || srcdir="$PWD/$srcdir"

srcdir="$srcdir/.."


cd "$srcdir"
echo Running std tests...
cargo test
echo Running std/fastfloat tests...
cargo test --features fastfloat
echo Running no_std tests...
cargo test --tests --no-default-features
echo Running no_std/fastfloat tests...
cargo test --tests --no-default-features --features fastfloat
