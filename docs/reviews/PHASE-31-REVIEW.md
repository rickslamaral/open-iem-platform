# Working-Tree Review — JWT Post-Issuance Revocation

> Working-tree status: Phase 31 is uncommitted work. It is not a completed release or merged progress.

## Scope

Open IEM access JWTs now carry optional-for-legacy-tests `session_id`; production login and refresh tokens carry refresh-token session ID. SQLite stores `jti`, `user_id`, `session_id`, `expires_at`, and `revoked` in `access_sessions`.

## Security behavior

- Login creates refresh row and access mapping before returning access token.
- Refresh rotates refresh token and creates new access mapping transactionally.
- Middleware verifies JWT, requires session association, checks active DB mapping and existing user.
- Logout, admin session revoke, and user deletion revoke access mappings.
- WebSocket handshake, keepalive and inbound-message paths re-check access-session validity; established connections can be closed after revocation on the next check interval or message.

## Verification

Current API-scoped verification: `cargo fmt --manifest-path server/Cargo.toml --all -- --check`, `cargo test -p api-server --test integration`, and `cargo clippy -p api-server --all-targets -- -D warnings` pass. API tests: 38 unit + 57 integration. Hardware, PipeWire, WebRTC media, and Raspberry Pi remain unvalidated.

## Migração de bancos existentes

A criação de `access_sessions` é compatível com bancos existentes, mas não há como reconstruir associações entre access JWTs já emitidos e refresh sessions antigas. Esses access JWTs falham fechado após atualização; o operador deve fazer login novamente. Refresh tokens antigos continuam utilizáveis para rotação e passam a gerar mapeamentos novos. Expiração, revogação ou ausência de refresh token exige novo login.

Release blockers remain: working-tree-only changes and no CI validation.
