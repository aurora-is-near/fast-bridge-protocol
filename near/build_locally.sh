#!/bin/bash
set -e

mkdir -p res
cd contracts
rustup target add wasm32-unknown-unknown
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
cp ./target/wasm32-unknown-unknown/release/*.wasm ../res/
