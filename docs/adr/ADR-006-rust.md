# ADR-006: Rust as Backend Language

## Status
Accepted

## Context
Open IEM Platform needs a backend language for the control server (REST, WebSocket, state management) and, potentially, for audio processing integration with PipeWire.

Requirements:
- Memory safety (audio server must be stable)
- Async I/O (many concurrent WebSocket clients)
- Realtime-safe primitives (lock-free data structures)
- Good ecosystem for HTTP, WebSocket, SQLite
- Cross-compilation support (ARM64 + x86_64)
- Performance (low latency control plane)

## Decision
Use Rust (stable channel, 2024 edition) for the backend server.

## Rationale
- Memory safety without garbage collector — no GC pauses that could affect control latency
- Excellent async ecosystem (Tokio, axum)
- Lock-free primitives available (`crossbeam`, `atomic`, ring buffers)
- `sqlx` provides compile-time verified SQL queries
- Cross-compilation to ARM64 is first-class (`aarch64-unknown-linux-gnu`)
- PipeWire Rust bindings (`pipewire-rs`) available
- `cargo audit` for supply chain security

## Consequences
**Positive:**
- Memory safe — no buffer overflows, use-after-free
- No GC pauses
- Strong type system catches errors at compile time
- Excellent cross-compilation
- Cargo ecosystem for all required components

**Negative:**
- Longer initial development time vs Python/Node
- PipeWire Rust bindings less mature than C bindings
- Learning curve for audio-safe concurrency patterns

## Technology Stack

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
axum = "0.8"
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio"] }
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1", features = ["v4"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## Alternatives Considered
- **Go**: GC pauses possible; less suited for realtime-safe boundaries
- **C**: Memory unsafe; higher bug risk
- **Python/Node**: GC/GIL issues; not suitable for realtime-adjacent work
