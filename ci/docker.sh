#!/bin/sh
. ./ci/preamble.sh
image=ghcr.io/igankevich/deko-ci:latest
docker build --tag $image - <ci/Dockerfile
docker push $image
