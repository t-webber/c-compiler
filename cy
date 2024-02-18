#!/usr/bin/bash
clear
export CARGO_TARGET_DIR=build/wsl
cargo clean
cargo clippy --all -- -W clippy::all -W clippy::pedantic -W clippy::nursery -W clippy::cargo -W clippy::correctness -W clippy::complexity -W clippy::perf -W clippy::style -W clippy::suspicious # -D warnings
cargo run -- $1 $2 $3 $4 $5 $6 $7 $8 $9
