#!/usr/bin/env bash

set -xe

cargo clippy -p stellation-core -- -D warnings

cargo clippy -p stellation-bridge -- -D warnings
cargo clippy -p stellation-bridge --features=resolvable -- -D warnings

cargo clippy -p stellation-backend -- -D warnings
cargo clippy -p stellation-backend --features=warp-filter -- -D warnings
cargo clippy -p stellation-backend --features=tower-service -- -D warnings
cargo clippy -p stellation-backend --features=hyper-server -- -D warnings

cargo clippy -p stellation-backend-cli -- -D warnings

cargo clippy -p stellation-frontend -- -D warnings

cargo clippy -p stctl -- -D warnings

cargo clippy -p stellation -- -D warnings

cargo clippy -p example-fullstack-client -- -D warnings
cargo clippy -p example-fullstack-server -- -D warnings
