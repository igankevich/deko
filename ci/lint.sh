#!/bin/sh

. ./ci/preamble.sh

git config --global --add safe.directory "$PWD"
pre-commit run --all-files --show-diff-on-failure
cargo deny check
