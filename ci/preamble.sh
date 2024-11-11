#!/bin/sh

sh_end() {
    rm -rf "$workdir"
}

sh_begin() {
    trap sh_end EXIT
    workdir="$(mktemp -d)"
    PS4='$0:$LINE 🦎 ' set -ex
}

sh_begin
