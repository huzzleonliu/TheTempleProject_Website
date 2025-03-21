#!/bin/bash

set -euo pipefail

echo "Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

echo "Adding WASM target for Rust..."
rustup target add wasm32-unknown-unknown

echo "Installing Trunk..."
cargo install trunk

echo "Setup complete."
