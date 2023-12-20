#!/usr/bin/env bash

echo "Tests:"
cargo test --quiet

echo "Examples:"
cargo test --examples --quiet