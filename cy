#!/bin/bash
clear
cargo clean
cargo clippy -- -W clippy::pedantic
cargo run $1
