#!/bin/sh

. ./ci/preamble.sh

git config --global --add safe.directory "$PWD"
cargo clippy --workspace --all-targets --all-features -- --deny warnings
cargo deny check
