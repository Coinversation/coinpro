#!/usr/bin/env bash

set -eu

cargo +nightly contract build --manifest-path pool/Cargo.toml
cargo +nightly contract build