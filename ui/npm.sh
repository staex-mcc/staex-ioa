#!/bin/sh
cleanup() {
    rm -f "$globalconfig"
}

set -e
trap cleanup EXIT
globalconfig=$(mktemp)

exec npm \
    --userconfig=/dev/null \
    --globalconfig="$globalconfig" \
    --no-update-notifier \
    --no-fund \
    --init-module=/dev/null \
    --cache=.npm \
    "$@"
