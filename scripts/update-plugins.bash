#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && cd ../ && pwd )"
PLUGINS_DIR="$ROOT_DIR/plugins"
ASSETS_DIR="$ROOT_DIR/assets"

main() {
    cd "$PLUGINS_DIR"
    cargo build --release --target=wasm32-wasi

    mkdir -p "$ASSETS_DIR"
    cd "$ASSETS_DIR"
    rm -f ./*.wasm
    cp "$PLUGINS_DIR"/target/wasm32-wasi/release/*.wasm "$ASSETS_DIR"
}

main
