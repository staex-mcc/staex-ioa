#!/bin/bash

exec docker run --rm \
  --user "$(id -u):$(id -g)" \
  -p 5173:5173 \
  -it \
  -v "${PWD}":/staex-ioa \
  --entrypoint="" \
  --workdir /staex-ioa \
  oven/bun "$@"
