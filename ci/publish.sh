#!/bin/sh

. ./ci/preamble.sh

cargo_publish() {
    cargo publish --quiet
}

if test "$GITHUB_ACTIONS" = "true" && test "$GITHUB_REF_TYPE" != "tag"; then
    exit 0
fi
cargo_publish
