# Phase 86 — Lock-free Broadcast Fan-out — Review

**Date:** 2026-09-13
**Status:** PASS (local gates)

---

## Scope

Remove `mix_assignment_lock` from the WebSocket broadcast fan-out receiver paths to eliminate O(sessions) contention per broadcast event. The lock is retained on all mutation sender paths.

---

## Deliverables

| Deliverable | Status |
|-------------|--------|
| `mix_assignment_lock` removed from send-delta fan-out receiver | ✅ Done |
| `mix_assignment_lock` removed from master-delta fan-out receiver | ✅ Done |
| Comment documenting intentional stale-read trade-off | ✅ Done |
| `docs/TODO.md` item checked | ✅ Done |
| `CHANGELOG.md` updated | ✅ Done |
| `docs/DEVELOPMENT-LOG.md` entry added | ✅ Done |

---

## Design Decision

### Before

```rust
// Both fan-out branches (send-delta and master-delta):
let assignment_guard = state.mix_assignment_lock.lock().await;
// ... ownership check + ack_json build ...
drop(assignment_guard);
```

### After

```rust
// Lock-free: ownership check is a read-only DB query.
// Mutation senders still hold the lock (inbound WS path and HTTP routes).
let should_forward = match claims.role {
    Role::Admin | Role::Engineer => true,
    Role::Musician => musician_assigned_to_mix(&state, claims.user_id, delta.mix_index),
};
```

### Rationale

Previously, every connected WebSocket session had to acquire `mix_assignment_lock` before reading musician assignment from the SQLite DB during each broadcast fan-out. With N connected sessions and a high broadcast rate, this created O(N × events) serialization points on a single async mutex — all fan-out reads queue behind inbound assignment mutations.

The receiver-side ownership check is a **read-only** query. The ordering requirement (mutation must be fully committed before broadcast is observed) is satisfied by the sender, which holds the lock for the entire mutation+publish sequence. Removing the lock from receivers does not break this invariant.

### Trade-off: Narrow Stale-read Race

If a musician's assignment changes concurrently with a broadcast event:
- They may receive **one extra delta** for a mix they just left.
- They may **miss one delta** for a mix they just joined.

Both cases are bounded and harmless:
1. The client treats `SendAck` / `MasterAck` as idempotent state updates.
2. When the client receives a `State` revision message, it reconciles via REST snapshot — covering any gap introduced by the race.

This race existed before with the lock as well (the lock only serialised the read with mutations, not the entire lifecycle of what the client sees). Removing the lock makes the race slightly wider but not qualitatively different.

---

## Local Gates

| Gate | Result |
|------|--------|
| `cargo fmt --all -- --check` | ✅ PASS |
| `cargo clippy --all-targets -- -D warnings` | ✅ PASS |
| `cargo test --workspace` | ✅ PASS (all tests, no regressions) |
| Static security scan (secrets/injection/eval/pickle) | ✅ CLEAN |
| Independent reviewer subagent | ✅ `passed=true`, no security_concerns, no logic_errors |

---

## Reviewer Suggestions (non-blocking)

- Add an integration test that fires a rapid assignment-change concurrent with a broadcast fan-out to confirm the client reconciliation path (REST snapshot on `State` revision) is exercised under the race. Deferred to future phase.

---

## Limitations

- Remote CI not yet verified (runner quota block, existing limitation since Phase 45).
- Hardware (Raspberry Pi 5), real audio, PipeWire/ALSA, and release `v0.3.1` remain pending.
