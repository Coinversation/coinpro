#!/usr/bin/env bash

set -eu

cargo +nightly contract build --manifest-path math/Cargo.toml
cargo +nightly contract build --manifest-path base/Cargo.toml
cargo +nightly contract build --manifest-path token/Cargo.toml
cargo +nightly contract build --manifest-path pool/Cargo.toml
cargo +nightly contract build
