set positional-arguments

default:
  @just --list

ci:
  @just fmt-check
  @just lint
  @just test

test *args='':
  cargo test --workspace {{args}}

lint *args='':
  cargo clippy --workspace --all-targets --all-features -- -D warnings {{args}}

fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

build *args='':
  cargo build --workspace {{args}}

run *args='':
  cargo run --package veto -- {{args}}

docker-build:
  docker build -t veto-builder:latest --target veto-builder .
