# QR passwordless hourly rotation

## Objective
Implement test-only passwordless musician session entry from a valid QR invitation, with automatic hourly invitation rotation, while preserving normal login and server-authoritative scope.

## Scope
- Backend endpoint for invitation-only passwordless session bootstrap.
- Automatic hourly rotation/revocation of active invitation generation.
- Session TTL/revocation tied to invitation generation.
- Engineer/Admin UI state and controls for hourly rotation.
- Musician UI invitation bootstrap without password.
- Focused backend/frontend security tests.
- Update START.md, docs/TODO.md, handoff and log after verified behavior.

## Non-goals
- Production passwordless identity/authentication.
- Physical Raspberry Pi, PipeWire/ALSA, WebRTC/DTLS-SRTP or LAN validation.
- Release publication.

## Security acceptance
- QR contains invitation capability only, never access/refresh token.
- Hash-only persistence; TTL, generation, usage, revocation and rate limits enforced server-side.
- Band scope derived from invitation row; client cannot select band.
- Previous generation and descendant sessions revoked on rotation/deactivation.
- No secrets in logs, localStorage, JWT claims, docs or chat.
- Existing password login remains available.
