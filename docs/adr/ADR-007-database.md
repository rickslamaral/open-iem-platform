# ADR-007: SQLite as Database

## Status
Accepted

## Context
Open IEM Platform needs persistent storage for: users, musicians, channels, mixes, scenes, permissions, audit events, and system configuration.

Requirements:
- Embedded (no separate database server process)
- Suitable for Raspberry Pi deployment
- Supports migrations
- Rust ecosystem support
- Reliable for single-writer, many-reader workload

## Decision
Use SQLite with WAL (Write-Ahead Logging) mode via `sqlx`.

## Rationale
- Embedded — no PostgreSQL/MySQL server on Raspberry Pi
- WAL mode provides concurrent reads with single writer
- `sqlx` provides compile-time verified SQL with async support
- `sqlx-migrate` for schema migrations
- Sufficient for expected workload (≤16 musicians, ≤100 scenes, low write frequency)
- SQLite is battle-tested and reliable
- Simple backup (single file)

## Configuration

```sql
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;
PRAGMA busy_timeout=5000;
```

## Schema (Initial Entities)

```
users           (id, username, role, password_hash, created_at)
musicians       (id, user_id, name, assigned_mix_id, created_at)
channels        (id, name, gain_db, pan, mute, enabled, created_at, updated_at)
mixes           (id, name, musician_id, master_db, limiter_enabled, limiter_threshold_db, revision, created_at, updated_at)
mix_sends       (channel_id, mix_id, gain_db, pan, mute, solo, enabled, locked)
devices         (id, name, type, status, last_seen_at)
scenes          (id, name, snapshot_json, created_at, updated_at)
permissions     (user_id, resource, action)
system_config   (key, value, updated_at)
audit_events    (id, user_id, action, resource, detail_json, created_at)
```

## Consequences
**Positive:**
- Simple deployment (single file)
- No network dependency for database
- Good Rust support via sqlx
- Sufficient performance for IEM use case

**Negative:**
- Single writer limitation (not a problem for this use case)
- Not suitable if scaling beyond a single server (not in scope)

## Alternatives Considered
- **PostgreSQL**: Over-engineered for embedded Raspberry Pi use
- **sled**: Pure Rust embedded DB but less SQL tooling
- **DuckDB**: Analytical focus, not suited for OLTP
