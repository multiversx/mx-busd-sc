#!/bin/sh

# script provided for convenience, to build and extract wasm output to root

RUSTFLAGS='-C link-arg=-s' \
cargo build --bin busd --target=wasm32-unknown-unknown --release
mv target/wasm32-unknown-unknown/release/busd.wasm busd.wasm
wasm-snip busd.wasm -o busd.wasm --snip-rust-fmt-code #--snip-rust-panicking-code
# wasm-gc busd.wasm
# twiggy top -n 20 busd.wasm
# twiggy top -n 300 busd.wasm > twiggy-snip.txt
