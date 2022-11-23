#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && cd ../ && pwd )"

main() {
    local prev_tag
    prev_tag="$(git tag --sort=-version:refname | head -n 2 | tail -n 1)"

    local prev_hash
    prev_hash="$(git rev-parse "${prev_tag}")"

    local latest_tag
    latest_tag="$(git tag --sort=-version:refname | head -n 1)"

    local latest_hash
    latest_hash="$(git rev-parse "${latest_tag}")"

    git cliff "$prev_hash..$latest_hash" -p "$ROOT_DIR/CHANGELOG.md" -t "$latest_tag"
}

main
