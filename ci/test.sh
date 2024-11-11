#!/bin/sh

. ./ci/preamble.sh

clean() {
    find target -type f -name '*.profraw' -delete || true
    find target -type f -name '*.gcda' -delete || true
}

test_all() {
    cargo test --workspace --quiet --no-run "$@"
    cargo test --workspace --no-fail-fast "$@" -- --nocapture
}

test_all_nightly() {
    cargo +nightly test --workspace --quiet --no-run "$@"
    cargo +nightly test --workspace --no-fail-fast "$@" -- --nocapture
}

test_coverage_preamble() {
    export CARGO_INCREMENTAL=0
    export RUSTFLAGS='-Zprofile -Ccodegen-units=1 -Cllvm-args=--inline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort -Zpanic_abort_tests'
    export LLVM_PROFILE_FILE="any-decoder-%p-%m.profraw"
}

test_coverage_postamble() {
    grcov \
        . \
        --binary-path target/debug/ \
        -s . \
        -t lcov \
        --branch --ignore-not-existing --ignore "*index.crates.io*" \
        --excl-start '.*cfg\(.*test.*' \
        --excl-br-start '.*cfg\(.*test.*' \
        --excl-br-line '.*cfg.*test.*' \
        -o target/debug/lcov.info
    lcov --summary target/debug/lcov.info
    genhtml -o target/debug/coverage/ \
        --branch-coverage \
        --highlight \
        --ignore-errors source \
        --legend \
        target/debug/lcov.info
}

test_miri() {
    cargo +nightly miri setup --quiet
    do_test_miri --quiet --no-run "$@"
    do_test_miri "$@"
}

do_test_miri() {
    env MIRIFLAGS=-Zmiri-disable-isolation cargo +nightly miri test "$@"
}

clean
#test_coverage_preamble
test_all --no-default-features --features 'bzip2 flate2 xz zstd'
test_all_nightly --no-default-features --all-features
#test_coverage_postamble
# only `flate2` crate is pure Rust
test_miri --no-default-features --features flate2
