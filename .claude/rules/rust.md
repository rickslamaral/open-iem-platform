---
paths:
  - "server/**/*.rs"
  - "server/**/Cargo.toml"
---

# Rust Rules

- Run `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --workspace` after changes.
- Avoid `unsafe`; require an ADR and security review when unavoidable.
- Preserve realtime audio constraints: bounded work, no blocking calls in audio callbacks, finite samples.
- Use Ed25519 JWT and Argon2id. Do not introduce symmetric JWT or weak password hashing.
