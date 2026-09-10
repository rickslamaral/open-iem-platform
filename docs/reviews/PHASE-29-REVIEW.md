# Phase 29 Review — Failed WebSocket Authentication Limiting

## Status

Implemented locally. Remote CI unavailable; GitHub Actions run `34509968438` failed before steps with `runner_id=0`.

## Scope

`/ws/v1` failed authenticated upgrade attempts only. Five failures per peer IP per 60-second window trigger HTTP 429 with `Retry-After: 5`. Limiter retains at most 4,096 IP entries and evicts least-recently-observed state at capacity.

Successful authenticated WebSocket requests do not consume failure budget. Peer identity comes only from request extension `ConnectInfo<SocketAddr>`; forwarded headers are ignored.

## Validation

- Deterministic unit tests cover threshold, window reset and bounded state.
- `cargo fmt --manifest-path server/Cargo.toml --all`: PASS.
- `cargo test -p api-server --all-targets --no-fail-fast`: PASS — 38 unit tests and 52 integration tests.
- `cargo clippy -p api-server --all-targets -- -D warnings`: PASS.
- `scripts/validate-docs.sh` and `git diff --check`: PASS.
- Remote CI remains unavailable; run `34509968438` failed before steps with `runner_id=0`.

## Limitations

Limiter is process-local. Distributed deployments need shared coordination. JWT revocation after issuance remains separate policy.
