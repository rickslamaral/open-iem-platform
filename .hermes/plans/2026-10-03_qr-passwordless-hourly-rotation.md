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
- QR contains invitation capability only, never access/refresh token. Capability token is not bearer credential; exchange uses HTTPS POST.
- Bootstrap may carry capability in URL fragment only when needed; server sets strict `Referrer-Policy`, client immediately calls `history.replaceState`, analytics/proxy access logs redact fragment/query data, and no token enters browser history after bootstrap.
- Hash-only persistence; token TTL is 10 minutes, single-use consumption is atomic, and failed/replayed tokens never issue sessions.
- Invitation generation rotates on UTC hour boundary using server UTC clock. Rotation transaction marks prior generation revoked before creating next generation; restart reruns reconciliation idempotently. No grace period.
- Each invitation stores generation ID, token hash, issued/expiry timestamps, consumed/revoked state, and server-derived band ID. Client-provided band IDs are ignored/rejected.
- Exchange transaction locks or atomically updates eligible row (`unconsumed AND unrevoked AND expires_at > now`) before issuing credentials. Concurrent exchanges yield at most one success. Failed credential issuance rolls back consumption; token-existence responses stay generic and timing-bounded.
- Session lifetime is the lesser of configured session TTL and remaining generation lifetime. Rotation/deactivation revokes all sessions linked to prior generation; no descendant session survives revocation.
- Rate limits apply per source IP and invitation hash: 5 exchange attempts per minute, with failed attempts counted; rejected requests never reveal whether token exists. Limits reset by monotonic server time and fail closed on limiter storage errors.
- Existing password login remains available.
- No secrets in logs, localStorage, JWT claims, docs or chat.

## Required tests
- Token leakage negative tests: access/refresh bearer tokens never appear in URL, logs, JWT or localStorage; capability fragment is removed immediately and referrer/log redaction is verified.
- Replay and concurrent exchange tests: exactly one success for same token.
- Rotation boundary, restart reconciliation, expiry, deactivation and linked-session revocation tests.
- Band-scope test: client cannot select another band.
- Rate-limit and generic-error tests.

## Verification evidence
- Before commit, run `git diff --cached`, added-line secret/injection pattern scans, `scripts/validate-docs.sh`, and affected tests.
- Record only command output actually executed. Do not claim `/root/scan_patterns.py` results when scanner is unavailable.
