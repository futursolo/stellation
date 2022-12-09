#!/usr/bin/env bash

set -xe

cargo clippy -p stackable-core

cargo clippy -p stackable-bridge
cargo clippy -p stackable-bridge --features=resolvable

cargo clippy -p stackable-backend
cargo clippy -p stackable-backend --features=warp-filter
cargo clippy -p stackable-backend --features=tower-service
cargo clippy -p stackable-backend --features=hyper-server
cargo clippy -p stackable-backend --features=cli

cargo clippy -p stackable-frontend

cargo clippy -p stackctl

cargo clippy -p example-fullstack-client
cargo clippy -p example-fullstack-server
