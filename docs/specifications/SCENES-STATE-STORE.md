# Scenes and State Store Specification

**Status:** SPECIFICATION — implementation pending
**Scope:** P2 / GAP-027
**Date:** 2026-09-16

## Goal

Define durable scene configuration and transient runtime state without allowing an incomplete runtime checkpoint to be recalled as live configuration.

## Boundaries

- Scene persistence covers control-plane configuration only: channels, mixes, sends, EQ, compressor, limiter, routing and locks.
- Credentials, password hashes, access tokens, session mappings, device identities and transient DSP buffers never enter scene payloads.
- Audio callback never performs DB, filesystem, network or blocking work.
- Restore remains server-authoritative and executes through validated control-plane APIs.
- MVP remains 8 channels, 2 mixes and Channel Mode.

## State classes

| Class | Examples | Durable | Restore source |
|---|---|---:|---|
| Configuration | channel names/gain/mute, sends, mix master, processing, locks | Yes | Scene or explicit config backup |
| Runtime | connected clients, stream status, XRUN counters, jitter, packet loss, current WS sessions | No | Fresh runtime initialization |
| Checkpoint metadata | scene ID, schema version, revision, created/updated timestamps | Yes | Validated scene record |

Transient state must not be serialized into scene records. On process restart, server loads last valid durable scene, initializes runtime state empty, then publishes authoritative snapshot.

## Data model

`Scene` contains:

- stable `id` and non-empty bounded `name`;
- `schema_version`;
- monotonic `revision`;
- `created_at` and `updated_at`;
- strict configuration payload;
- optional parent scene ID for duplication provenance;
- no secrets or runtime fields.

`SceneRevision` is immutable history for each saved scene. Writes use one SQLite transaction: validate payload, insert revision, update scene pointer. Failed validation or persistence changes nothing.

## Durable transition rules

1. Load only scenes with supported schema version and valid semantic ranges.
2. Reject unknown fields; never silently discard persisted data.
3. Save creates a new revision; it does not mutate prior revision payload.
4. Recall validates complete payload before applying any change.
5. Recall applies atomically from control-plane perspective; failure leaves active configuration unchanged.
6. Delete rejects currently active scene unless explicit replacement is selected.
7. Import uses bounded input, strict schema and fresh validation before storage.
8. Export emits only durable scene data.

## Failure and recovery

- Corrupt or unsupported latest scene: keep active in-memory configuration, mark scene load failure, expose structured diagnostic, do not fall back silently to partial data.
- Crash during save: SQLite transaction leaves previous revision and active-scene pointer intact.
- Crash during recall: startup uses last committed active-scene pointer; no transient checkpoint is resumed.
- Runtime disconnects never alter scene content.

## API shape (future implementation)

- `GET /api/v1/scenes`
- `POST /api/v1/scenes`
- `POST /api/v1/scenes/:id/recall`
- `POST /api/v1/scenes/:id/duplicate`
- `DELETE /api/v1/scenes/:id`
- `GET /api/v1/scenes/:id/export`
- `POST /api/v1/scenes/import`

Engineer/Admin only for create, recall, duplicate, delete, import and export. Musician scene access remains deferred until explicit product authorization is specified.

## Acceptance tests

- Fresh migration creates scene tables and is idempotent.
- Save/restart/recall restores complete configuration.
- Two revisions preserve prior immutable payload.
- Unknown fields, unsupported versions, invalid ranges and oversized names/payloads fail closed.
- Failed recall leaves active configuration unchanged.
- Corrupt latest scene produces diagnostic without partial restore.
- Export/import round trip contains no secret or runtime field.
- Concurrent save/recall serializes without lost revision or torn state.

## Traceability

- `docs/ARCHITECTURE-GAPS.md`: GAP-027
- [`docs/decisions/ADR-011-scenes-state-store.md`](../decisions/ADR-011-scenes-state-store.md)
- `docs/decisions/ADR-007-real-time-audio-boundary.md`: realtime exclusion
- `docs/decisions/ADR-009-authentication-bootstrap.md`: secret boundary
- [`docs/DEVELOPMENT-HANDOFF.md`](../DEVELOPMENT-HANDOFF.md): P2 queue

Implementation requires ADR approval before Rust schema or API code.
