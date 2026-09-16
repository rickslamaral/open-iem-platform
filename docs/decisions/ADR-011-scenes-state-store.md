# ADR-011: Durable Scenes and Transient Runtime State

- **Status:** PROPOSED — implementation pending
- **Date:** 2026-09-16
- **Scope:** P2 / GAP-027

## Context

Open IEM needs scene save/recall without persisting incomplete runtime work. Current `config-backup` is a library and does not provide scene lifecycle, immutable revisions or an operational state store.

## Decision

Adopt strict, versioned scene records backed by SQLite. Persist control-plane configuration only. Keep runtime state transient. Save and recall use transactions and validated complete payloads. Unknown persisted fields, unsupported versions, invalid values and oversized inputs fail closed.

Scene revisions are immutable. A scene pointer identifies active configuration. Recall validates before mutation and must not partially apply. Startup loads last committed valid configuration and initializes connections, stream state, metrics and DSP runtime state fresh.

## Authorization

Engineer/Admin may create, recall, duplicate, delete, import and export scenes. Musician scene operations remain deferred until authorization contract exists.

## Rejected alternatives

- Persisting full `AppState`: mixes secrets and transient sessions with durable configuration.
- Resuming arbitrary runtime checkpoints: can resume incomplete audio work.
- Non-strict deserialization: silently drops unknown fields and causes data loss on rewrite.
- In-place scene mutation: destroys rollback and auditability.

## Consequences

Positive: deterministic recovery, rollback history, explicit schema evolution and safe failure.

Cost: migrations, payload validation, transaction tests and API work before scenes become user-facing.

## Acceptance evidence

Required tests: idempotent migration; save/restart/recall; immutable revisions; strict schema; bounds; failed recall atomicity; corrupt-record handling; secret/runtime exclusion; concurrent save/recall serialization. Runtime and hardware validation remain separate.

## Related documents

- [`docs/specifications/SCENES-STATE-STORE.md`](../specifications/SCENES-STATE-STORE.md)
- `docs/ARCHITECTURE-GAPS.md` GAP-027
- `docs/decisions/ADR-007-real-time-audio-boundary.md`
- `docs/decisions/ADR-009-authentication-bootstrap.md`
