#!/usr/bin/env bash
# Script for running check on your rust projects.
set -e
set -x
set -u

declare -A FEATURES
FEATURES=(
)

run_doc() {
    rustup component add rust-docs
    cargo doc --no-default-features
    cargo doc
    for FEATURE in "${FEATURES[@]}"
    do
        cargo doc --no-default-features --features "$FEATURE"
    done
    cargo doc --no-default-features --features="document-features,serde,properties_as_hashmap,properties_as_vector"
}

run_fmt() {
    rustup component add rustfmt
    cargo fmt --check
}

run_clippy() {
    rustup component add clippy-preview
    cargo clippy --no-default-features  -- -D warnings
    cargo clippy -- -D warnings
    for FEATURE in "${FEATURES[@]}"
    do
        cargo clippy --no-default-features --features "$FEATURE" -- -D warnings
    done
}

run_check() {
    cargo check --no-default-features
    cargo check
    for FEATURE in "${FEATURES[@]}"
    do
        cargo check --no-default-features --features "$FEATURE"
    done
}

run_test() {
    cargo test --no-default-features
    cargo test
    for FEATURE in "${FEATURES[@]}"
    do
        cargo test --no-default-features --features "$FEATURE"
    done
}

run_build() {
    cargo build  --no-default-features
    cargo build
    for FEATURE in "${FEATURES[@]}"
    do
        cargo build  --no-default-features --features "$FEATURE"
    done
}

run_build_release() {
    cargo build --release --no-default-features
    cargo build --release
    for FEATURE in "${FEATURES[@]}"
    do
        cargo build --release --no-default-features --features "$FEATURE"
    done
}

case $1 in
    doc)
        run_doc
        ;;
    fmt)
        run_fmt
        ;;
    check)
        run_check
        ;;
    clippy)
        run_clippy
        ;;
    test)
        run_test
        ;;
    build)
        run_build
        ;;
    build-release)
        run_build_release
        ;;
esac
