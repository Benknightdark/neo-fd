#!/bin/sh

set -eu

HARNESS_DIR=$(CDPATH= cd "$(dirname "$0")" && pwd)
PROJECT_ROOT=$(CDPATH= cd "$HARNESS_DIR/.." && pwd)

cd "$PROJECT_ROOT"

usage() {
  printf '%s\n' '用法：sh .neo_harness/verify.sh [all|frontend|rust]' >&2
  exit 2
}

run_frontend() {
  npm --prefix neo-fd-desktop run lint
  npm --prefix neo-fd-desktop run build
  npm --prefix neo-fd-desktop run test
}

run_rust() {
  cd neo-fd-desktop/src-tauri
  cargo fmt --check
  cargo clippy -- -D warnings
  cargo test
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
  *)
    usage
    ;;
esac
