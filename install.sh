#!/bin/bash
set -e

cargo build --release

if [[ "$OSTYPE" == "darwin"* ]]; then
    cp target/release/cargo-save ~/.cargo/bin/cargo-save
    echo "Installed cargo-save to ~/.cargo/bin/cargo-save"
else
    sudo cp target/release/cargo-save /usr/local/bin/cargo-save
    echo "Installed cargo-save to /usr/local/bin/cargo-save"
fi
