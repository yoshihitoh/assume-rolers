default: plugins build

build:
    cargo build --release

plugins:
    bash ./scripts/update-plugins.bash

changelog:
    bash ./scripts/update-changelog.bash
