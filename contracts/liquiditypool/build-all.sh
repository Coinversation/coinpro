#!/usr/bin/env bash

set -eu

cargo +nightly contract build --manifest-path math/Cargo.toml
cargo +nightly contract build --manifest-path base/Cargo.toml
cargo +nightly contract build