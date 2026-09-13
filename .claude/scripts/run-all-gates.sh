#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$ROOT"

cargo fmt --manifest-path server/Cargo.toml --all -- --check
cargo clippy --manifest-path server/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path server/Cargo.toml --workspace

for app in web/musician web/engineer; do
  npm --prefix "$app" run typecheck
  npm --prefix "$app" test -- --run
  npm --prefix "$app" run build
done

python3 -m pytest -q tests
bash scripts/validate-skills.sh
printf 'ALL LOCAL GATES PASSED\n'
