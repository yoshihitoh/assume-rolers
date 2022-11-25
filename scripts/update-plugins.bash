#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && cd ../ && pwd )"
PLUGINS_DIR="$ROOT_DIR/plugins"

main() {
    cd $PLUGINS_DIR
    cargo build --release --target=wasm32-wasi

    rm -f *.wasm
    cp target/wasm32-wasi/release/*.wasm .
}

main
