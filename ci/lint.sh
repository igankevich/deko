#!/bin/sh

. ./ci/preamble.sh

git config --global --add safe.directory "$PWD"
cargo clippy --quiet --workspace --all-targets -- --deny warnings
cargo deny check
