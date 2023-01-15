#!/usr/bin/env bash

set -xe

cargo clippy -p stackable-core -- -D warnings

cargo clippy -p stackable-bridge -- -D warnings
cargo clippy -p stackable-bridge --features=resolvable -- -D warnings

cargo clippy -p stackable-backend -- -D warnings
cargo clippy -p stackable-backend --features=warp-filter -- -D warnings
cargo clippy -p stackable-backend --features=tower-service -- -D warnings
cargo clippy -p stackable-backend --features=hyper-server -- -D warnings
cargo clippy -p stackable-backend --features=cli -- -D warnings

cargo clippy -p stackable-frontend -- -D warnings

cargo clippy -p stackctl -- -D warnings

cargo clippy -p example-fullstack-client -- -D warnings
cargo clippy -p example-fullstack-server -- -D warnings
