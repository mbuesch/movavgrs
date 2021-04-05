#!/bin/sh
set -e

srcdir="$(dirname "$0")"
[ "$(echo "$srcdir" | cut -c1)" = '/' ] || srcdir="$PWD/$srcdir"

srcdir="$srcdir/.."


cd "$srcdir"
echo Running std tests...
cargo test
echo Running no_std tests...
cargo test --lib --no-default-features
