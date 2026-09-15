#!/bin/sh

set -eu

HARNESS_DIR=$(CDPATH= cd "$(dirname "$0")" && pwd)
PROJECT_ROOT=$(CDPATH= cd "$HARNESS_DIR/.." && pwd)

cd "$PROJECT_ROOT"

usage() {
  printf '%s\n' '用法：sh .neo_harness/verify.sh [all|frontend|rust|lint]' >&2
  exit 2
}

run_frontend_lint() {
  npm --prefix neo-fd-desktop run lint
  npm --prefix tests/frontend run lint
}

run_rust_lint() {
  (
    cd neo-fd-desktop/src-tauri
    cargo fmt --check
    cargo clippy --quiet --all-targets -- -D warnings
  )
  cargo fmt --manifest-path tests/backend/Cargo.toml -- --check
  cargo clippy --quiet --manifest-path tests/backend/Cargo.toml --all-targets -- -D warnings
}

run_frontend() {
  npm --prefix neo-fd-desktop run lint
  npm --prefix neo-fd-desktop run build
  npm --prefix tests/frontend run lint
  npm --prefix tests/frontend run typecheck
  npm --prefix tests/frontend run test:unit
  npm --prefix tests/frontend run test:e2e
}

run_rust() {
  (
    cd neo-fd-desktop/src-tauri
    cargo fmt --check
    cargo clippy --quiet --all-targets -- -D warnings
    cargo test --quiet
  )
  cargo fmt --manifest-path tests/backend/Cargo.toml -- --check
  cargo clippy --quiet --manifest-path tests/backend/Cargo.toml --all-targets -- -D warnings
  cargo test --quiet --manifest-path tests/backend/Cargo.toml
}

run_lint() {
  run_frontend_lint
  run_rust_lint
}

case "${1:-all}" in
  all)
    run_frontend
    run_rust
    ;;
  frontend)
    run_frontend
    ;;
  rust)
    run_rust
    ;;
  lint)
    run_lint
    ;;
  *)
    usage
    ;;
esac
