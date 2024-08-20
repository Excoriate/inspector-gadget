default:
    @just --list

build:
    cargo build --release

test:
    cargo test

doc:
    cargo doc --no-deps --open

run *args:
    cargo run -- {{args}}

lint:
    cargo clippy -- -D warnings

format:
    cargo fmt

check: lint test