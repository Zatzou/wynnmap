#!/bin/sh

version="$(
    cargo metadata --format-version 1 --no-deps |
    jq -r '
        .packages[]
        | select(.name == "wynnmap")
        | .version
    '
)"

printf '"%s"' "$version" > "$TRUNK_STAGING_DIR/version.json"