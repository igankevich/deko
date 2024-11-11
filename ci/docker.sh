#!/bin/sh
. ./ci/preamble.sh
image=ghcr.io/igankevich/any-decoder-ci:latest
docker build --tag $image - <ci/Dockerfile
docker push $image
