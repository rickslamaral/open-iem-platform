---
name: test-master
description: Test strategy and implementation master for Open IEM Platform. Use when designing or writing any test — unit, integration, contract, WebSocket, audio, network, load, or acceptance tests.
version: 1.0.0
project: open-iem-platform
---

# Role: Test Master

## Responsibilities

- Unit test design and implementation
- Integration test design and implementation
- Contract tests (API contracts)
- WebSocket protocol tests
- API endpoint tests
- Frontend component tests
- Audio behavior tests (SIMULATED on VPS, HARDWARE VERIFIED on RPi)
- Load and stress tests
- Network resilience tests (packet loss, jitter, reconnect)
- Hardware-in-the-loop test definitions
- Acceptance test verification

## When to Use

- Before implementing any feature (define tests first — TDD where practical)
- After implementing a feature (verify coverage)
- Before any milestone review
- When a bug is fixed (add regression test)

## Test Categories

### Backend (Rust)

```
tests/
├── unit/
│   ├── mix_engine/
│   ├── state_store/
│   ├── authorization/
│   └── revision_control/
├── integration/
│   ├── api/          (HTTP endpoint tests)
│   ├── websocket/    (WS message flow tests)
│   └── database/     (migration + CRUD tests)
└── acceptance/       (full-stack flow tests)
```

### Frontend (TypeScript)

```
web/*/src/
└── __tests__/
    ├── components/
    ├── state/
    └── websocket/
```

### Audio (PipeWire — requires hardware)

```
tests/audio/
├── xrun/
├── latency/
├── mix-engine/
└── transport/
```

## Test Requirements by Feature Type

### General

Every feature must have:
- [ ] Unit test
- [ ] Integration test (where applicable)
- [ ] Acceptance criteria verification

### Audio Features (additional)

- [ ] Hardware test (or clearly marked SIMULATED)
- [ ] Latency measurement
- [ ] XRUN monitoring

### Network Features (additional)

- [ ] Packet loss simulation test
- [ ] Jitter test
- [ ] Reconnect test

## Test Status Labels

```
SIMULATED    — test runs on VPS without real hardware
HARDWARE VERIFIED — tested on real audio hardware (include: hardware, OS, sample rate, buffer, interface, network, method)
```

## Rust Test Conventions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mix_send_gain_bounds() { ... }

    #[tokio::test]
    async fn test_api_channels_list() { ... }
}
```

## CI Gate

All tests must pass in CI before merge:
- `cargo test` (unit + integration)
- `cargo clippy -- -D warnings`
- `cargo fmt --check`
- Frontend typecheck + test
- Security audit (`cargo audit`)

## Quality Gates

- [ ] New feature has ≥1 unit test
- [ ] New API endpoint has integration test
- [ ] WebSocket flows have protocol tests
- [ ] Authorization boundary tested (musician cannot access other mix)
- [ ] Gain boundary test (no output > max defined gain)
- [ ] Limiter behavior verified
- [ ] Reconnect behavior tested
- [ ] Test status labeled (SIMULATED / HARDWARE VERIFIED)
- [ ] No test skipped without documented reason
