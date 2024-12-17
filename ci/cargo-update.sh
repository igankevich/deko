#!/bin/sh

cargo_update() {
    cargo update --color=never --verbose "$@"
}

check_patch() {
    sha256sum Cargo.toml Cargo.lock >hashes
    printf "\n## Patches\n\n" >>message.txt
    printf '```'"\n" >>message.txt
    cargo_update 2>&1 | tee -a message.txt
    printf '```'"\n" >>message.txt
    if ! sha256sum -c hashes >/dev/null 2>&1; then
        patch=1
    fi
}

check_minor() {
    sha256sum Cargo.toml Cargo.lock >hashes
    printf "\n## Minor\n\n" >>message.txt
    printf '```'"\n" >>message.txt
    cargo_update -Z unstable-options --breaking 2>&1 | tee -a message.txt
    printf '```'"\n" >>message.txt
    if ! sha256sum -c hashes >/dev/null 2>&1; then
        minor=1
    fi
}

bump_version() {
    cargo install --quiet cargo-bump
    if test "$patch" = 1; then
        cargo bump patch
    fi
    if test "$minor" = 1; then
        cargo bump minor
    fi
    # include the new version of the package in Cargo.lock
    cargo_update >/dev/null 2>&1
}

create_pull_request() {
    git config --global user.name "Cargo Updater"
    git config --global user.email "igankevich@users.noreply.github.com"
    hash="$(cat Cargo.toml Cargo.lock | sha256sum | awk -F' ' '{ print $1 }')"
    branch=cargo-update/"$hash"
    git checkout -b "$branch"
    git commit --all --file message.txt
    git push origin "$branch"
    gh pr create -B master -H "$branch" --title "Cargo update" --body "$(cat message.txt)"
}

set -e
patch=0
minor=0
printf "# Cargo update\n" >message.txt
check_patch
check_minor
if test "$minor" = 0 && test "$patch" = 0; then
    printf "No updates\n" >&2
    exit 0
fi
bump_version
create_pull_request
